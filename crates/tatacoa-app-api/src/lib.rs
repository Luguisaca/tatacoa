#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tatacoa_core::{
    CaptureStatus, Engagement, EngagementId, ExecutionId, Manifest, Result, SecurityProfile,
    Session, SessionId, create_engagement, create_environment, create_scope, create_session,
    create_target, execute, list_engagements, list_execution_manifests, list_sessions,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewWorkRequest {
    pub engagement_name: String,
    pub security_profile: SecurityProfile,
    pub scope_name: String,
    pub authorization_boundary: String,
    pub environment_name: String,
    pub target_label: String,
    pub target_locator: String,
    pub session_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkContext {
    pub engagement: Engagement,
    pub session: Session,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunRequest {
    pub engagement_id: EngagementId,
    pub session_id: SessionId,
    pub executable: String,
    pub argv: Vec<String>,
    pub max_stream_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionSummary {
    pub id: ExecutionId,
    pub session_id: Option<SessionId>,
    pub executable: String,
    pub capture_status: CaptureStatus,
    pub artifact_count: usize,
    pub started_unix_ms_observed: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkSummary {
    pub engagement: Engagement,
    pub sessions: Vec<Session>,
    pub executions: Vec<ExecutionSummary>,
}

pub struct AppService {
    workspace: PathBuf,
}

impl AppService {
    pub fn open(workspace: impl AsRef<Path>) -> Self {
        Self {
            workspace: workspace.as_ref().to_path_buf(),
        }
    }

    pub fn list_work(&self) -> Result<Vec<Engagement>> {
        list_engagements(&self.workspace)
    }

    pub fn create_work(&self, request: NewWorkRequest) -> Result<WorkContext> {
        let engagement = create_engagement(
            &self.workspace,
            request.engagement_name,
            request.security_profile,
        )?;
        let scope = create_scope(
            &self.workspace,
            &engagement.id,
            request.scope_name,
            request.authorization_boundary,
        )?;
        let environment = create_environment(
            &self.workspace,
            &engagement.id,
            &scope.id,
            request.environment_name,
        )?;
        let target = create_target(
            &self.workspace,
            &engagement.id,
            &scope.id,
            &environment.id,
            request.target_label,
            request.target_locator,
        )?;
        let session = create_session(
            &self.workspace,
            &engagement.id,
            &scope.id,
            &environment.id,
            &target.id,
            request.session_name,
        )?;
        Ok(WorkContext {
            engagement,
            session,
        })
    }

    pub fn run(&self, request: RunRequest) -> Result<Manifest> {
        execute(
            &self.workspace,
            &request.engagement_id,
            &request.session_id,
            request.executable,
            request.argv,
            request.max_stream_bytes,
        )
    }

    pub fn summarize(&self, engagement_id: &EngagementId) -> Result<WorkSummary> {
        let engagement = tatacoa_core::load_engagement(&self.workspace, engagement_id)?;
        let sessions = list_sessions(&self.workspace, engagement_id)?;
        let executions = list_execution_manifests(&self.workspace, engagement_id)?
            .into_iter()
            .map(|manifest| ExecutionSummary {
                id: manifest.execution.id,
                session_id: manifest.execution.context.map(|context| context.session_id),
                executable: manifest.execution.invocation.executable,
                capture_status: manifest.execution.capture_status,
                artifact_count: manifest.artifacts.len(),
                started_unix_ms_observed: manifest.execution.started_unix_ms_observed,
            })
            .collect();
        Ok(WorkSummary {
            engagement,
            sessions,
            executions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn user_workflow_creates_runs_reopens_and_summarizes() -> Result<()> {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| tatacoa_core::Error::Execution(error.to_string()))?
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("tatacoa-app-api-{}-{suffix}", std::process::id()));
        let result = exercise_workflow(&root);
        let _ = fs::remove_dir_all(&root);
        result
    }

    fn exercise_workflow(root: &Path) -> Result<()> {
        let service = AppService::open(root);
        let work = service.create_work(NewWorkRequest {
            engagement_name: "Usable Alpha QA".to_owned(),
            security_profile: SecurityProfile::LabLearning,
            scope_name: "Local authorized scope".to_owned(),
            authorization_boundary: "Only this test process".to_owned(),
            environment_name: "Test host".to_owned(),
            target_label: "App API test".to_owned(),
            target_locator: "localhost".to_owned(),
            session_name: "First session".to_owned(),
        })?;
        let executable = std::env::current_exe()
            .map_err(|source| tatacoa_core::Error::io("find test executable", source))?;
        let manifest = service.run(RunRequest {
            engagement_id: work.engagement.id.clone(),
            session_id: work.session.id.clone(),
            executable: executable.to_string_lossy().into_owned(),
            argv: vec!["--list".to_owned()],
            max_stream_bytes: Some(64 * 1024),
        })?;
        let reopened = AppService::open(root);
        let listed = reopened.list_work()?;
        assert_eq!(listed.len(), 1);
        let summary = reopened.summarize(&work.engagement.id)?;
        assert_eq!(summary.sessions, vec![work.session]);
        assert_eq!(summary.executions.len(), 1);
        assert_eq!(summary.executions[0].id, manifest.execution.id);
        assert_eq!(summary.executions[0].artifact_count, 2);
        Ok(())
    }
}
