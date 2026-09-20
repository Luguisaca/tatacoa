use crate::{
    Artifact, ArtifactId, ArtifactPreview, Engagement, EngagementId, Error, ExecutionId,
    ExportMode, LEGACY_MANIFEST_SCHEMA_VERSION, MANIFEST_SCHEMA_VERSION, Manifest,
    PlainExportAuthorization, Result, SecurityProfile, authorize_plain_export,
    validate_artifact_provenance, validate_knowledge_card, validate_replay_recipe,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest as _, Sha256};

const MAX_METADATA_BYTES: u64 = 2 * 1024 * 1024;
pub const MAX_ARTIFACT_PREVIEW_BYTES: u64 = 1024 * 1024;

pub fn unix_ms_observed() -> Result<u128> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .map_err(|error| Error::Execution(format!("system clock is before Unix epoch: {error}")))
}

pub fn create_engagement(
    workspace: &Path,
    name: String,
    security_profile: SecurityProfile,
) -> Result<Engagement> {
    if name.trim().is_empty() {
        return Err(Error::InvalidManifest(
            "engagement name must not be empty".to_owned(),
        ));
    }
    fs::create_dir_all(workspace)
        .map_err(|source| Error::io("create workspace directory", source))?;
    reject_symlink(workspace, "workspace")?;
    let engagements_root = workspace.join("engagements");
    fs::create_dir_all(&engagements_root)
        .map_err(|source| Error::io("create engagements directory", source))?;
    reject_symlink(&engagements_root, "engagements directory")?;

    let engagement = Engagement {
        id: EngagementId::new(),
        name,
        created_unix_ms_observed: unix_ms_observed()?,
        security_profile,
    };
    let engagement_root = engagements_root.join(engagement.id.as_str());
    fs::create_dir(&engagement_root)
        .map_err(|source| Error::io("create isolated engagement directory", source))?;
    fs::create_dir(engagement_root.join("objects"))
        .map_err(|source| Error::io("create engagement objects directory", source))?;
    fs::create_dir(engagement_root.join("manifests"))
        .map_err(|source| Error::io("create engagement manifests directory", source))?;
    for directory in [
        "context/scopes",
        "context/environments",
        "context/targets",
        "context/sessions",
        "knowledge",
        "replay",
    ] {
        fs::create_dir_all(engagement_root.join(directory))
            .map_err(|source| Error::io("create engagement data directory", source))?;
    }
    write_json_new_atomic(&engagement_root.join("engagement.json"), &engagement)?;
    Ok(engagement)
}

pub fn load_engagement(workspace: &Path, id: &EngagementId) -> Result<Engagement> {
    let root = engagement_root(workspace, id)?;
    let engagement: Engagement = read_json_limited(&root.join("engagement.json"))?;
    if &engagement.id != id {
        return Err(Error::InvalidManifest(
            "engagement ID does not match its isolated directory".to_owned(),
        ));
    }
    Ok(engagement)
}

pub fn list_engagements(workspace: &Path) -> Result<Vec<Engagement>> {
    reject_symlink(workspace, "workspace")?;
    let root = workspace.join("engagements");
    reject_symlink(&root, "engagements directory")?;
    let mut paths = fs::read_dir(root)
        .map_err(|source| Error::io("read engagements directory", source))?
        .map(|entry| {
            entry
                .map(|value| value.path())
                .map_err(|source| Error::io("read engagement entry", source))
        })
        .collect::<Result<Vec<_>>>()?;
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            reject_symlink(&path, "engagement entry")?;
            if !path.is_dir() {
                return Err(Error::InvalidPath(
                    "engagement entry is not a directory".to_owned(),
                ));
            }
            let value = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| {
                    Error::InvalidPath("engagement directory name is not UTF-8".to_owned())
                })?;
            let id = value.parse()?;
            load_engagement(workspace, &id)
        })
        .collect()
}

pub fn engagement_root(workspace: &Path, id: &EngagementId) -> Result<PathBuf> {
    let root = workspace.join("engagements").join(id.as_str());
    let canonical_workspace = workspace
        .canonicalize()
        .map_err(|source| Error::io("canonicalize workspace", source))?;
    let canonical_root = root
        .canonicalize()
        .map_err(|source| Error::io("find engagement directory", source))?;
    reject_symlink(&root, "engagement directory")?;
    if !canonical_root.starts_with(&canonical_workspace) {
        return Err(Error::InvalidPath(
            "engagement directory escapes workspace".to_owned(),
        ));
    }
    Ok(canonical_root)
}

