#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tatacoa_core::{
    ArtifactId, ArtifactPreview, CaptureStatus, ContinuityInspection, ContinuityState, Engagement,
    EngagementId, ExecutionContext, ExecutionId, ExportMode, KnowledgeCard, KnowledgeCardInput,
    KnowledgeReference, KnowledgeReviewStatus, Manifest, PlainExportAuthorization,
    ReplayPlaceholder, ReplayRecipe, ReplayRecipeInput, Result, SecretPassword, SecurityProfile,
    Session, SessionId, TimestampObject, TimestampReport, TsaConfig, create_engagement,
    create_environment, create_knowledge_card, create_replay_recipe, create_scope, create_session,
    create_target, execute, export_bundle, export_encrypted_bundle, inspect_continuity,
    list_engagements, list_execution_manifests, list_sessions, load_execution_context,
    load_execution_manifest, pause_work, read_artifact_preview, request_timestamp, resume_work,
    verify_timestamp_sidecar,
};
use zeroize::Zeroize;

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
pub struct AuthorizationReview {
    pub engagement: Engagement,
    pub context: ExecutionContext,
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
    pub continuity: ContinuityInspection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeRequest {
    pub engagement_id: EngagementId,
    pub execution_id: ExecutionId,
    pub source_reviewed: bool,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayRequest {
    pub engagement_id: EngagementId,
    pub execution_id: ExecutionId,
    pub executable: String,
    pub argv_template: Vec<String>,
    pub placeholders: Vec<ReplayPlaceholder>,
    pub prerequisites: Vec<String>,
    pub authorization_limits: Vec<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportRequest {
    pub engagement_id: EngagementId,
    pub execution_id: ExecutionId,
    pub destination: PathBuf,
    pub mode: ExportMode,
    pub acknowledge_plaintext: bool,
    pub password: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimestampRequest {
    pub bundle: PathBuf,
    pub sidecar: PathBuf,
    pub mode: ExportMode,
    pub tsa_endpoint: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimestampVerifyRequest {
    pub bundle: PathBuf,
    pub sidecar: PathBuf,
    pub mode: ExportMode,
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

    pub fn authorization_review(
        &self,
        engagement_id: &EngagementId,
        session_id: &SessionId,
    ) -> Result<AuthorizationReview> {
        Ok(AuthorizationReview {
            engagement: tatacoa_core::load_engagement(&self.workspace, engagement_id)?,
            context: load_execution_context(&self.workspace, engagement_id, session_id)?,
        })
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
            continuity: inspect_continuity(&self.workspace, engagement_id)?,
        })
    }

    pub fn pause(
        &self,
        engagement_id: &EngagementId,
        session_id: Option<SessionId>,
        pending: Vec<String>,
    ) -> Result<ContinuityState> {
        pause_work(&self.workspace, engagement_id, session_id, pending)
    }

    pub fn resume(
        &self,
        engagement_id: &EngagementId,
        authorization_revalidated: bool,
    ) -> Result<ContinuityState> {
        resume_work(&self.workspace, engagement_id, authorization_revalidated)
    }

    pub fn execution(
        &self,
        engagement_id: &EngagementId,
        execution_id: &ExecutionId,
    ) -> Result<Manifest> {
        load_execution_manifest(&self.workspace, engagement_id, execution_id)
    }

    pub fn artifact_preview(
        &self,
        engagement_id: &EngagementId,
        execution_id: &ExecutionId,
        artifact_id: &ArtifactId,
    ) -> Result<ArtifactPreview> {
        read_artifact_preview(&self.workspace, engagement_id, execution_id, artifact_id)
    }

    pub fn create_knowledge(&self, request: KnowledgeRequest) -> Result<KnowledgeCard> {
        create_knowledge_card(
            &self.workspace,
            &request.engagement_id,
            &request.execution_id,
            KnowledgeCardInput {
                review_status: if request.source_reviewed {
                    KnowledgeReviewStatus::SourceReviewed
                } else {
                    KnowledgeReviewStatus::Draft
                },
                what: request.what,
                why: request.why,
                objective: request.objective,
                how: request.how,
                observe: request.observe,
                proves: request.proves,
                does_not_prove: request.does_not_prove,
                errors: request.errors,
                validation: request.validation,
                defensive_context: request.defensive_context,
                references: request.references,
                related_techniques: request.related_techniques,
            },
        )
    }

    pub fn create_replay(&self, request: ReplayRequest) -> Result<ReplayRecipe> {
        create_replay_recipe(
            &self.workspace,
            &request.engagement_id,
            &request.execution_id,
            ReplayRecipeInput {
                executable: request.executable,
                argv_template: request.argv_template,
                placeholders: request.placeholders,
                prerequisites: request.prerequisites,
                authorization_limits: request.authorization_limits,
            },
        )
    }

    pub fn export(&self, request: ExportRequest) -> Result<()> {
        let manifest = load_execution_manifest(
            &self.workspace,
            &request.engagement_id,
            &request.execution_id,
        )?;
        match request.mode {
            ExportMode::Plain => {
                if let Some(mut password) = request.password {
                    password.zeroize();
                    return Err(tatacoa_core::Error::InvalidManifest(
                        "PLAIN export must not receive a password".to_owned(),
                    ));
                }
                export_bundle(
                    &self.workspace,
                    &manifest,
                    &request.destination,
                    PlainExportAuthorization {
                        acknowledged_plaintext: request.acknowledge_plaintext,
                    },
                )
            }
            ExportMode::Encrypted => {
                let password = request.password.ok_or_else(|| {
                    tatacoa_core::Error::InvalidManifest(
                        "ENCRYPTED export requires a password".to_owned(),
                    )
                })?;
                let secret = SecretPassword::for_export_profile(
                    manifest.engagement.security_profile,
                    password,
                )?;
                export_encrypted_bundle(&self.workspace, &manifest, &request.destination, &secret)
            }
        }
    }

    pub fn request_timestamp(&self, request: TimestampRequest) -> Result<TimestampReport> {
        let config = TsaConfig::new(
            request.tsa_endpoint,
            Duration::from_secs(request.timeout_seconds),
        )?;
        request_timestamp(
            timestamp_object(request.mode, &request.bundle),
            &request.sidecar,
            &config,
        )
    }

    pub fn verify_timestamp(&self, request: TimestampVerifyRequest) -> Result<TimestampReport> {
        verify_timestamp_sidecar(
            timestamp_object(request.mode, &request.bundle),
            &request.sidecar,
        )
    }
}

fn timestamp_object(mode: ExportMode, bundle: &Path) -> TimestampObject<'_> {
    match mode {
        ExportMode::Plain => TimestampObject::PlainBundle(bundle),
        ExportMode::Encrypted => TimestampObject::EncryptedBundle(bundle),
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
        let review = service.authorization_review(&work.engagement.id, &work.session.id)?;
        assert_eq!(
            review.engagement.security_profile,
            SecurityProfile::LabLearning
        );
        assert_eq!(
            review.context.scope.authorization_boundary,
            "Only this test process"
        );
        assert_eq!(review.context.environment.name, "Test host");
        assert_eq!(review.context.target.label, "App API test");
        assert_eq!(review.context.session, work.session);
        let manifest = service.run(RunRequest {
            engagement_id: work.engagement.id.clone(),
            session_id: work.session.id.clone(),
            executable: executable.to_string_lossy().into_owned(),
            argv: vec!["--list".to_owned()],
            max_stream_bytes: Some(64 * 1024),
        })?;
        let artifact = manifest.artifacts[0].clone();
        let preview =
            service.artifact_preview(&work.engagement.id, &manifest.execution.id, &artifact.id)?;
        assert_eq!(preview.artifact.id, artifact.id);

        let card = service.create_knowledge(KnowledgeRequest {
            engagement_id: work.engagement.id.clone(),
            execution_id: manifest.execution.id.clone(),
            source_reviewed: false,
            what: "Captured test output".to_owned(),
            why: "Preserve operator context".to_owned(),
            objective: "Understand the local test run".to_owned(),
            how: "Review the captured output".to_owned(),
            observe: "Process output".to_owned(),
            proves: "The process produced this output".to_owned(),
            does_not_prove: "Any security finding".to_owned(),
            errors: "Output may be incomplete".to_owned(),
            validation: "Manual review remains required".to_owned(),
            defensive_context: "Use only in this test workspace".to_owned(),
            references: vec![KnowledgeReference {
                classification: tatacoa_core::SourceClassification::OperatorNote,
                locator: "local:test".to_owned(),
                title: "Operator test note".to_owned(),
            }],
            related_techniques: Vec::new(),
        })?;
        assert_eq!(card.execution_id, manifest.execution.id);

        let recipe = service.create_replay(ReplayRequest {
            engagement_id: work.engagement.id.clone(),
            execution_id: manifest.execution.id.clone(),
            executable: executable.to_string_lossy().into_owned(),
            argv_template: vec!["--list".to_owned()],
            placeholders: Vec::new(),
            prerequisites: vec!["Local test binary".to_owned()],
            authorization_limits: vec!["Only this test workspace".to_owned()],
        })?;
        assert_eq!(recipe.source_execution_id, manifest.execution.id);

        let bundle = root.join("plain-bundle");
        service.export(ExportRequest {
            engagement_id: work.engagement.id.clone(),
            execution_id: manifest.execution.id.clone(),
            destination: bundle.clone(),
            mode: ExportMode::Plain,
            acknowledge_plaintext: false,
            password: None,
        })?;
        assert!(bundle.join("manifest.json").is_file());
        let first_root = tatacoa_core::compute_plain_root(&bundle)?;
        let second_root = tatacoa_core::compute_plain_root(&bundle)?;
        assert_eq!(first_root, second_root);
        assert_eq!(first_root.entry_count, 3);
        assert_eq!(first_root.message_imprint()?.len(), 32);
        let malformed_sidecar = root.join("malformed.tsr");
        fs::write(&malformed_sidecar, b"not DER")
            .map_err(|source| tatacoa_core::Error::io("write malformed sidecar", source))?;
        assert!(
            service
                .verify_timestamp(TimestampVerifyRequest {
                    bundle: bundle.clone(),
                    sidecar: malformed_sidecar,
                    mode: ExportMode::Plain,
                })
                .is_err()
        );
        let rejected_sidecar = root.join("rejected.tsr");
        assert!(
            service
                .request_timestamp(TimestampRequest {
                    bundle: bundle.clone(),
                    sidecar: rejected_sidecar.clone(),
                    mode: ExportMode::Plain,
                    tsa_endpoint: "http://tsa.invalid".to_owned(),
                    timeout_seconds: 30,
                })
                .is_err()
        );
        assert!(!rejected_sidecar.exists());
        let reopened = AppService::open(root);
        let listed = reopened.list_work()?;
        assert_eq!(listed.len(), 1);
        let summary = reopened.summarize(&work.engagement.id)?;
        assert_eq!(summary.sessions, vec![work.session.clone()]);
        assert_eq!(summary.executions.len(), 1);
        assert_eq!(summary.executions[0].id, manifest.execution.id);
        assert_eq!(summary.executions[0].artifact_count, 2);
        let paused = service.pause(
            &work.engagement.id,
            Some(work.session.id.clone()),
            vec!["Review captured output".to_owned()],
        )?;
        assert_eq!(paused.status, tatacoa_core::ContinuityStatus::Paused);
        assert!(service.resume(&work.engagement.id, false).is_err());
        let resumed = service.resume(&work.engagement.id, true)?;
        assert_eq!(resumed.status, tatacoa_core::ContinuityStatus::Active);

        fs::write(
            root.join("engagements")
                .join(work.engagement.id.as_str())
                .join("objects")
                .join(format!("{}.bin", artifact.id)),
            b"tampered",
        )
        .map_err(|source| tatacoa_core::Error::io("tamper test artifact", source))?;
        assert!(
            service
                .artifact_preview(&work.engagement.id, &manifest.execution.id, &artifact.id)
                .is_err()
        );
        fs::write(
            bundle.join("objects").join(format!("{}.bin", artifact.id)),
            b"tampered",
        )
        .map_err(|source| tatacoa_core::Error::io("tamper exported artifact", source))?;
        assert!(tatacoa_core::compute_plain_root(&bundle).is_err());
        Ok(())
    }
}
