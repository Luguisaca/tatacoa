#![forbid(unsafe_code)]

use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use tatacoa_core::{
    ArtifactId, Error, Result, compute_sha256, read_bundle_manifest, safe_join_existing,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactVerification {
    pub artifact_id: ArtifactId,
    pub path: String,
    pub valid: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerificationReport {
    pub valid: bool,
    pub schema_version: String,
    pub artifacts: Vec<ArtifactVerification>,
}

pub fn verify_bundle(bundle_root: &Path) -> Result<VerificationReport> {
    let manifest = read_bundle_manifest(bundle_root)?;
    validate_bundle_layout(
        bundle_root,
        manifest
            .artifacts
            .iter()
            .map(|artifact| artifact.path.as_str()),
    )?;
    let mut artifact_reports = Vec::with_capacity(manifest.artifacts.len());

    for artifact in &manifest.artifacts {
        let path = Path::new(&artifact.path);
        let report = match safe_join_existing(bundle_root, path) {
            Ok(resolved) => {
                let metadata = std::fs::metadata(&resolved)
                    .map_err(|source| Error::io("inspect bundle artifact", source))?;
                if !metadata.is_file() {
                    ArtifactVerification {
                        artifact_id: artifact.id.clone(),
                        path: artifact.path.clone(),
                        valid: false,
                        message: "artifact is not a regular file".to_owned(),
                    }
                } else {
                    let (actual_size, actual_digest) = compute_sha256(&resolved)?;
                    if actual_size != artifact.size_bytes {
                        ArtifactVerification {
                            artifact_id: artifact.id.clone(),
                            path: artifact.path.clone(),
                            valid: false,
                            message: format!(
                                "size mismatch: expected {}, found {actual_size}",
                                artifact.size_bytes
                            ),
                        }
                    } else if !actual_digest.eq_ignore_ascii_case(&artifact.digest.value) {
                        ArtifactVerification {
                            artifact_id: artifact.id.clone(),
                            path: artifact.path.clone(),
                            valid: false,
                            message: "SHA-256 mismatch".to_owned(),
                        }
                    } else {
                        ArtifactVerification {
                            artifact_id: artifact.id.clone(),
                            path: artifact.path.clone(),
                            valid: true,
                            message: "size and SHA-256 match".to_owned(),
                        }
                    }
                }
            }
            Err(error) => ArtifactVerification {
                artifact_id: artifact.id.clone(),
                path: artifact.path.clone(),
                valid: false,
                message: error.to_string(),
            },
        };
        artifact_reports.push(report);
    }

    let valid = artifact_reports.iter().all(|report| report.valid);
    Ok(VerificationReport {
        valid,
        schema_version: manifest.schema_version,
        artifacts: artifact_reports,
    })
}

fn validate_bundle_layout<'a>(
    bundle_root: &Path,
    artifact_paths: impl Iterator<Item = &'a str>,
) -> Result<()> {
    let allowed_root: HashSet<&str> = [
        "manifest.json",
        "objects",
        "knowledge",
        "replay",
        "verification",
    ]
    .into_iter()
    .collect();
    let root_entries =
        fs::read_dir(bundle_root).map_err(|source| Error::io("read bundle root", source))?;
    for entry in root_entries {
        let entry = entry.map_err(|source| Error::io("read bundle root entry", source))?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| Error::InvalidManifest("bundle entry name is not UTF-8".to_owned()))?;
        if !allowed_root.contains(name.as_str()) {
            return Err(Error::InvalidManifest(format!(
                "unexpected bundle root entry: {name}"
            )));
        }
        if entry
            .file_type()
            .map_err(|source| Error::io("inspect bundle root entry", source))?
            .is_symlink()
        {
            return Err(Error::InvalidPath(format!(
                "bundle root entry must not be a symbolic link/reparse point: {name}"
            )));
        }
    }

    let manifest_metadata = fs::symlink_metadata(bundle_root.join("manifest.json"))
        .map_err(|source| Error::io("inspect bundle manifest", source))?;
    if !manifest_metadata.is_file() {
        return Err(Error::InvalidManifest(
            "manifest.json is not a regular file".to_owned(),
        ));
    }
    for directory in ["objects", "knowledge", "replay", "verification"] {
        let metadata = fs::symlink_metadata(bundle_root.join(directory))
            .map_err(|source| Error::io("inspect required bundle directory", source))?;
        if !metadata.is_dir() {
            return Err(Error::InvalidManifest(format!(
                "required bundle entry is not a directory: {directory}"
            )));
        }
    }

    let expected_objects: HashSet<String> = artifact_paths
        .map(|path| path.trim_start_matches("objects/").to_owned())
        .collect();
    for entry in fs::read_dir(bundle_root.join("objects"))
        .map_err(|source| Error::io("read bundle objects directory", source))?
    {
        let entry = entry.map_err(|source| Error::io("read bundle object entry", source))?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| Error::InvalidManifest("object file name is not UTF-8".to_owned()))?;
        let file_type = entry
            .file_type()
            .map_err(|source| Error::io("inspect bundle object entry", source))?;
        if file_type.is_symlink() || !file_type.is_file() {
            return Err(Error::InvalidPath(format!(
                "bundle object is not a regular file: {name}"
            )));
        }
        if !expected_objects.contains(&name) {
            return Err(Error::InvalidManifest(format!(
                "undeclared object in bundle: objects/{name}"
            )));
        }
    }
    for directory in ["knowledge", "replay", "verification"] {
        let mut entries = fs::read_dir(bundle_root.join(directory))
            .map_err(|source| Error::io("read reserved bundle directory", source))?;
        if entries.next().is_some() {
            return Err(Error::InvalidManifest(format!(
                "reserved bundle directory is not empty: {directory}"
            )));
        }
    }
    Ok(())
}
