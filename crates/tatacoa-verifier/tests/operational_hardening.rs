use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::SystemTime;
use tatacoa_core::{
    CaptureStatus, Engagement, EngagementId, PlainExportAuthorization, SecurityProfile, Session,
    create_engagement, create_environment, create_scope, create_session, create_target, execute,
    export_bundle,
};
use tatacoa_verifier::verify_bundle;

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new(label: &str) -> io::Result<Self> {
        let path =
            std::env::temp_dir().join(format!("tatacoa-hardening-{label}-{}", EngagementId::new()));
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
                .is_some_and(|name| name.to_string_lossy().starts_with("tatacoa-hardening-"))
        {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

#[test]
fn large_concurrent_streams_are_drained_and_truncated() -> Result<(), Box<dyn std::error::Error>> {
    let root = TestDirectory::new("large-output")?;
    let workspace = root.path().join("workspace");
    let bundle = root.path().join("bundle");
    let (engagement, session) = create_test_context(&workspace)?;
    let (executable, argv) = fixture_command("fixture_emits_large_streams")?;
    let manifest = execute(
        &workspace,
        &engagement.id,
        &session.id,
        executable,
        argv,
        Some(64 * 1024),
    )?;
    assert_eq!(manifest.execution.capture_status, CaptureStatus::Truncated);
    assert!(
        manifest
            .artifacts
            .iter()
            .all(|artifact| artifact.size_bytes <= 64 * 1024)
    );
    export_bundle(
        &workspace,
        &manifest,
        &bundle,
        PlainExportAuthorization::default(),
    )?;
    assert!(verify_bundle(&bundle)?.valid);
    Ok(())
}

#[test]
fn parallel_executions_remain_isolated_and_verifiable() -> Result<(), Box<dyn std::error::Error>> {
    let root = TestDirectory::new("parallel")?;
    let workspace = root.path().join("workspace");
    let (engagement, session) = create_test_context(&workspace)?;
    let (fixture_executable, fixture_argv) = fixture_command("fixture_emits_small_streams")?;
    let mut handles = Vec::new();
    for _ in 0..6 {
        let workspace = workspace.clone();
        let engagement_id = engagement.id.clone();
        let session_id = session.id.clone();
        let executable = fixture_executable.clone();
        let argv = fixture_argv.clone();
        handles.push(thread::spawn(move || {
            execute(
                &workspace,
                &engagement_id,
                &session_id,
                executable,
                argv,
                None,
            )
        }));
    }

    let mut manifests = Vec::new();
    for handle in handles {
        let manifest = handle
            .join()
            .map_err(|_| io::Error::other("parallel execution worker panicked"))??;
        manifests.push(manifest);
    }
    let execution_ids: HashSet<_> = manifests
        .iter()
        .map(|manifest| manifest.execution.id.as_str())
        .collect();
    let artifact_ids: HashSet<_> = manifests
        .iter()
        .flat_map(|manifest| {
            manifest
                .artifacts
                .iter()
                .map(|artifact| artifact.id.as_str())
        })
        .collect();
    assert_eq!(execution_ids.len(), manifests.len());
    assert_eq!(artifact_ids.len(), manifests.len() * 2);

    for (index, manifest) in manifests.iter().enumerate() {
        let bundle = root.path().join(format!("bundle-{index}"));
        export_bundle(
            &workspace,
            manifest,
            &bundle,
            PlainExportAuthorization::default(),
        )?;
        assert!(verify_bundle(&bundle)?.valid);
    }
    Ok(())
}

#[test]
fn invalid_recorder_storage_fails_without_manifest() -> Result<(), Box<dyn std::error::Error>> {
    let root = TestDirectory::new("recorder-failure")?;
    let workspace = root.path().join("workspace");
    let (engagement, session) = create_test_context(&workspace)?;
    let engagement_root = workspace.join("engagements").join(engagement.id.as_str());
    let objects = engagement_root.join("objects");
    fs::remove_dir(&objects)?;
    fs::write(&objects, b"hostile replacement")?;
    let (executable, argv) = fixture_command("fixture_emits_small_streams")?;

    assert!(
        execute(
            &workspace,
            &engagement.id,
            &session.id,
            executable,
            argv,
            None,
        )
        .is_err()
    );
    assert_eq!(fs::read_dir(engagement_root.join("manifests"))?.count(), 0);
    Ok(())
}

#[test]
fn verifier_rejects_unknown_adapter_and_undeclared_objects()
-> Result<(), Box<dyn std::error::Error>> {
    let root = TestDirectory::new("hostile-bundle")?;
    let workspace = root.path().join("workspace");
    let first_bundle = root.path().join("unknown-adapter");
    let second_bundle = root.path().join("extra-object");
    let third_bundle = root.path().join("reserved-content");
    let (engagement, session) = create_test_context(&workspace)?;
    let (executable, argv) = fixture_command("fixture_emits_small_streams")?;
    let manifest = execute(
        &workspace,
        &engagement.id,
        &session.id,
        executable,
        argv,
        None,
    )?;
    export_bundle(
        &workspace,
        &manifest,
        &first_bundle,
        PlainExportAuthorization::default(),
    )?;
    export_bundle(
        &workspace,
        &manifest,
        &second_bundle,
        PlainExportAuthorization::default(),
    )?;
    export_bundle(
        &workspace,
        &manifest,
        &third_bundle,
        PlainExportAuthorization::default(),
    )?;

    let manifest_path = first_bundle.join("manifest.json");
    let mut json: serde_json::Value = serde_json::from_slice(&fs::read(&manifest_path)?)?;
    json["execution"]["adapter"] = serde_json::Value::String("unknown-adapter".to_owned());
    fs::write(&manifest_path, serde_json::to_vec_pretty(&json)?)?;
    assert!(verify_bundle(&first_bundle).is_err());

    fs::write(
        second_bundle.join("objects").join("undeclared.bin"),
        b"undeclared",
    )?;
    assert!(verify_bundle(&second_bundle).is_err());

    fs::write(
        third_bundle.join("verification").join("untrusted.json"),
        b"{}",
    )?;
    assert!(verify_bundle(&third_bundle).is_err());
    Ok(())
}

#[cfg(unix)]
#[test]
fn workspace_object_symlink_is_rejected_before_capture() -> Result<(), Box<dyn std::error::Error>> {
    let root = TestDirectory::new("workspace-symlink")?;
    let workspace = root.path().join("workspace");
    let outside = root.path().join("outside");
    fs::create_dir(&outside)?;
    let (engagement, session) = create_test_context(&workspace)?;
    let objects = workspace
        .join("engagements")
        .join(engagement.id.as_str())
        .join("objects");
    fs::remove_dir(&objects)?;
    std::os::unix::fs::symlink(&outside, &objects)?;
    let (executable, argv) = fixture_command("fixture_emits_small_streams")?;

    assert!(
        execute(
            &workspace,
            &engagement.id,
            &session.id,
            executable,
            argv,
            None,
        )
        .is_err()
    );
    assert_eq!(fs::read_dir(outside)?.count(), 0);
    Ok(())
}

#[test]
fn verifier_does_not_modify_bundle() -> Result<(), Box<dyn std::error::Error>> {
    let root = TestDirectory::new("read-only")?;
    let workspace = root.path().join("workspace");
    let bundle = root.path().join("bundle");
    let (engagement, session) = create_test_context(&workspace)?;
    let (executable, argv) = fixture_command("fixture_emits_small_streams")?;
    let manifest = execute(
        &workspace,
        &engagement.id,
        &session.id,
        executable,
        argv,
        None,
    )?;
    export_bundle(
        &workspace,
        &manifest,
        &bundle,
        PlainExportAuthorization::default(),
    )?;
    let before = snapshot_tree(&bundle)?;
    assert!(verify_bundle(&bundle)?.valid);
    let after = snapshot_tree(&bundle)?;
    assert_eq!(before, after);
    Ok(())
}

#[test]
#[ignore = "helper process invoked by capture tests"]
fn fixture_emits_large_streams() -> io::Result<()> {
    let stdout = io::stdout();
    let stderr = io::stderr();
    let mut stdout = stdout.lock();
    let mut stderr = stderr.lock();
    let stdout_chunk = [b'x'; 1024];
    let stderr_chunk = [b'y'; 1024];
    for _ in 0..2048 {
        stdout.write_all(&stdout_chunk)?;
        stderr.write_all(&stderr_chunk)?;
    }
    stdout.flush()?;
    stderr.flush()?;
    Ok(())
}

#[test]
#[ignore = "helper process invoked by capture tests"]
fn fixture_emits_small_streams() -> io::Result<()> {
    io::stdout().write_all(b"fixture-stdout")?;
    io::stderr().write_all(b"fixture-stderr")?;
    Ok(())
}

type Snapshot = BTreeMap<String, (bool, u64, SystemTime, Option<String>)>;

fn snapshot_tree(root: &Path) -> Result<Snapshot, Box<dyn std::error::Error>> {
    let mut snapshot = BTreeMap::new();
    snapshot_directory(root, root, &mut snapshot)?;
    Ok(snapshot)
}

fn snapshot_directory(
    root: &Path,
    directory: &Path,
    snapshot: &mut Snapshot,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut entries = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        let relative = path
            .strip_prefix(root)?
            .to_string_lossy()
            .replace('\\', "/");
        let digest = if metadata.is_file() {
            Some(tatacoa_core::compute_sha256(&path)?.1)
        } else {
            None
        };
        snapshot.insert(
            relative,
            (
                metadata.is_dir(),
                metadata.len(),
                metadata.modified()?,
                digest,
            ),
        );
        if metadata.is_dir() {
            snapshot_directory(root, &path, snapshot)?;
        }
    }
    Ok(())
}

fn create_test_context(
    workspace: &Path,
) -> Result<(Engagement, Session), Box<dyn std::error::Error>> {
    let engagement = create_engagement(
        workspace,
        "Operational hardening".to_owned(),
        SecurityProfile::LabLearning,
    )?;
    let scope = create_scope(
        workspace,
        &engagement.id,
        "authorized scope".to_owned(),
        "test process only".to_owned(),
    )?;
    let environment = create_environment(
        workspace,
        &engagement.id,
        &scope.id,
        "test environment".to_owned(),
    )?;
    let target = create_target(
        workspace,
        &engagement.id,
        &scope.id,
        &environment.id,
        "local target".to_owned(),
        "localhost".to_owned(),
    )?;
    let session = create_session(
        workspace,
        &engagement.id,
        &scope.id,
        &environment.id,
        &target.id,
        "test session".to_owned(),
    )?;
    Ok((engagement, session))
}

fn fixture_command(test_name: &str) -> io::Result<(String, Vec<String>)> {
    Ok((
        std::env::current_exe()?.to_string_lossy().into_owned(),
        vec![
            "--exact".to_owned(),
            test_name.to_owned(),
            "--ignored".to_owned(),
            "--nocapture".to_owned(),
        ],
    ))
}