pub fn store_execution_manifest(workspace: &Path, manifest: &Manifest) -> Result<PathBuf> {
    validate_manifest_links(manifest)?;
    let manifests = engagement_existing_subdirectory(
        workspace,
        &manifest.engagement.id,
        Path::new("manifests"),
    )?;
    let path = manifests.join(format!("{}.json", manifest.execution.id));
    write_json_new_atomic(&path, manifest)?;
    Ok(path)
}

pub fn load_execution_manifest(
    workspace: &Path,
    engagement_id: &EngagementId,
    execution_id: &ExecutionId,
) -> Result<Manifest> {
    let manifests =
        engagement_existing_subdirectory(workspace, engagement_id, Path::new("manifests"))?;
    let path = manifests.join(format!("{execution_id}.json"));
    let manifest: Manifest = read_json_limited(&path)?;
    validate_manifest_links(&manifest)?;
    if &manifest.engagement.id != engagement_id || &manifest.execution.id != execution_id {
        return Err(Error::InvalidManifest(
            "manifest identity does not match requested engagement/execution".to_owned(),
        ));
    }
    Ok(manifest)
}

pub fn read_artifact_preview(
    workspace: &Path,
    engagement_id: &EngagementId,
    execution_id: &ExecutionId,
    artifact_id: &ArtifactId,
) -> Result<ArtifactPreview> {
    let manifest = load_execution_manifest(workspace, engagement_id, execution_id)?;
    let artifact = manifest
        .artifacts
        .iter()
        .find(|artifact| &artifact.id == artifact_id)
        .cloned()
        .ok_or_else(|| {
            Error::InvalidManifest("artifact is not declared by execution".to_owned())
        })?;
    let path = artifact_source_path(workspace, engagement_id, &artifact)?;
    let mut file = File::open(path).map_err(|source| Error::io("open artifact preview", source))?;
    let mut hasher = Sha256::new();
    let mut total = 0_u64;
    let mut preview = Vec::with_capacity(
        usize::try_from(artifact.size_bytes.min(MAX_ARTIFACT_PREVIEW_BYTES)).unwrap_or(0),
    );
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|source| Error::io("read artifact preview", source))?;
        if read == 0 {
            break;
        }
        total = total
            .checked_add(read as u64)
            .ok_or_else(|| Error::InvalidManifest("artifact size overflow".to_owned()))?;
        hasher.update(&buffer[..read]);
        let remaining = MAX_ARTIFACT_PREVIEW_BYTES.saturating_sub(preview.len() as u64) as usize;
        preview.extend_from_slice(&buffer[..read.min(remaining)]);
    }
    let mut digest = String::with_capacity(64);
    for byte in hasher.finalize() {
        use std::fmt::Write as _;
        write!(&mut digest, "{byte:02x}")
            .map_err(|error| Error::Execution(format!("format SHA-256 digest: {error}")))?;
    }
    if total != artifact.size_bytes || digest != artifact.digest.value {
        return Err(Error::InvalidManifest(
            "artifact changed after capture; preview denied".to_owned(),
        ));
    }
    Ok(ArtifactPreview {
        artifact,
        bytes: preview,
        truncated_for_preview: total > MAX_ARTIFACT_PREVIEW_BYTES,
    })
}

pub fn list_execution_manifests(
    workspace: &Path,
    engagement_id: &EngagementId,
) -> Result<Vec<Manifest>> {
    let root = engagement_existing_subdirectory(workspace, engagement_id, Path::new("manifests"))?;
    let mut paths = fs::read_dir(root)
        .map_err(|source| Error::io("read execution manifests", source))?
        .map(|entry| {
            entry
                .map(|value| value.path())
                .map_err(|source| Error::io("read execution manifest entry", source))
        })
        .collect::<Result<Vec<_>>>()?;
    paths.sort();
    paths
        .into_iter()
        .filter(|path| path.extension().is_some_and(|value| value == "json"))
        .map(|path| {
            let manifest: Manifest = read_json_limited(&path)?;
            validate_manifest_links(&manifest)?;
            if &manifest.engagement.id != engagement_id {
                return Err(Error::InvalidManifest(
                    "execution manifest belongs to another engagement".to_owned(),
                ));
            }
            Ok(manifest)
        })
        .collect()
}

