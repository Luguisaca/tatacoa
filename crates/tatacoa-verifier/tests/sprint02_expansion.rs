use std::fs;
use std::path::{Path, PathBuf};
use tatacoa_core::{
    ArtifactClassification, Engagement, EngagementId, EvidenceState, KnowledgeCardInput,
    KnowledgeReference, KnowledgeReviewStatus, PlainExportAuthorization, ProvenanceKind,
    ReplayPlaceholder, ReplayRecipeInput, SecurityProfile, Session, SourceClassification,
    create_engagement, create_environment, create_knowledge_card, create_replay_recipe,
    create_scope, create_session, create_target, execute, export_bundle, read_bundle_manifest,
};
use tatacoa_verifier::verify_bundle;

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new(label: &str) -> std::io::Result<Self> {
        let path =
            std::env::temp_dir().join(format!("tatacoa-sprint02-{label}-{}", EngagementId::new()));
        fs::create_dir(&path)?;
        Ok(Self(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        if self.0.starts_with(std::env::temp_dir())
            && self
                .0
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("tatacoa-sprint02-"))
        {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
}

#[test]
fn knowledge_and_replay_are_exported_with_valid_context() -> Result<(), Box<dyn std::error::Error>>
{
    let root = TestDirectory::new("knowledge-replay")?;
    let workspace = root.path().join("workspace");
    let bundle = root.path().join("bundle");
    let (engagement, session) = create_test_context(&workspace, SecurityProfile::LabLearning)?;
    let (executable, argv) = test_command();
    let manifest = execute(
        &workspace,
        &engagement.id,
        &session.id,
        executable.clone(),
        argv,
        None,
    )?;

    create_knowledge_card(
        &workspace,
        &engagement.id,
        &manifest.execution.id,
        KnowledgeCardInput {
            review_status: KnowledgeReviewStatus::Draft,
            what: "Captura de salida local".to_owned(),
            why: "Conservar una observación reproducible".to_owned(),
            objective: "Validar el flujo de evidencia".to_owned(),
            how: "Ejecutar un proceso local autorizado".to_owned(),
            observe: "stdout y stderr".to_owned(),
            proves: "Los bytes capturados coinciden con el digest".to_owned(),
            does_not_prove: "Autoría o trusted time".to_owned(),
            errors: "Un fallo de captura invalida COMPLETE".to_owned(),
            validation: "Pendiente de Validación Humana".to_owned(),
            defensive_context: "Revisar invocación y límites".to_owned(),
            references: vec![KnowledgeReference {
                classification: SourceClassification::ProjectDocumentation,
                locator: "docs/specs/ADAPTERS-KNOWLEDGE-REPLAY.md".to_owned(),
                title: "Especificación Knowledge y Replay".to_owned(),
            }],
            related_techniques: vec!["authorized-local-capture".to_owned()],
        },
    )?;
    create_replay_recipe(
        &workspace,
        &engagement.id,
        &manifest.execution.id,
        ReplayRecipeInput {
            executable,
            argv_template: vec!["--target={{TARGET}}".to_owned()],
            placeholders: vec![ReplayPlaceholder {
                name: "TARGET".to_owned(),
                description: "Target autorizado para el retest".to_owned(),
                secret: false,
                required: true,
            }],
            prerequisites: vec!["Confirmar alcance vigente".to_owned()],
            authorization_limits: vec!["Solo el target del engagement".to_owned()],
        },
    )?;

    export_bundle(
        &workspace,
        &manifest,
        &bundle,
        PlainExportAuthorization::default(),
    )?;
    let report = verify_bundle(&bundle)?;
    assert!(report.valid);
    let exported = read_bundle_manifest(&bundle)?;
    assert_eq!(exported.knowledge_cards.len(), 1);
    assert_eq!(exported.replay_recipes.len(), 1);
    assert_eq!(exported.replay_recipes[0].context.session_id, session.id);
    Ok(())
}

#[test]
fn replay_rejects_undeclared_and_malformed_placeholders() -> Result<(), Box<dyn std::error::Error>>
{
    let root = TestDirectory::new("replay-placeholders-invalid")?;
    let workspace = root.path().join("workspace");
    let (engagement, session) = create_test_context(&workspace, SecurityProfile::LabLearning)?;
    let (executable, argv) = test_command();
    let manifest = execute(
        &workspace,
        &engagement.id,
        &session.id,
        executable.clone(),
        argv,
        None,
    )?;

    let base = ReplayRecipeInput {
        executable: executable.clone(),
        argv_template: vec!["--token={{SECRET_TOKEN}}".to_owned()],
        placeholders: Vec::new(),
        prerequisites: vec!["Confirmar el entorno autorizado".to_owned()],
        authorization_limits: vec!["Solo el engagement actual".to_owned()],
    };
    assert!(
        create_replay_recipe(&workspace, &engagement.id, &manifest.execution.id, base,).is_err()
    );

    assert!(
        create_replay_recipe(
            &workspace,
            &engagement.id,
            &manifest.execution.id,
            ReplayRecipeInput {
                executable,
                argv_template: vec!["--token={{SECRET_TOKEN".to_owned()],
                placeholders: vec![ReplayPlaceholder {
                    name: "SECRET_TOKEN".to_owned(),
                    description: "Token suministrado fuera de la receta".to_owned(),
                    secret: true,
                    required: true,
                }],
                prerequisites: vec!["Confirmar el entorno autorizado".to_owned()],
                authorization_limits: vec!["Solo el engagement actual".to_owned()],
            },
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn replay_exports_secret_metadata_without_secret_values() -> Result<(), Box<dyn std::error::Error>>
{
    let root = TestDirectory::new("replay-secret-boundary")?;
    let workspace = root.path().join("workspace");
    let bundle = root.path().join("bundle");
    let (engagement, session) = create_test_context(&workspace, SecurityProfile::LabLearning)?;
    let (executable, argv) = test_command();
    let manifest = execute(
        &workspace,
        &engagement.id,
        &session.id,
        executable.clone(),
        argv,
        None,
    )?;

    create_replay_recipe(
        &workspace,
        &engagement.id,
        &manifest.execution.id,
        ReplayRecipeInput {
            executable,
            argv_template: vec!["--token={{SECRET_TOKEN}}".to_owned()],
            placeholders: vec![ReplayPlaceholder {
                name: "SECRET_TOKEN".to_owned(),
                description: "Valor secreto suministrado al reproducir".to_owned(),
                secret: true,
                required: true,
            }],
            prerequisites: vec!["Obtener el secreto por un canal autorizado".to_owned()],
            authorization_limits: vec!["No persistir el valor resuelto".to_owned()],
        },
    )?;
    export_bundle(
        &workspace,
        &manifest,
        &bundle,
        PlainExportAuthorization::default(),
    )?;

    let serialized = fs::read_to_string(bundle.join("manifest.json"))?;
    assert!(serialized.contains("{{SECRET_TOKEN}}"));
    assert!(serialized.contains("\"secret\": true"));
    assert!(!serialized.contains("tatacoa-secret-sentinel-value"));
    Ok(())
}

#[test]
fn professional_plain_requires_ack_and_high_sensitivity_denies_plain()
-> Result<(), Box<dyn std::error::Error>> {
    let root = TestDirectory::new("profiles")?;
    let workspace = root.path().join("workspace");
    let (professional, professional_session) =
        create_test_context(&workspace, SecurityProfile::Professional)?;
    let (executable, argv) = test_command();
    let professional_manifest = execute(
        &workspace,
        &professional.id,
        &professional_session.id,
        executable,
        argv,
        None,
    )?;
    assert!(
        export_bundle(
            &workspace,
            &professional_manifest,
            &root.path().join("professional-denied"),
            PlainExportAuthorization::default(),
        )
        .is_err()
    );
    export_bundle(
        &workspace,
        &professional_manifest,
        &root.path().join("professional-acknowledged"),
        PlainExportAuthorization {
            acknowledged_plaintext: true,
        },
    )?;

    let (sensitive, sensitive_session) =
        create_test_context(&workspace, SecurityProfile::HighSensitivity)?;
    let (executable, argv) = test_command();
    let sensitive_manifest = execute(
        &workspace,
        &sensitive.id,
        &sensitive_session.id,
        executable,
        argv,
        None,
    )?;
    assert!(
        export_bundle(
            &workspace,
            &sensitive_manifest,
            &root.path().join("sensitive-denied"),
            PlainExportAuthorization {
                acknowledged_plaintext: true,
            },
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn cross_engagement_context_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
    let root = TestDirectory::new("cross-context")?;
    let workspace = root.path().join("workspace");
    let first = create_engagement(&workspace, "first".to_owned(), SecurityProfile::LabLearning)?;
    let first_scope = create_scope(
        &workspace,
        &first.id,
        "first scope".to_owned(),
        "first boundary".to_owned(),
    )?;
    let first_environment = create_environment(
        &workspace,
        &first.id,
        &first_scope.id,
        "first environment".to_owned(),
    )?;
    let second = create_engagement(
        &workspace,
        "second".to_owned(),
        SecurityProfile::LabLearning,
    )?;
    let second_scope = create_scope(
        &workspace,
        &second.id,
        "second scope".to_owned(),
        "second boundary".to_owned(),
    )?;

    assert!(
        create_target(
            &workspace,
            &second.id,
            &second_scope.id,
            &first_environment.id,
            "invalid target".to_owned(),
            "outside".to_owned(),
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn ai_draft_cannot_claim_source_reviewed() -> Result<(), Box<dyn std::error::Error>> {
    let root = TestDirectory::new("ai-review")?;
    let workspace = root.path().join("workspace");
    let (engagement, session) = create_test_context(&workspace, SecurityProfile::LabLearning)?;
    let (executable, argv) = test_command();
    let manifest = execute(
        &workspace,
        &engagement.id,
        &session.id,
        executable,
        argv,
        None,
    )?;
    let result = create_knowledge_card(
        &workspace,
        &engagement.id,
        &manifest.execution.id,
        KnowledgeCardInput {
            review_status: KnowledgeReviewStatus::SourceReviewed,
            what: "what".to_owned(),
            why: "why".to_owned(),
            objective: "objective".to_owned(),
            how: "how".to_owned(),
            observe: "observe".to_owned(),
            proves: "proves".to_owned(),
            does_not_prove: "limits".to_owned(),
            errors: "errors".to_owned(),
            validation: "human pending".to_owned(),
            defensive_context: "defensive".to_owned(),
            references: vec![KnowledgeReference {
                classification: SourceClassification::AiDraft,
                locator: "local-draft".to_owned(),
                title: "Unreviewed draft".to_owned(),
            }],
            related_techniques: Vec::new(),
        },
    );
    assert!(result.is_err());
    Ok(())
}

#[test]
fn unbacked_evidence_and_provenance_cycles_are_rejected() -> Result<(), Box<dyn std::error::Error>>
{
    let root = TestDirectory::new("provenance")?;
    let workspace = root.path().join("workspace");
    let (engagement, session) = create_test_context(&workspace, SecurityProfile::LabLearning)?;
    let (executable, argv) = test_command();
    let mut manifest = execute(
        &workspace,
        &engagement.id,
        &session.id,
        executable,
        argv,
        None,
    )?;
    manifest.artifacts[0].evidence_state = EvidenceState::Validated;
    assert!(
        export_bundle(
            &workspace,
            &manifest,
            &root.path().join("false-evidence"),
            PlainExportAuthorization::default(),
        )
        .is_err()
    );

    manifest.artifacts[0].evidence_state = EvidenceState::Captured;
    let first = manifest.artifacts[0].id.clone();
    let second = manifest.artifacts[1].id.clone();
    for artifact in &mut manifest.artifacts {
        artifact.classification = ArtifactClassification::Derived;
        artifact.provenance.kind = ProvenanceKind::Copy;
    }
    manifest.artifacts[0].provenance.source_artifact_ids = vec![second];
    manifest.artifacts[1].provenance.source_artifact_ids = vec![first];
    assert!(
        export_bundle(
            &workspace,
            &manifest,
            &root.path().join("cycle"),
            PlainExportAuthorization::default(),
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn sprint01_manifest_remains_readable_and_verifiable() -> Result<(), Box<dyn std::error::Error>> {
    let root = TestDirectory::new("legacy-v1")?;
    let workspace = root.path().join("workspace");
    let bundle = root.path().join("bundle");
    let (engagement, session) = create_test_context(&workspace, SecurityProfile::LabLearning)?;
    let (executable, argv) = test_command();
    let manifest = execute(
        &workspace,
        &engagement.id,
        &session.id,
        executable,
        argv,
        None,
    )?;
    export_bundle(
        &workspace,
        &manifest,
        &bundle,
        PlainExportAuthorization::default(),
    )?;

    let manifest_path = bundle.join("manifest.json");
    let mut json: serde_json::Value = serde_json::from_slice(&fs::read(&manifest_path)?)?;
    let object = json
        .as_object_mut()
        .ok_or("serialized manifest must be an object")?;
    object.insert(
        "schema_version".to_owned(),
        serde_json::Value::String("tatacoa.alpha.v1".to_owned()),
    );
    object.remove("context");
    object.remove("knowledge_cards");
    object.remove("replay_recipes");
    object
        .get_mut("execution")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or("execution must be an object")?
        .remove("context");
    for artifact in object
        .get_mut("artifacts")
        .and_then(serde_json::Value::as_array_mut)
        .ok_or("artifacts must be an array")?
    {
        let artifact = artifact
            .as_object_mut()
            .ok_or("artifact must be an object")?;
        artifact.remove("evidence_state");
        artifact.remove("provenance");
    }
    fs::write(&manifest_path, serde_json::to_vec_pretty(&json)?)?;

    let report = verify_bundle(&bundle)?;
    assert!(report.valid);
    assert_eq!(report.schema_version, "tatacoa.alpha.v1");
    Ok(())
}

fn create_test_context(
    workspace: &Path,
    profile: SecurityProfile,
) -> Result<(Engagement, Session), Box<dyn std::error::Error>> {
    let engagement = create_engagement(workspace, "Sprint 02 test".to_owned(), profile)?;
    let scope = create_scope(
        workspace,
        &engagement.id,
        "authorized scope".to_owned(),
        "local target only".to_owned(),
    )?;
    let environment = create_environment(
        workspace,
        &engagement.id,
        &scope.id,
        "test environment".to_owned(),
    )?;
    let target = create_target(
        workspace,
        &engagement.id,
        &scope.id,
        &environment.id,
        "local target".to_owned(),
        "localhost".to_owned(),
    )?;
    let session = create_session(
        workspace,
        &engagement.id,
        &scope.id,
        &environment.id,
        &target.id,
        "test session".to_owned(),
    )?;
    Ok((engagement, session))
}

#[cfg(windows)]
fn test_command() -> (String, Vec<String>) {
    (
        "cmd.exe".to_owned(),
        vec![
            "/D".to_owned(),
            "/S".to_owned(),
            "/C".to_owned(),
            "<nul set /p =alpha & <nul set /p =beta 1>&2".to_owned(),
        ],
    )
}

#[cfg(unix)]
fn test_command() -> (String, Vec<String>) {
    (
        "/bin/sh".to_owned(),
        vec!["-c".to_owned(), "printf alpha; printf beta >&2".to_owned()],
    )
}
