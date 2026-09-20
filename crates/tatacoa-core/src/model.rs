use crate::{
    ArtifactId, EngagementId, ExecutionContext, ExecutionContextIds, ExecutionId, KnowledgeCard,
    ReplayRecipe,
};
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
    Encrypted,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvidenceState {
    #[default]
    Captured,
    Candidate,
    Reviewed,
    Validated,
    Discarded,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArtifactClassification {
    #[default]
    Raw,
    Derived,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProvenanceKind {
    #[default]
    Capture,
    Import,
    Copy,
    Crop,
    Redaction,
    Annotation,
    Conversion,
    Extraction,
    Merge,
    Other,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactProvenance {
    pub kind: ProvenanceKind,
    #[serde(default)]
    pub source_artifact_ids: Vec<ArtifactId>,
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
    #[serde(default)]
    pub context: Option<ExecutionContextIds>,
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
    pub classification: ArtifactClassification,
    pub role: ArtifactRole,
    pub path: String,
    pub size_bytes: u64,
    pub digest: Digest,
    pub capture_status: CaptureStatus,
    #[serde(default)]
    pub evidence_state: EvidenceState,
    #[serde(default)]
    pub provenance: ArtifactProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: String,
    pub export_mode: ExportMode,
    pub engagement: Engagement,
    #[serde(default)]
    pub context: Option<ExecutionContext>,
    pub execution: Execution,
    pub artifacts: Vec<Artifact>,
    #[serde(default)]
    pub knowledge_cards: Vec<KnowledgeCard>,
    #[serde(default)]
    pub replay_recipes: Vec<ReplayRecipe>,
}
