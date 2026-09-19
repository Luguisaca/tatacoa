use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use tatacoa_core::{
    CaptureStatus, EngagementId, SecurityProfile, create_engagement, execute, export_bundle,
};
use tatacoa_verifier::verify_bundle;

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new(label: &str) -> std::io::Result<Self> {
        let unique = EngagementId::new();
        let path = std::env::temp_dir().join(format!("tatacoa-{label}-{unique}"));
        fs::create_dir(&path)?;
        Ok(Self(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        if self.0.starts_with(std::env::temp_dir())
            && self
                .0
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("tatacoa-"))
        {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

#[test]
fn valid_bundle_passes_and_tampering_fails() -> Result<(), Box<dyn std::error::Error>> {
    let root = TestDirectory::new("e2e")?;
    let workspace = root.path().join("workspace");
    let bundle = root.path().join("bundle");
    let engagement = create_engagement(
        &workspace,
        "authorized alpha test".to_owned(),
        SecurityProfile::LabLearning,
    )?;
    let (executable, argv) = test_command();
    let manifest = execute(&workspace, &engagement.id, executable.clone(), argv, None)?;

    assert_eq!(manifest.execution.engagement_id, engagement.id);
    assert_eq!(manifest.execution.capture_status, CaptureStatus::Complete);
    assert!(!manifest.execution.invocation.shell);
    assert_eq!(manifest.execution.invocation.executable, executable);
    assert_eq!(manifest.artifacts.len(), 2);

    export_bundle(&workspace, &manifest, &bundle)?;
    let initial = verify_bundle(&bundle)?;
    assert!(initial.valid);
    assert!(initial.artifacts.iter().all(|artifact| artifact.valid));

    let artifact_path = bundle.join(&manifest.artifacts[0].path);
    let mut artifact = OpenOptions::new().append(true).open(&artifact_path)?;
    artifact.write_all(b"tampered")?;
    artifact.flush()?;

    let tampered = verify_bundle(&bundle)?;
    assert!(!tampered.valid);
    assert!(tampered.artifacts.iter().any(|artifact| !artifact.valid));
    Ok(())
}

#[test]
fn capture_limit_is_explicitly_truncated() -> Result<(), Box<dyn std::error::Error>> {
    let root = TestDirectory::new("truncate")?;
    let workspace = root.path().join("workspace");
    let engagement = create_engagement(
        &workspace,
        "truncation test".to_owned(),
        SecurityProfile::LabLearning,
    )?;
    let (executable, argv) = test_command();
    let manifest = execute(&workspace, &engagement.id, executable, argv, Some(2))?;

    assert_eq!(manifest.execution.capture_status, CaptureStatus::Truncated);
    assert!(
        manifest
            .artifacts
            .iter()
            .all(|artifact| artifact.capture_status == CaptureStatus::Truncated)
    );
    assert!(
        manifest
            .artifacts
            .iter()
            .all(|artifact| artifact.size_bytes <= 2)
    );
    Ok(())
}

#[test]
fn cross_engagement_manifest_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let root = TestDirectory::new("isolation")?;
    let workspace = root.path().join("workspace");
    let first = create_engagement(
        &workspace,
        "first engagement".to_owned(),
        SecurityProfile::LabLearning,
    )?;
    let second = create_engagement(
        &workspace,
        "second engagement".to_owned(),
        SecurityProfile::LabLearning,
    )?;
    let (executable, argv) = test_command();
    let mut manifest = execute(&workspace, &first.id, executable, argv, None)?;
    manifest.engagement = second;

    let result = export_bundle(&workspace, &manifest, &root.path().join("invalid-bundle"));
    assert!(result.is_err());
    Ok(())
}

#[test]
fn missing_artifact_is_reported_invalid() -> Result<(), Box<dyn std::error::Error>> {
    let root = TestDirectory::new("missing")?;
    let workspace = root.path().join("workspace");
    let bundle = root.path().join("bundle");
    let engagement = create_engagement(
        &workspace,
        "missing artifact test".to_owned(),
        SecurityProfile::LabLearning,
    )?;
    let (executable, argv) = test_command();
    let manifest = execute(&workspace, &engagement.id, executable, argv, None)?;
    export_bundle(&workspace, &manifest, &bundle)?;
    fs::remove_file(bundle.join(&manifest.artifacts[0].path))?;

    let report = verify_bundle(&bundle)?;
    assert!(!report.valid);
    Ok(())
}

#[cfg(windows)]
fn test_command() -> (String, Vec<String>) {
    (
        "cmd.exe".to_owned(),
        vec![
            "/D".to_owned(),
            "/S".to_owned(),
            "/C".to_owned(),
            "<nul set /p =alpha & <nul set /p =beta 1>&2".to_owned(),
        ],
    )
}

#[cfg(unix)]
fn test_command() -> (String, Vec<String>) {
    (
        "/bin/sh".to_owned(),
        vec!["-c".to_owned(), "printf alpha; printf beta >&2".to_owned()],
    )
}
