#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tatacoa_core::{
    ArtifactId, ArtifactPreview, CaptureStatus, ContinuityInspection, ContinuityState, Engagement,
    EngagementId, ExecutionContext, ExecutionId, ExportMode, KnowledgeCard, KnowledgeCardInput,
    KnowledgeReference, KnowledgeReviewStatus, Manifest, PlainExportAuthorization,
    ReplayPlaceholder, ReplayRecipe, ReplayRecipeInput, Result, SecretPassword, SecurityProfile,
    Session, SessionId, SourceClassification, TimestampObject, TimestampReport, ToolAssistance,
    TsaConfig, TsaTrustPolicy, assist_execution, authorize_encrypted_export,
    authorize_plain_export, create_engagement, create_environment, create_knowledge_card,
    create_replay_recipe, create_scope, create_session, create_target, default_export_mode,
    execute, export_bundle, export_encrypted_bundle, inspect_continuity, list_engagements,
    list_execution_manifests, list_sessions, load_associated_knowledge, load_associated_replay,
    load_execution_context, load_execution_manifest, pause_work, read_artifact_preview,
    request_timestamp, resume_work, verify_timestamp_sidecar, verify_timestamp_sidecar_with_trust,
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

/// Human interpretation added to facts already captured by an Execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KnowledgeFromExecutionRequest {
    pub engagement_id: EngagementId,
    pub execution_id: ExecutionId,
    pub source_reviewed: bool,
    pub why: String,
    pub objective: String,
    pub observe: String,
    pub proves: String,
    pub does_not_prove: String,
    pub validation: String,
    pub defensive_context: String,
    pub references: Vec<KnowledgeReference>,
    pub related_techniques: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoteFromExecutionRequest {
    pub engagement_id: EngagementId,
    pub execution_id: ExecutionId,
    pub note: String,
}

/// Replay preparation that reuses the source invocation unless the operator
/// explicitly supplies a deliberate override.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayFromExecutionRequest {
    pub engagement_id: EngagementId,
    pub execution_id: ExecutionId,
    pub executable_override: Option<String>,
    pub argv_template_override: Option<Vec<String>>,
    pub placeholders: Vec<ReplayPlaceholder>,
    pub prerequisites: Vec<String>,
    pub authorization_limits: Vec<String>,
}