pub fn read_bundle_manifest(bundle_root: &Path) -> Result<Manifest> {
    reject_symlink(bundle_root, "bundle root")?;
    let manifest_path = bundle_root.join("manifest.json");
    let manifest: Manifest = read_json_limited(&manifest_path)?;
    validate_manifest_links(&manifest)?;
    Ok(manifest)
}

pub fn export_bundle(
    workspace: &Path,
    manifest: &Manifest,
    destination: &Path,
    authorization: PlainExportAuthorization,
) -> Result<()> {
    validate_manifest_links(manifest)?;
    authorize_plain_export(manifest.engagement.security_profile, authorization)?;
    if destination.exists() {
        return Err(Error::Conflict(format!(
            "bundle destination already exists: {}",
            destination.display()
        )));
    }
    let parent = destination.parent().ok_or_else(|| {
        Error::InvalidPath("bundle destination has no parent directory".to_owned())
    })?;
    fs::create_dir_all(parent)
        .map_err(|source| Error::io("create bundle parent directory", source))?;
    let staging = parent.join(format!(
        ".{}.{}.partial",
        destination
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("tatacoa-bundle"),
        uuid::Uuid::new_v4().simple()
    ));
    fs::create_dir(&staging)
        .map_err(|source| Error::io("create bundle staging directory", source))?;
    for directory in ["objects", "knowledge", "replay", "verification"] {
        fs::create_dir(staging.join(directory))
            .map_err(|source| Error::io("create bundle directory", source))?;
    }

    let objects_root =
        engagement_existing_subdirectory(workspace, &manifest.engagement.id, Path::new("objects"))?;
    for artifact in &manifest.artifacts {
        let file_name = format!("{}.bin", artifact.id);
        let source = objects_root.join(&file_name);
        reject_symlink(&source, "workspace artifact")?;
        let destination_file = staging.join("objects").join(file_name);
        copy_new(&source, &destination_file)?;
    }

    let exported = prepare_export_manifest(workspace, manifest, ExportMode::Plain)?;
    write_json_new_atomic(&staging.join("manifest.json"), &exported)?;
    fs::rename(&staging, destination)
        .map_err(|source| Error::io("commit portable bundle", source))?;
    Ok(())
}

pub(crate) fn prepare_export_manifest(
    workspace: &Path,
    manifest: &Manifest,
    export_mode: ExportMode,
) -> Result<Manifest> {
    validate_manifest_links(manifest)?;
    let mut exported = manifest.clone();
    exported.export_mode = export_mode;
    exported.knowledge_cards = crate::workspace::load_associated_knowledge(
        workspace,
        &manifest.engagement.id,
        &manifest.execution.id,
    )?;
    exported.replay_recipes = crate::workspace::load_associated_replay(
        workspace,
        &manifest.engagement.id,
        &manifest.execution.id,
    )?;
    validate_manifest_links(&exported)?;
    Ok(exported)
}

pub(crate) fn write_json_new_atomic<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| Error::InvalidPath(format!("invalid file name: {}", path.display())))?;
    let temporary = path.with_file_name(format!(".{file_name}.{}.partial", uuid::Uuid::new_v4()));
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|source| Error::io("create temporary JSON file", source))?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, value)?;
    writer
        .write_all(b"\n")
        .map_err(|source| Error::io("finish JSON file", source))?;
    writer
        .flush()
        .map_err(|source| Error::io("flush JSON file", source))?;
    writer
        .get_ref()
        .sync_all()
        .map_err(|source| Error::io("sync JSON file", source))?;
    drop(writer);
    fs::rename(&temporary, path).map_err(|source| Error::io("commit JSON file", source))?;
    Ok(())
}

