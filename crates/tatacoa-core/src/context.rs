use crate::{EngagementId, EnvironmentId, Error, Result, ScopeId, SessionId, TargetId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub id: ScopeId,
    pub engagement_id: EngagementId,
    pub name: String,
    pub authorization_boundary: String,
    pub created_unix_ms_observed: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    pub id: EnvironmentId,
    pub engagement_id: EngagementId,
    pub scope_id: ScopeId,
    pub name: String,
    pub created_unix_ms_observed: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub id: TargetId,
    pub engagement_id: EngagementId,
    pub scope_id: ScopeId,
    pub environment_id: EnvironmentId,
    pub label: String,
    pub locator: String,
    pub created_unix_ms_observed: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub id: SessionId,
    pub engagement_id: EngagementId,
    pub scope_id: ScopeId,
    pub environment_id: EnvironmentId,
    pub target_id: TargetId,
    pub name: String,
    pub created_unix_ms_observed: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionContextIds {
    pub scope_id: ScopeId,
    pub environment_id: EnvironmentId,
    pub target_id: TargetId,
    pub session_id: SessionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionContext {
    pub scope: Scope,
    pub environment: Environment,
    pub target: Target,
    pub session: Session,
}

impl ExecutionContext {
    pub fn ids(&self) -> ExecutionContextIds {
        ExecutionContextIds {
            scope_id: self.scope.id.clone(),
            environment_id: self.environment.id.clone(),
            target_id: self.target.id.clone(),
            session_id: self.session.id.clone(),
        }
    }

    pub fn validate(&self, engagement_id: &EngagementId) -> Result<()> {
        if &self.scope.engagement_id != engagement_id
            || &self.environment.engagement_id != engagement_id
            || &self.target.engagement_id != engagement_id
            || &self.session.engagement_id != engagement_id
        {
            return Err(Error::InvalidManifest(
                "execution context crosses engagement boundaries".to_owned(),
            ));
        }
        if self.environment.scope_id != self.scope.id
            || self.target.scope_id != self.scope.id
            || self.session.scope_id != self.scope.id
            || self.target.environment_id != self.environment.id
            || self.session.environment_id != self.environment.id
            || self.session.target_id != self.target.id
        {
            return Err(Error::InvalidManifest(
                "execution context contains inconsistent relationships".to_owned(),
            ));
        }
        Ok(())
    }
}
