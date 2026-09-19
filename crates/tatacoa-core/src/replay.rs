use crate::{EngagementId, ExecutionContextIds, ExecutionId, ReplayId};
use serde::{Deserialize, Serialize};

pub const REPLAY_SCHEMA_VERSION: &str = "tatacoa.replay.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayPlaceholder {
    pub name: String,
    pub description: String,
    pub secret: bool,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayRecipe {
    pub schema_version: String,
    pub id: ReplayId,
    pub engagement_id: EngagementId,
    pub source_execution_id: ExecutionId,
    pub context: ExecutionContextIds,
    pub executable: String,
    pub argv_template: Vec<String>,
    pub placeholders: Vec<ReplayPlaceholder>,
    pub prerequisites: Vec<String>,
    pub authorization_limits: Vec<String>,
    pub created_unix_ms_observed: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayRecipeInput {
    pub executable: String,
    pub argv_template: Vec<String>,
    pub placeholders: Vec<ReplayPlaceholder>,
    pub prerequisites: Vec<String>,
    pub authorization_limits: Vec<String>,
}
