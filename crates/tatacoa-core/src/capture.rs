use crate::bundle::{
    artifact_source_path, engagement_existing_subdirectory, store_execution_manifest,
    unix_ms_observed,
};
use crate::{
    Artifact, ArtifactClassification, ArtifactId, ArtifactProvenance, ArtifactRole, CaptureStatus,
    Digest, EngagementId, Error, EvidenceState, Execution, ExecutionId, ExportMode, Invocation,
    MANIFEST_SCHEMA_VERSION, Manifest, ProvenanceKind, Result, SessionId,
};
use sha2::{Digest as ShaDigest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Instant;

const STREAM_BUFFER_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, Default)]
pub struct GenericExecutionAdapter;

impl GenericExecutionAdapter {
    pub const NAME: &'static str = "generic-execution";

    pub fn supports_current_platform(self) -> bool {
        cfg!(windows) || cfg!(unix)
    }
}

#[derive(Debug)]
struct StreamCapture {
    temporary_path: PathBuf,
    final_path: PathBuf,
    artifact_id: ArtifactId,
    role: ArtifactRole,
    truncated: bool,
}

pub fn execute(
    workspace: &Path,
    engagement_id: &EngagementId,
    session_id: &SessionId,
    executable: String,
    argv: Vec<String>,
    max_stream_bytes: Option<u64>,
) -> Result<Manifest> {
    if executable.trim().is_empty() {
        return Err(Error::Execution("executable must not be empty".to_owned()));
    }
    let engagement = crate::load_engagement(workspace, engagement_id)?;
    let context = crate::load_execution_context(workspace, engagement_id, session_id)?;
    let context_ids = context.ids();
    let execution_id = ExecutionId::new();
    let objects_root =
        engagement_existing_subdirectory(workspace, engagement_id, Path::new("objects"))?;
    let stdout_capture = prepare_capture(&objects_root, ArtifactRole::Stdout)?;
    let stderr_capture = prepare_capture(&objects_root, ArtifactRole::Stderr)?;

    let started_unix_ms_observed = unix_ms_observed()?;
    let monotonic_start = Instant::now();
    let mut child = Command::new(&executable)
        .args(&argv)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| Error::io("spawn executable without a shell", source))?;
    let stdout = child.stdout.take().ok_or_else(|| {
        Error::Execution("stdout pipe was not available after process spawn".to_owned())
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        Error::Execution("stderr pipe was not available after process spawn".to_owned())
    })?;

    let stdout_thread = spawn_stream_capture(stdout, stdout_capture, max_stream_bytes);
    let stderr_thread = spawn_stream_capture(stderr, stderr_capture, max_stream_bytes);
    let status = child
        .wait()
        .map_err(|source| Error::io("wait for executable", source))?;
    let stdout_joined = join_capture(stdout_thread, "stdout");
    let stderr_joined = join_capture(stderr_thread, "stderr");
    let stdout_result = stdout_joined?;
    let stderr_result = stderr_joined?;
    let duration_ms_observed = monotonic_start.elapsed().as_millis();
    let finished_unix_ms_observed = unix_ms_observed()?;

    let any_truncated = stdout_result.truncated || stderr_result.truncated;
    let capture_status = if any_truncated {
        CaptureStatus::Truncated
    } else {
        CaptureStatus::Complete
    };
    let artifacts = vec![
        finalize_capture(workspace, engagement_id, &execution_id, stdout_result)?,
        finalize_capture(workspace, engagement_id, &execution_id, stderr_result)?,
    ];
    let manifest = Manifest {
        schema_version: MANIFEST_SCHEMA_VERSION.to_owned(),
        export_mode: ExportMode::Plain,
        engagement,
        context: Some(context),
        execution: Execution {
            id: execution_id,
            engagement_id: engagement_id.clone(),
            adapter: GenericExecutionAdapter::NAME.to_owned(),
            context: Some(context_ids),
            invocation: Invocation {
                executable,
                argv,
                shell: false,
            },
            started_unix_ms_observed,
            finished_unix_ms_observed,
            duration_ms_observed,
            exit_code: status.code(),
            capture_status,
        },
        artifacts,
        knowledge_cards: Vec::new(),
        replay_recipes: Vec::new(),
    };
    store_execution_manifest(workspace, &manifest)?;
    Ok(manifest)
}

