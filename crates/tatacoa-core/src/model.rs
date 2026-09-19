use crate::{ArtifactId, EngagementId, ExecutionId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SecurityProfile {
    LabLearning,
    Professional,
    HighSensitivity,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CaptureStatus {
    Complete,
    Partial,
    Truncated,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArtifactRole {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExportMode {
    Plain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Engagement {
    pub id: EngagementId,
    pub name: String,
    pub created_unix_ms_observed: u128,
    pub security_profile: SecurityProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invocation {
    pub executable: String,
    pub argv: Vec<String>,
    pub shell: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Execution {
    pub id: ExecutionId,
    pub engagement_id: EngagementId,
    pub adapter: String,
    pub invocation: Invocation,
    pub started_unix_ms_observed: u128,
    pub finished_unix_ms_observed: u128,
    pub duration_ms_observed: u128,
    pub exit_code: Option<i32>,
    pub capture_status: CaptureStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Digest {
    pub algorithm: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub id: ArtifactId,
    pub engagement_id: EngagementId,
    pub execution_id: ExecutionId,
    pub classification: String,
    pub role: ArtifactRole,
    pub path: String,
    pub size_bytes: u64,
    pub digest: Digest,
    pub capture_status: CaptureStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: String,
    pub export_mode: ExportMode,
    pub engagement: Engagement,
    pub execution: Execution,
    pub artifacts: Vec<Artifact>,
}
