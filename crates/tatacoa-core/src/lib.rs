#![forbid(unsafe_code)]

mod bundle;
mod capture;
mod context;
// Foundation criptográfica interna pendiente de integrar al envelope versionado.
#[allow(dead_code)]
mod crypto;
mod error;
mod ids;
mod knowledge;
mod model;
mod paths;
mod policy;
mod replay;
mod validation;
mod workspace;

pub use bundle::{
    create_engagement, export_bundle, load_engagement, load_execution_manifest,
    read_bundle_manifest,
};
pub use capture::{GenericExecutionAdapter, compute_sha256, execute};
pub use context::{Environment, ExecutionContext, ExecutionContextIds, Scope, Session, Target};
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
    Artifact, ArtifactClassification, ArtifactProvenance, ArtifactRole, CaptureStatus, Digest,
    Engagement, EvidenceState, Execution, ExportMode, Invocation, Manifest, ProvenanceKind,
    SecurityProfile,
};
pub use paths::{ensure_safe_relative_path, safe_join_existing};
pub use policy::{PlainExportAuthorization, authorize_plain_export};
pub use replay::{REPLAY_SCHEMA_VERSION, ReplayPlaceholder, ReplayRecipe, ReplayRecipeInput};
pub(crate) use validation::{
    validate_artifact_provenance, validate_knowledge_card, validate_replay_recipe,
};
pub use workspace::{
    create_environment, create_knowledge_card, create_replay_recipe, create_scope, create_session,
    create_target, load_execution_context,
};

pub const MANIFEST_SCHEMA_VERSION: &str = "tatacoa.alpha.v2";
pub const LEGACY_MANIFEST_SCHEMA_VERSION: &str = "tatacoa.alpha.v1";
