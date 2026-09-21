#![forbid(unsafe_code)]

mod bundle;
mod capture;
mod context;
mod continuity;
// Foundation criptográfica interna pendiente de integrar al envelope versionado.
#[allow(dead_code)]
mod crypto;
mod encrypted_export;
#[allow(dead_code)]
mod encrypted_format;
mod encrypted_read;
mod error;
mod ids;
mod knowledge;
mod model;
mod paths;
mod policy;
mod replay;
mod timestamp;
mod timestamp_signature;
mod validation;
mod workspace;

pub use bundle::{
    MAX_ARTIFACT_PREVIEW_BYTES, create_engagement, export_bundle, list_engagements,
    list_execution_manifests, load_engagement, load_execution_manifest, read_artifact_preview,
    read_bundle_manifest,
};
pub use capture::{GenericExecutionAdapter, compute_sha256, execute};
pub use context::{Environment, ExecutionContext, ExecutionContextIds, Scope, Session, Target};
pub use continuity::{
    CONTINUITY_SCHEMA_VERSION, ContinuityInspection, ContinuityState, ContinuityStatus,
    inspect_continuity, pause_work, resume_work,
};
pub use crypto::{MAX_PASSWORD_BYTES, MIN_EXPORT_PASSWORD_CHARACTERS, SecretPassword};
pub use encrypted_export::export_encrypted_bundle;
pub use encrypted_read::verify_encrypted_bundle;
pub use error::{Error, Result};
pub use ids::{
    ArtifactId, EngagementId, EnvironmentId, ExecutionId, KnowledgeId, ReplayId, ScopeId,
    SessionId, TargetId,
};
pub use knowledge::{
    KNOWLEDGE_SCHEMA_VERSION, KnowledgeCard, KnowledgeCardInput, KnowledgeReference,
    KnowledgeReviewStatus, SourceClassification,
};
pub use model::{
    Artifact, ArtifactClassification, ArtifactPreview, ArtifactProvenance, ArtifactRole,
    CaptureStatus, Digest, Engagement, EvidenceState, Execution, ExportMode, Invocation, Manifest,
    ProvenanceKind, SecurityProfile,
};
pub use paths::{ensure_safe_relative_path, safe_join_existing};
pub use policy::{
    PlainExportAuthorization, authorize_encrypted_export, authorize_plain_export,
    default_export_mode,
};
pub use replay::{REPLAY_SCHEMA_VERSION, ReplayPlaceholder, ReplayRecipe, ReplayRecipeInput};
pub use timestamp::{
    MAX_TIMESTAMP_RESPONSE_BYTES, PLAIN_ROOT_VERSION, PlainRootDigest, TimestampAssurance,
    TimestampCheck, TimestampCheckStatus, TimestampObject, TimestampReport, TsaConfig,
    compute_plain_root, request_timestamp, verify_timestamp_sidecar,
};
pub(crate) use validation::{
    validate_artifact_provenance, validate_knowledge_card, validate_replay_recipe,
};
pub use workspace::{
    create_environment, create_knowledge_card, create_replay_recipe, create_scope, create_session,
    create_target, list_sessions, load_execution_context,
};

pub const MANIFEST_SCHEMA_VERSION: &str = "tatacoa.alpha.v2";
pub const LEGACY_MANIFEST_SCHEMA_VERSION: &str = "tatacoa.alpha.v1";