pub(crate) fn read_json_limited<T: DeserializeOwned>(path: &Path) -> Result<T> {
    reject_symlink(path, "JSON input")?;
    let file = File::open(path).map_err(|source| Error::io("open JSON file", source))?;
    let metadata = file
        .metadata()
        .map_err(|source| Error::io("inspect JSON file", source))?;
    if metadata.len() > MAX_METADATA_BYTES {
        return Err(Error::InvalidManifest(format!(
            "JSON exceeds {MAX_METADATA_BYTES} byte limit"
        )));
    }
    let reader = BufReader::new(file.take(MAX_METADATA_BYTES + 1));
    Ok(serde_json::from_reader(reader)?)
}

pub(crate) fn validate_manifest_links(manifest: &Manifest) -> Result<()> {
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION
        && manifest.schema_version != LEGACY_MANIFEST_SCHEMA_VERSION
    {
        return Err(Error::InvalidManifest(format!(
            "unsupported schema version: {}",
            manifest.schema_version
        )));
    }
    if manifest.execution.engagement_id != manifest.engagement.id {
        return Err(Error::InvalidManifest(
            "execution belongs to a different engagement".to_owned(),
        ));
    }
    if manifest.execution.adapter != crate::GenericExecutionAdapter::NAME {
        return Err(Error::InvalidManifest(format!(
            "unsupported execution adapter: {}",
            manifest.execution.adapter
        )));
    }
    if manifest.schema_version == MANIFEST_SCHEMA_VERSION {
        let context = manifest.context.as_ref().ok_or_else(|| {
            Error::InvalidManifest("alpha v2 manifest requires full execution context".to_owned())
        })?;
        context.validate(&manifest.engagement.id)?;
        let execution_context = manifest.execution.context.as_ref().ok_or_else(|| {
            Error::InvalidManifest("alpha v2 execution requires context IDs".to_owned())
        })?;
        if execution_context != &context.ids() {
            return Err(Error::InvalidManifest(
                "execution context IDs do not match embedded context".to_owned(),
            ));
        }
    } else if manifest.context.is_some()
        || manifest.execution.context.is_some()
        || !manifest.knowledge_cards.is_empty()
        || !manifest.replay_recipes.is_empty()
    {
        return Err(Error::InvalidManifest(
            "alpha v1 manifest cannot claim Sprint 02 relationships".to_owned(),
        ));
    }
    let mut ids = std::collections::HashSet::new();
    for artifact in &manifest.artifacts {
        if artifact.engagement_id != manifest.engagement.id
            || artifact.execution_id != manifest.execution.id
        {
            return Err(Error::InvalidManifest(format!(
                "artifact {} has a cross-context reference",
                artifact.id
            )));
        }
        if !ids.insert(artifact.id.as_str()) {
            return Err(Error::InvalidManifest(format!(
                "duplicate artifact ID: {}",
                artifact.id
            )));
        }
        if artifact.digest.algorithm != "SHA-256"
            || artifact.digest.value.len() != 64
            || !artifact
                .digest
                .value
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(Error::InvalidManifest(format!(
                "artifact {} has an invalid SHA-256 digest",
                artifact.id
            )));
        }
        crate::ensure_safe_relative_path(Path::new(&artifact.path))?;
        let expected = format!("objects/{}.bin", artifact.id);
        if artifact.path != expected {
            return Err(Error::InvalidManifest(format!(
                "artifact {} path must be {expected}",
                artifact.id
            )));
        }
    }
    validate_artifact_provenance(&manifest.artifacts)?;
    let mut knowledge_ids = std::collections::HashSet::new();
    for card in &manifest.knowledge_cards {
        if !knowledge_ids.insert(card.id.as_str()) {
            return Err(Error::InvalidManifest(format!(
                "duplicate knowledge card ID: {}",
                card.id
            )));
        }
        validate_knowledge_card(card, &manifest.engagement.id, &manifest.execution.id)?;
    }
    let mut replay_ids = std::collections::HashSet::new();
    for recipe in &manifest.replay_recipes {
        if !replay_ids.insert(recipe.id.as_str()) {
            return Err(Error::InvalidManifest(format!(
                "duplicate replay recipe ID: {}",
                recipe.id
            )));
        }
        validate_replay_recipe(recipe, &manifest.engagement.id, &manifest.execution.id)?;
        if manifest.execution.context.as_ref() != Some(&recipe.context) {
            return Err(Error::InvalidManifest(format!(
                "replay recipe {} context differs from its source execution",
                recipe.id
            )));
        }
    }
    Ok(())
}