/// Descriptive guidance derived from Core policy. Export still rechecks policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExportGuidance {
    pub default_mode: Option<ExportMode>,
    pub plain_available: bool,
    pub plain_requires_acknowledgement: bool,
    pub encrypted_available: bool,
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
    #[serde(default)]
    pub trust_anchor_der: Vec<PathBuf>,
    #[serde(default)]
    pub trust_intermediate_der: Vec<PathBuf>,
    #[serde(default)]
    pub accepted_policy_oids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimestampVerifyRequest {
    pub bundle: PathBuf,
    pub sidecar: PathBuf,
    pub mode: ExportMode,
    #[serde(default)]
    pub trust_anchor_der: Vec<PathBuf>,
    #[serde(default)]
    pub trust_intermediate_der: Vec<PathBuf>,
    #[serde(default)]
    pub accepted_policy_oids: Vec<String>,
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

    pub fn execution_workspace(
        &self,
        engagement_id: &EngagementId,
        execution_id: &ExecutionId,
    ) -> Result<Manifest> {
        let mut manifest = self.execution(engagement_id, execution_id)?;
        manifest.knowledge_cards =
            load_associated_knowledge(&self.workspace, engagement_id, execution_id)?;
        manifest.replay_recipes =
            load_associated_replay(&self.workspace, engagement_id, execution_id)?;
        Ok(manifest)
    }

    pub fn execution_assistance(
        &self,
        engagement_id: &EngagementId,
        execution_id: &ExecutionId,
    ) -> Result<ToolAssistance> {
        let manifest = self.execution(engagement_id, execution_id)?;
        Ok(assist_execution(&manifest, None, None))
    }

    pub fn export_guidance(
        &self,
        engagement_id: &EngagementId,
        execution_id: &ExecutionId,
    ) -> Result<ExportGuidance> {
        let manifest = self.execution(engagement_id, execution_id)?;
        let profile = manifest.engagement.security_profile;
        let plain_without_ack = authorize_plain_export(
            profile,
            PlainExportAuthorization {
                acknowledged_plaintext: false,
            },
        )
        .is_ok();
        let plain_with_ack = authorize_plain_export(
            profile,
            PlainExportAuthorization {
                acknowledged_plaintext: true,
            },
        )
        .is_ok();
        Ok(ExportGuidance {
            default_mode: default_export_mode(profile).ok(),
            plain_available: plain_with_ack,
            plain_requires_acknowledgement: !plain_without_ack && plain_with_ack,
            encrypted_available: authorize_encrypted_export(profile).is_ok(),
        })
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

    pub fn create_knowledge_from_execution(
        &self,
        request: KnowledgeFromExecutionRequest,
    ) -> Result<KnowledgeCard> {
        let manifest = self.execution(&request.engagement_id, &request.execution_id)?;
        let execution = &manifest.execution;
        let arguments = execution
            .invocation
            .argv
            .iter()
            .enumerate()
            .map(|(index, argument)| format!("argv[{index}]={argument:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        let how = if arguments.is_empty() {
            format!(
                "Captured invocation: executable={:?}, no arguments, shell={}",
                execution.invocation.executable, execution.invocation.shell
            )
        } else {
            format!(
                "Captured invocation: executable={:?}, {arguments}, shell={}",
                execution.invocation.executable, execution.invocation.shell
            )
        };
        self.create_knowledge(KnowledgeRequest {
            engagement_id: request.engagement_id,
            execution_id: request.execution_id,
            source_reviewed: request.source_reviewed,
            what: format!(
                "Execution {} captured by adapter {} with {} artifact(s)",
                execution.id,
                execution.adapter,
                manifest.artifacts.len()
            ),
            why: request.why,
            objective: request.objective,
            how,
            observe: request.observe,
            proves: request.proves,
            does_not_prove: request.does_not_prove,
            errors: format!(
                "Recorded capture status: {:?}; recorded exit code: {:?}",
                execution.capture_status, execution.exit_code
            ),
            validation: request.validation,
            defensive_context: request.defensive_context,
            references: request.references,
            related_techniques: request.related_techniques,
        })
    }

    pub fn create_replay_from_execution(
        &self,
        request: ReplayFromExecutionRequest,
    ) -> Result<ReplayRecipe> {
        let manifest = self.execution(&request.engagement_id, &request.execution_id)?;
        let authorization_limits = if request.authorization_limits.is_empty() {
            let context = manifest.context.as_ref().ok_or_else(|| {
                tatacoa_core::Error::InvalidManifest(
                    "legacy execution has no scope boundary for replay preparation".to_owned(),
                )
            })?;
            vec![context.scope.authorization_boundary.clone()]
        } else {
            request.authorization_limits
        };
        self.create_replay(ReplayRequest {
            engagement_id: request.engagement_id,
            execution_id: request.execution_id,
            executable: request
                .executable_override
                .unwrap_or(manifest.execution.invocation.executable),
            argv_template: request
                .argv_template_override
                .unwrap_or(manifest.execution.invocation.argv),
            placeholders: request.placeholders,
            prerequisites: request.prerequisites,
            authorization_limits,
        })
    }

    pub fn create_note_from_execution(
        &self,
        request: NoteFromExecutionRequest,
    ) -> Result<KnowledgeCard> {
        if request.note.trim().is_empty() {
            return Err(tatacoa_core::Error::InvalidManifest(
                "operator note must not be empty".to_owned(),
            ));
        }
        let execution_id = request.execution_id.clone();
        self.create_knowledge_from_execution(KnowledgeFromExecutionRequest {
            engagement_id: request.engagement_id,
            execution_id: request.execution_id,
            source_reviewed: false,
            why: "Operator note; relevance not assessed".to_owned(),
            objective: "Operator note; objective not specified".to_owned(),
            observe: request.note,
            proves: "Not assessed by operator".to_owned(),
            does_not_prove: "No conclusion or vulnerability was validated".to_owned(),
            validation: "Human validation pending".to_owned(),
            defensive_context: "Not provided by operator".to_owned(),
            references: vec![KnowledgeReference {
                classification: SourceClassification::OperatorNote,
                locator: format!("execution:{execution_id}"),
                title: "Operator note linked to Execution".to_owned(),
            }],
            related_techniques: Vec::new(),
        })
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
        let mut config = TsaConfig::new(
            request.tsa_endpoint,
            Duration::from_secs(request.timeout_seconds),
        )?;
        if let Some(policy) = load_trust_policy(
            &request.trust_anchor_der,
            &request.trust_intermediate_der,
            request.accepted_policy_oids,
        )? {
            config = config.with_trust_policy(policy);
        }
        request_timestamp(
            timestamp_object(request.mode, &request.bundle),
            &request.sidecar,
            &config,
        )
    }

    pub fn verify_timestamp(&self, request: TimestampVerifyRequest) -> Result<TimestampReport> {
        let object = timestamp_object(request.mode, &request.bundle);
        match load_trust_policy(
            &request.trust_anchor_der,
            &request.trust_intermediate_der,
            request.accepted_policy_oids,
        )? {
            Some(policy) => verify_timestamp_sidecar_with_trust(object, &request.sidecar, &policy),
            None => verify_timestamp_sidecar(object, &request.sidecar),
        }
    }
}

fn load_trust_policy(
    anchors: &[PathBuf],
    intermediates: &[PathBuf],
    accepted_policy_oids: Vec<String>,
) -> Result<Option<TsaTrustPolicy>> {
    if anchors.is_empty() && intermediates.is_empty() && accepted_policy_oids.is_empty() {
        return Ok(None);
    }
    let anchors = anchors
        .iter()
        .map(|path| {
            std::fs::read(path)
                .map_err(|source| tatacoa_core::Error::io("read TSA trust anchor DER", source))
        })
        .collect::<Result<Vec<_>>>()?;
    let intermediates = intermediates
        .iter()
        .map(|path| {
            std::fs::read(path)
                .map_err(|source| tatacoa_core::Error::io("read TSA intermediate DER", source))
        })
        .collect::<Result<Vec<_>>>()?;
    TsaTrustPolicy::new(anchors, intermediates, accepted_policy_oids).map(Some)
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
    use tatacoa_core::{
        AssistanceLevel, AssistanceStatus, FactKind, LocalDocumentation,
        LocalDocumentationProvider, SpecializedAdapter,
    };

    struct TestDocumentation;
    impl LocalDocumentationProvider for TestDocumentation {
        fn documentation_for(&self, executable: &str) -> Option<LocalDocumentation> {
            Some(LocalDocumentation {
                executable: executable.to_owned(),
                title: "Local test documentation".to_owned(),
                source_path: Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../docs/specs/ADAPTERS-KNOWLEDGE-REPLAY.md"),
            })
        }
    }

    struct TestAdapter;
    impl SpecializedAdapter for TestAdapter {
        fn matches(&self, _executable: &str) -> bool {
            true
        }
        fn identifier(&self) -> &'static str {
            "test-only-adapter"
        }
    }

    struct MissingDocumentation;
    impl LocalDocumentationProvider for MissingDocumentation {
        fn documentation_for(&self, executable: &str) -> Option<LocalDocumentation> {
            Some(LocalDocumentation {
                executable: executable.to_owned(),
                title: "Unverifiable local file".to_owned(),
                source_path: Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../docs/specs/this-file-does-not-exist.md"),
            })
        }
    }

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
        let guidance = service.export_guidance(&work.engagement.id, &manifest.execution.id)?;
        assert_eq!(guidance.default_mode, Some(ExportMode::Plain));
        assert!(guidance.plain_available);
        assert!(!guidance.plain_requires_acknowledgement);
        assert!(guidance.encrypted_available);
        let artifact = manifest.artifacts[0].clone();
        let generic = service.execution_assistance(&work.engagement.id, &manifest.execution.id)?;
        assert_eq!(generic.level, AssistanceLevel::Generic);
        assert_eq!(generic.documentation, AssistanceStatus::Unavailable);
        assert_eq!(generic.adapter, AssistanceStatus::Unavailable);
        assert!(
            generic
                .facts
                .iter()
                .all(|fact| fact.kind == FactKind::ObservedFact)
        );
        assert!(
            generic
                .facts
                .iter()
                .all(|fact| fact.source.starts_with("manifest."))
        );
        let documented = tatacoa_core::assist_execution(&manifest, Some(&TestDocumentation), None);
        assert_eq!(documented.level, AssistanceLevel::Documented);
        assert!(
            documented
                .facts
                .iter()
                .any(|fact| fact.kind == FactKind::DocumentedFact
                    && fact.source.contains("#sha256="))
        );
        let unavailable =
            tatacoa_core::assist_execution(&manifest, Some(&MissingDocumentation), None);
        assert_eq!(unavailable.level, AssistanceLevel::Generic);
        assert_eq!(unavailable.documentation, AssistanceStatus::Unavailable);
        assert!(
            unavailable
                .facts
                .iter()
                .all(|fact| fact.kind == FactKind::ObservedFact)
        );
        let adapted =
            tatacoa_core::assist_execution(&manifest, Some(&TestDocumentation), Some(&TestAdapter));
        assert_eq!(adapted.level, AssistanceLevel::Adapted);
        assert_eq!(adapted.adapter, AssistanceStatus::Available);
        assert_eq!(
            service.execution(&work.engagement.id, &manifest.execution.id)?,
            manifest
        );
        let preview =
            service.artifact_preview(&work.engagement.id, &manifest.execution.id, &artifact.id)?;
        assert_eq!(preview.artifact.id, artifact.id);

        let card = service.create_knowledge_from_execution(KnowledgeFromExecutionRequest {
            engagement_id: work.engagement.id.clone(),
            execution_id: manifest.execution.id.clone(),
            source_reviewed: false,
            why: "Preserve operator context".to_owned(),
            objective: "Understand the local test run".to_owned(),
            observe: "Process output".to_owned(),
            proves: "The process produced this output".to_owned(),
            does_not_prove: "Any security finding".to_owned(),
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
        assert!(card.what.contains(manifest.execution.id.as_str()));
        assert!(card.how.contains(&format!(
            "executable={:?}",
            manifest.execution.invocation.executable
        )));
        assert!(card.how.contains("argv[0]=\"--list\""));
        assert!(card.errors.contains("Recorded capture status"));

        assert!(
            service
                .create_note_from_execution(NoteFromExecutionRequest {
                    engagement_id: work.engagement.id.clone(),
                    execution_id: manifest.execution.id.clone(),
                    note: "   ".to_owned(),
                })
                .is_err()
        );
        let note = service.create_note_from_execution(NoteFromExecutionRequest {
            engagement_id: work.engagement.id.clone(),
            execution_id: manifest.execution.id.clone(),
            note: "Operator interpretation only".to_owned(),
        })?;
        assert_eq!(note.observe, "Operator interpretation only");
        assert_eq!(note.review_status, KnowledgeReviewStatus::Draft);
        assert_eq!(
            note.references[0].classification,
            SourceClassification::OperatorNote
        );

        let recipe = service.create_replay_from_execution(ReplayFromExecutionRequest {
            engagement_id: work.engagement.id.clone(),
            execution_id: manifest.execution.id.clone(),
            executable_override: None,
            argv_template_override: None,
            placeholders: Vec::new(),
            prerequisites: Vec::new(),
            authorization_limits: Vec::new(),
        })?;
        assert_eq!(recipe.source_execution_id, manifest.execution.id);
        assert_eq!(recipe.executable, manifest.execution.invocation.executable);
        assert_eq!(recipe.argv_template, manifest.execution.invocation.argv);
        assert_eq!(recipe.authorization_limits, vec!["Only this test process"]);
        let other_work = service.create_work(NewWorkRequest {
            engagement_name: "Other isolated engagement".to_owned(),
            security_profile: SecurityProfile::LabLearning,
            scope_name: "Other scope".to_owned(),
            authorization_boundary: "Only the other test process".to_owned(),
            environment_name: "Other host".to_owned(),
            target_label: "Other target".to_owned(),
            target_locator: "other.local".to_owned(),
            session_name: "Other session".to_owned(),
        })?;
        assert!(
            service
                .execution_assistance(&other_work.engagement.id, &manifest.execution.id)
                .is_err()
        );
        assert!(
            service
                .export_guidance(&other_work.engagement.id, &manifest.execution.id)
                .is_err()
        );
        assert!(
            service
                .create_replay_from_execution(ReplayFromExecutionRequest {
                    engagement_id: other_work.engagement.id,
                    execution_id: manifest.execution.id.clone(),
                    executable_override: None,
                    argv_template_override: None,
                    placeholders: Vec::new(),
                    prerequisites: Vec::new(),
                    authorization_limits: Vec::new(),
                })
                .is_err()
        );
        let expected_context = manifest.execution.context.ok_or_else(|| {
            tatacoa_core::Error::InvalidManifest("test execution has no context".to_owned())
        })?;
        assert_eq!(recipe.context, expected_context);

        let execution_workspace =
            service.execution_workspace(&work.engagement.id, &manifest.execution.id)?;
        assert_eq!(execution_workspace.knowledge_cards.len(), 2);
        assert!(execution_workspace.knowledge_cards.contains(&card));
        assert!(execution_workspace.knowledge_cards.contains(&note));
        assert_eq!(execution_workspace.replay_recipes, vec![recipe]);
        assert!(
            execution_workspace
                .artifacts
                .iter()
                .all(|item| item.evidence_state == tatacoa_core::EvidenceState::Captured)
        );

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
                    trust_anchor_der: vec![],
                    trust_intermediate_der: vec![],
                    accepted_policy_oids: vec![],
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
                    trust_anchor_der: vec![],
                    trust_intermediate_der: vec![],
                    accepted_policy_oids: vec![],
                })
                .is_err()
        );
        assert!(!rejected_sidecar.exists());
        let reopened = AppService::open(root);
        let listed = reopened.list_work()?;
        assert_eq!(listed.len(), 2);
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
