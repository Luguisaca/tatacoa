#![forbid(unsafe_code)]

use serde::Serialize;
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
