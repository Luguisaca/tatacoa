#![forbid(unsafe_code)]

mod bundle;
mod capture;
mod error;
mod ids;
mod model;
mod paths;

pub use bundle::{
    create_engagement, export_bundle, load_engagement, load_execution_manifest,
    read_bundle_manifest,
};
pub use capture::{GenericExecutionAdapter, compute_sha256, execute};
pub use error::{Error, Result};
pub use ids::{ArtifactId, EngagementId, ExecutionId};
pub use model::{
    Artifact, ArtifactRole, CaptureStatus, Digest, Engagement, Execution, ExportMode, Invocation,
    Manifest, SecurityProfile,
};
pub use paths::{ensure_safe_relative_path, safe_join_existing};

pub const MANIFEST_SCHEMA_VERSION: &str = "tatacoa.alpha.v1";
