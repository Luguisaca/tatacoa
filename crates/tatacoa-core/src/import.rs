use crate::bundle::{
    read_bundle_manifest, reject_symlink, unix_ms_observed, write_json_new_atomic,
};
use crate::{
    CONTINUITY_SCHEMA_VERSION, ContinuityState, ContinuityStatus, Engagement, Error, ExportMode,
    MANIFEST_SCHEMA_VERSION, Manifest, Result, SecretPassword, SecurityProfile, compute_sha256,
};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Imports one verified Plain v2 bundle as a continuable engagement. The received
/// bundle is retained byte-for-byte under `received/plain`; IDs and historical
/// records are not rewritten. The imported work starts paused.
pub fn import_plain_bundle(
    workspace: &Path,
    source: &Path,
    authorization_revalidated: bool,
) -> Result<Engagement> {
    if !authorization_revalidated {
        return Err(Error::InvalidManifest(
            "current authorization must be revalidated before importing".to_owned(),
        ));
    }
    let manifest = verify_plain_bundle_for_import(source)?;
    manifest.context.as_ref().ok_or_else(|| {
        Error::InvalidManifest("continuable import requires v2 context".to_owned())
    })?;
    if manifest
        .artifacts
        .iter()
        .any(|artifact| artifact.evidence_state != crate::EvidenceState::Captured)
    {
        return Err(Error::InvalidManifest(
            "continuable import accepts only CAPTURED artifacts in this Alpha".to_owned(),
        ));
    }
    if matches!(
        manifest.engagement.security_profile,
        SecurityProfile::HighSensitivity | SecurityProfile::Custom
    ) {
        return Err(Error::InvalidManifest(
            "Plain import is denied for this Security Profile".to_owned(),
        ));
    }
    fs::create_dir_all(workspace).map_err(|e| Error::io("create workspace", e))?;
    reject_symlink(workspace, "workspace")?;
    let engagements = workspace.join("engagements");
    fs::create_dir_all(&engagements).map_err(|e| Error::io("create engagements", e))?;
    reject_symlink(&engagements, "engagements")?;
    let destination = engagements.join(manifest.engagement.id.as_str());
    if destination.exists() {
        return Err(Error::Conflict(format!(
            "engagement already exists: {}",
            manifest.engagement.id
        )));
    }
    let staging = workspace.join(format!(
        ".tatacoa-import-{}.partial",
        uuid::Uuid::new_v4().simple()
    ));
    fs::create_dir(&staging).map_err(|e| Error::io("create import staging", e))?;
    let result = (|| -> Result<()> {
        for dir in [
            "objects",
            "manifests",
            "context/scopes",
            "context/environments",
            "context/targets",
            "context/sessions",
            "knowledge",
            "replay",
            "continuity",
            "received/plain/objects",
            "received/plain/knowledge",
            "received/plain/replay",
            "received/plain/verification",
        ] {
            fs::create_dir_all(staging.join(dir))
                .map_err(|e| Error::io("create import directory", e))?;
        }
        copy_file(
            source.join("manifest.json").as_path(),
            &staging.join("received/plain/manifest.json"),
        )?;
        for artifact in &manifest.artifacts {
            let name = format!("{}.bin", artifact.id);
            copy_file(
                &source.join("objects").join(&name),
                &staging.join("received/plain/objects").join(&name),
            )?;
        }
        // Validate the retained copy, not only the mutable external source.
        verify_plain_bundle_for_import(&staging.join("received/plain"))?;
        for artifact in &manifest.artifacts {
            let name = format!("{}.bin", artifact.id);
            copy_file(
                &staging.join("received/plain/objects").join(&name),
                &staging.join("objects").join(&name),
            )?;
            let (size, digest) = compute_sha256(&staging.join("objects").join(name))?;
            if size != artifact.size_bytes || !digest.eq_ignore_ascii_case(&artifact.digest.value) {
                return Err(Error::InvalidManifest(
                    "imported artifact changed while materializing".to_owned(),
                ));
            }
        }
        write_import_records(&staging, &manifest)?;
        if destination.exists() {
            return Err(Error::Conflict(
                "engagement appeared during import".to_owned(),
            ));
        }
        fs::rename(&staging, &destination)
            .map_err(|e| Error::io("commit imported engagement", e))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result.map(|()| manifest.engagement)
}

pub fn import_encrypted_bundle(
    workspace: &Path,
    source: &Path,
    password: &SecretPassword,
    authorization_revalidated: bool,
) -> Result<Engagement> {
    if !authorization_revalidated {
        return Err(Error::InvalidManifest(
            "current authorization must be revalidated before importing".to_owned(),
        ));
    }
    let manifest = crate::verify_encrypted_bundle(source, password)?;
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION
        || manifest.export_mode != ExportMode::Encrypted
        || manifest.context.is_none()
    {
        return Err(Error::InvalidManifest(
            "continuable Encrypted import requires alpha v2 context".to_owned(),
        ));
    }
    if manifest.engagement.security_profile == SecurityProfile::Custom
        || manifest
            .artifacts
            .iter()
            .any(|item| item.evidence_state != crate::EvidenceState::Captured)
    {
        return Err(Error::InvalidManifest(
            "Encrypted import profile or Evidence state is not supported".to_owned(),
        ));
    }
    fs::create_dir_all(workspace).map_err(|e| Error::io("create workspace", e))?;
    reject_symlink(workspace, "workspace")?;
    let engagements = workspace.join("engagements");
    fs::create_dir_all(&engagements).map_err(|e| Error::io("create engagements", e))?;
    reject_symlink(&engagements, "engagements")?;
    let destination = engagements.join(manifest.engagement.id.as_str());
    if destination.exists() {
        return Err(Error::Conflict(format!(
            "engagement already exists: {}",
            manifest.engagement.id
        )));
    }
    let staging = workspace.join(format!(
        ".tatacoa-import-{}.partial",
        uuid::Uuid::new_v4().simple()
    ));
    fs::create_dir(&staging).map_err(|e| Error::io("create import staging", e))?;
    let result = (|| -> Result<()> {
        for dir in [
            "objects",
            "manifests",
            "context/scopes",
            "context/environments",
            "context/targets",
            "context/sessions",
            "knowledge",
            "replay",
            "continuity",
            "received",
        ] {
            fs::create_dir_all(staging.join(dir))
                .map_err(|e| Error::io("create import directory", e))?;
        }
        let retained = staging.join("received/encrypted.tatacoa");
        copy_file(source, &retained)?;
        let retained_manifest = crate::verify_encrypted_bundle(&retained, password)?;
        if retained_manifest != manifest {
            return Err(Error::InvalidManifest(
                "received bundle changed during import".to_owned(),
            ));
        }
        let materialized = crate::encrypted_read::materialize_encrypted_bundle(
            &retained,
            password,
            &staging.join("objects"),
        )?;
        if materialized != manifest {
            return Err(Error::InvalidManifest(
                "materialized bundle changed during import".to_owned(),
            ));
        }
        write_import_records(&staging, &manifest)?;
        if destination.exists() {
            return Err(Error::Conflict(
                "engagement appeared during import".to_owned(),
            ));
        }
        fs::rename(&staging, &destination)
            .map_err(|e| Error::io("commit imported engagement", e))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result.map(|()| manifest.engagement)
}

fn write_import_records(staging: &Path, manifest: &Manifest) -> Result<()> {
    let context = manifest.context.as_ref().ok_or_else(|| {
        Error::InvalidManifest("continuable import requires v2 context".to_owned())
    })?;
    write_json_new_atomic(&staging.join("engagement.json"), &manifest.engagement)?;
    write_json_new_atomic(
        &staging
            .join("manifests")
            .join(format!("{}.json", manifest.execution.id)),
        manifest,
    )?;
    write_json_new_atomic(
        &staging
            .join("context/scopes")
            .join(format!("{}.json", context.scope.id)),
        &context.scope,
    )?;
    write_json_new_atomic(
        &staging
            .join("context/environments")
            .join(format!("{}.json", context.environment.id)),
        &context.environment,
    )?;
    write_json_new_atomic(
        &staging
            .join("context/targets")
            .join(format!("{}.json", context.target.id)),
        &context.target,
    )?;
    write_json_new_atomic(
        &staging
            .join("context/sessions")
            .join(format!("{}.json", context.session.id)),
        &context.session,
    )?;
    for card in &manifest.knowledge_cards {
        write_json_new_atomic(
            &staging.join("knowledge").join(format!("{}.json", card.id)),
            card,
        )?;
    }
    for recipe in &manifest.replay_recipes {
        write_json_new_atomic(
            &staging.join("replay").join(format!("{}.json", recipe.id)),
            recipe,
        )?;
    }
    let continuity = ContinuityState {
        schema_version: CONTINUITY_SCHEMA_VERSION.to_owned(),
        engagement_id: manifest.engagement.id.clone(),
        status: ContinuityStatus::Paused,
        current_session_id: Some(context.session.id.clone()),
        pending: vec!["Revalidar autorización y contexto actual antes de continuar".to_owned()],
        revision: 1,
        updated_unix_ms_observed: unix_ms_observed()?,
    };
    write_json_new_atomic(
        &staging.join("continuity/00000000000000000001-import.json"),
        &continuity,
    )
}

fn verify_plain_bundle_for_import(root: &Path) -> Result<Manifest> {
    reject_symlink(root, "received bundle")?;
    if !root.is_dir() {
        return Err(Error::InvalidPath(
            "Plain bundle must be a directory".to_owned(),
        ));
    }
    let manifest = read_bundle_manifest(root)?;
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION
        || manifest.export_mode != ExportMode::Plain
    {
        return Err(Error::InvalidManifest(
            "continuable Plain import requires alpha v2".to_owned(),
        ));
    }
    let expected: HashSet<_> = manifest
        .artifacts
        .iter()
        .map(|a| format!("{}.bin", a.id))
        .collect();
    let allowed: HashSet<_> = [
        "manifest.json",
        "objects",
        "knowledge",
        "replay",
        "verification",
    ]
    .into_iter()
    .collect();
    for entry in fs::read_dir(root).map_err(|e| Error::io("read bundle root", e))? {
        let entry = entry.map_err(|e| Error::io("read bundle entry", e))?;
        reject_symlink(&entry.path(), "bundle entry")?;
        let name = entry.file_name();
        let name = name
            .to_str()
            .ok_or_else(|| Error::InvalidPath("non-UTF8 bundle entry".to_owned()))?;
        if !allowed.contains(name) {
            return Err(Error::InvalidManifest("unexpected bundle entry".to_owned()));
        }
    }
    for dir in ["objects", "knowledge", "replay", "verification"] {
        let path = root.join(dir);
        reject_symlink(&path, "bundle directory")?;
        if !path.is_dir() {
            return Err(Error::InvalidManifest(
                "required bundle directory missing".to_owned(),
            ));
        }
        if dir != "objects"
            && fs::read_dir(&path)
                .map_err(|e| Error::io("read reserved directory", e))?
                .next()
                .is_some()
        {
            return Err(Error::InvalidManifest(
                "reserved bundle directory is not empty".to_owned(),
            ));
        }
    }
    let mut seen = HashSet::new();
    for entry in
        fs::read_dir(root.join("objects")).map_err(|e| Error::io("read bundle objects", e))?
    {
        let entry = entry.map_err(|e| Error::io("read bundle object", e))?;
        reject_symlink(&entry.path(), "bundle object")?;
        if !entry
            .file_type()
            .map_err(|e| Error::io("inspect bundle object", e))?
            .is_file()
        {
            return Err(Error::InvalidPath(
                "bundle object is not a regular file".to_owned(),
            ));
        }
        let name = entry.file_name();
        let name = name
            .to_str()
            .ok_or_else(|| Error::InvalidPath("non-UTF8 object".to_owned()))?;
        if !expected.contains(name) {
            return Err(Error::InvalidManifest(
                "undeclared bundle object".to_owned(),
            ));
        }
        seen.insert(name.to_owned());
    }
    if seen != expected {
        return Err(Error::InvalidManifest("bundle object missing".to_owned()));
    }
    for artifact in &manifest.artifacts {
        let (size, digest) = compute_sha256(&root.join(&artifact.path))?;
        if size != artifact.size_bytes || !digest.eq_ignore_ascii_case(&artifact.digest.value) {
            return Err(Error::InvalidManifest(
                "bundle artifact size or SHA-256 mismatch".to_owned(),
            ));
        }
    }
    Ok(manifest)
}

fn copy_file(source: &Path, destination: &Path) -> Result<()> {
    reject_symlink(source, "import source")?;
    if !source.is_file() {
        return Err(Error::InvalidPath(
            "import source is not a regular file".to_owned(),
        ));
    }
    let mut reader = fs::File::open(source).map_err(|e| Error::io("open import source", e))?;
    let mut writer = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(destination)
        .map_err(|e| Error::io("create import copy", e))?;
    std::io::copy(&mut reader, &mut writer).map_err(|e| Error::io("copy imported bytes", e))?;
    writer
        .sync_all()
        .map_err(|e| Error::io("sync import copy", e))
}
