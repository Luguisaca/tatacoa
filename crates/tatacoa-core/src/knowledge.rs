use crate::{EngagementId, ExecutionId, KnowledgeId};
use serde::{Deserialize, Serialize};

pub const KNOWLEDGE_SCHEMA_VERSION: &str = "tatacoa.knowledge.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SourceClassification {
    UpstreamOfficial,
    Standard,
    Government,
    ProjectDocumentation,
    OperatorNote,
    AiDraft,
    Community,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KnowledgeReviewStatus {
    Draft,
    SourceReviewed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeReference {
    pub classification: SourceClassification,
    pub locator: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeCard {
    pub schema_version: String,
    pub id: KnowledgeId,
    pub engagement_id: EngagementId,
    pub execution_id: ExecutionId,
    pub review_status: KnowledgeReviewStatus,
    pub what: String,
    pub why: String,
    pub objective: String,
    pub how: String,
    pub observe: String,
    pub proves: String,
    pub does_not_prove: String,
    pub errors: String,
    pub validation: String,
    pub defensive_context: String,
    pub references: Vec<KnowledgeReference>,
    pub related_techniques: Vec<String>,
    pub created_unix_ms_observed: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeCardInput {
    pub review_status: KnowledgeReviewStatus,
    pub what: String,
    pub why: String,
    pub objective: String,
    pub how: String,
    pub observe: String,
    pub proves: String,
    pub does_not_prove: String,
    pub errors: String,
    pub validation: String,
    pub defensive_context: String,
    pub references: Vec<KnowledgeReference>,
    pub related_techniques: Vec<String>,
}