fn copy_new(source: &Path, destination: &Path) -> Result<()> {
    let mut reader = BufReader::new(
        File::open(source).map_err(|source| Error::io("open workspace artifact", source))?,
    );
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|source| Error::io("create bundle artifact", source))?;
    let mut writer = BufWriter::new(file);
    std::io::copy(&mut reader, &mut writer)
        .map_err(|source| Error::io("copy bundle artifact", source))?;
    writer
        .flush()
        .map_err(|source| Error::io("flush bundle artifact", source))?;
    writer
        .get_ref()
        .sync_all()
        .map_err(|source| Error::io("sync bundle artifact", source))?;
    Ok(())
}

pub(crate) fn reject_symlink(path: &Path, role: &'static str) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|source| Error::io("inspect filesystem object", source))?;
    if metadata.file_type().is_symlink() {
        return Err(Error::InvalidPath(format!(
            "{role} must not be a symbolic link/reparse point: {}",
            path.display()
        )));
    }
    Ok(())
}

pub(crate) fn engagement_existing_subdirectory(
    workspace: &Path,
    engagement_id: &EngagementId,
    relative: &Path,
) -> Result<PathBuf> {
    validate_engagement_relative(relative)?;
    let root = engagement_root(workspace, engagement_id)?;
    let mut current = root.clone();
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err(Error::InvalidPath(
                "engagement subdirectory contains a non-normal component".to_owned(),
            ));
        };
        current.push(name);
        reject_symlink(&current, "engagement subdirectory")?;
        let canonical = current
            .canonicalize()
            .map_err(|source| Error::io("canonicalize engagement subdirectory", source))?;
        if !canonical.starts_with(&root) {
            return Err(Error::InvalidPath(format!(
                "engagement subdirectory escapes its root: {}",
                relative.display()
            )));
        }
    }
    if !current.is_dir() {
        return Err(Error::InvalidPath(format!(
            "engagement subpath is not a directory: {}",
            relative.display()
        )));
    }
    Ok(current)
}

pub(crate) fn ensure_engagement_subdirectory(
    workspace: &Path,
    engagement_id: &EngagementId,
    relative: &Path,
) -> Result<PathBuf> {
    validate_engagement_relative(relative)?;
    let root = engagement_root(workspace, engagement_id)?;
    let mut current = root.clone();
    for component in relative.components() {
        let Component::Normal(name) = component else {
            return Err(Error::InvalidPath(
                "engagement subdirectory contains a non-normal component".to_owned(),
            ));
        };
        current.push(name);
        if current.exists() {
            reject_symlink(&current, "engagement subdirectory")?;
            if !current.is_dir() {
                return Err(Error::InvalidPath(format!(
                    "engagement subpath is not a directory: {}",
                    current.display()
                )));
            }
        } else {
            fs::create_dir(&current)
                .map_err(|source| Error::io("create engagement subdirectory", source))?;
        }
        let canonical = current
            .canonicalize()
            .map_err(|source| Error::io("canonicalize engagement subdirectory", source))?;
        if !canonical.starts_with(&root) {
            return Err(Error::InvalidPath(format!(
                "engagement subdirectory escapes its root: {}",
                relative.display()
            )));
        }
    }
    Ok(current)
}

fn validate_engagement_relative(relative: &Path) -> Result<()> {
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || !relative
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(Error::InvalidPath(format!(
            "invalid engagement-relative directory: {}",
            relative.display()
        )));
    }
    Ok(())
}

pub(crate) fn artifact_source_path(
    workspace: &Path,
    engagement_id: &EngagementId,
    artifact: &Artifact,
) -> Result<PathBuf> {
    if &artifact.engagement_id != engagement_id {
        return Err(Error::InvalidManifest(
            "artifact belongs to another engagement".to_owned(),
        ));
    }
    Ok(
        engagement_existing_subdirectory(workspace, engagement_id, Path::new("objects"))?
            .join(format!("{}.bin", artifact.id)),
    )
}