fn prepare_capture(objects_root: &Path, role: ArtifactRole) -> Result<StreamCapture> {
    let artifact_id = ArtifactId::new();
    let final_path = objects_root.join(format!("{artifact_id}.bin"));
    let temporary_path = objects_root.join(format!(".{artifact_id}.partial"));
    Ok(StreamCapture {
        temporary_path,
        final_path,
        artifact_id,
        role,
        truncated: false,
    })
}

fn spawn_stream_capture<R: Read + Send + 'static>(
    reader: R,
    capture: StreamCapture,
    max_stream_bytes: Option<u64>,
) -> thread::JoinHandle<Result<StreamCapture>> {
    thread::spawn(move || capture_stream(reader, capture, max_stream_bytes))
}

fn capture_stream<R: Read>(
    mut reader: R,
    mut capture: StreamCapture,
    max_stream_bytes: Option<u64>,
) -> Result<StreamCapture> {
    let mut writer = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&capture.temporary_path)
        .map_err(|source| Error::io("create RAW capture file", source))?;
    let mut buffer = [0_u8; STREAM_BUFFER_BYTES];
    let mut written = 0_u64;
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|source| Error::io("read process output", source))?;
        if read == 0 {
            break;
        }
        let allowed = max_stream_bytes
            .map(|limit| limit.saturating_sub(written).min(read as u64) as usize)
            .unwrap_or(read);
        if allowed > 0 {
            writer
                .write_all(&buffer[..allowed])
                .map_err(|source| Error::io("write RAW capture", source))?;
            written += allowed as u64;
        }
        if allowed < read {
            capture.truncated = true;
        }
    }
    writer
        .flush()
        .map_err(|source| Error::io("flush RAW capture", source))?;
    writer
        .sync_all()
        .map_err(|source| Error::io("sync RAW capture", source))?;
    drop(writer);
    Ok(capture)
}

fn join_capture(
    handle: thread::JoinHandle<Result<StreamCapture>>,
    stream: &'static str,
) -> Result<StreamCapture> {
    handle
        .join()
        .map_err(|_| Error::Execution(format!("{stream} capture worker stopped unexpectedly")))?
}

fn finalize_capture(
    workspace: &Path,
    engagement_id: &EngagementId,
    execution_id: &ExecutionId,
    capture: StreamCapture,
) -> Result<Artifact> {
    fs::rename(&capture.temporary_path, &capture.final_path)
        .map_err(|source| Error::io("finalize RAW capture", source))?;
    let (size_bytes, digest) = compute_sha256(&capture.final_path)?;
    let artifact = Artifact {
        id: capture.artifact_id,
        engagement_id: engagement_id.clone(),
        execution_id: execution_id.clone(),
        classification: ArtifactClassification::Raw,
        role: capture.role,
        path: String::new(),
        size_bytes,
        digest: Digest {
            algorithm: "SHA-256".to_owned(),
            value: digest,
        },
        capture_status: if capture.truncated {
            CaptureStatus::Truncated
        } else {
            CaptureStatus::Complete
        },
        evidence_state: EvidenceState::Captured,
        provenance: ArtifactProvenance {
            kind: ProvenanceKind::Capture,
            source_artifact_ids: Vec::new(),
        },
    };
    let mut artifact = artifact;
    artifact.path = format!("objects/{}.bin", artifact.id);
    let expected = artifact_source_path(workspace, engagement_id, &artifact)?;
    if expected != capture.final_path {
        return Err(Error::InvalidPath(
            "finalized artifact path did not match isolated engagement".to_owned(),
        ));
    }
    Ok(artifact)
}

pub fn compute_sha256(path: &Path) -> Result<(u64, String)> {
    let mut file =
        File::open(path).map_err(|source| Error::io("open artifact for hashing", source))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; STREAM_BUFFER_BYTES];
    let mut size = 0_u64;
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|source| Error::io("read artifact for hashing", source))?;
        if read == 0 {
            break;
        }
        size = size
            .checked_add(read as u64)
            .ok_or_else(|| Error::Execution("artifact size overflow".to_owned()))?;
        hasher.update(&buffer[..read]);
    }
    let digest = hasher.finalize();
    let mut hexadecimal = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut hexadecimal, "{byte:02x}")
            .map_err(|error| Error::Execution(format!("format SHA-256 digest: {error}")))?;
    }
    Ok((size, hexadecimal))
}
