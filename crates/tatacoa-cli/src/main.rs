#![forbid(unsafe_code)]

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;
use tatacoa_core::{
    EngagementId, EnvironmentId, ExecutionId, KnowledgeCardInput, KnowledgeReference,
    KnowledgeReviewStatus, PlainExportAuthorization, ReplayPlaceholder, ReplayRecipeInput, ScopeId,
    SecurityProfile, SessionId, SourceClassification, TargetId, create_engagement,
    create_environment, create_knowledge_card, create_replay_recipe, create_scope, create_session,
    create_target, execute, export_bundle, load_execution_manifest,
};
use tatacoa_verifier::verify_bundle;

#[derive(Debug, Parser)]
#[command(
    name = "tatacoa",
    version,
    about = "Verifiable local execution capture"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Create an isolated engagement context.
    EngagementCreate {
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long)]
        name: String,
        #[arg(long, value_enum, default_value_t = Profile::LabLearning)]
        security_profile: Profile,
    },
    /// Add an authorized scope to an engagement.
    ScopeCreate {
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long)]
        engagement: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        authorization_boundary: String,
    },
    /// Add an environment inside a scope.
    EnvironmentCreate {
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long)]
        engagement: String,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        name: String,
    },
    /// Add a target inside an environment and scope.
    TargetCreate {
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long)]
        engagement: String,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        environment: String,
        #[arg(long)]
        label: String,
        #[arg(long)]
        locator: String,
    },
    /// Create an execution session bound to a target context.
    SessionCreate {
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long)]
        engagement: String,
        #[arg(long)]
        scope: String,
        #[arg(long)]
        environment: String,
        #[arg(long)]
        target: String,
        #[arg(long)]
        name: String,
    },
    /// Execute without an implicit shell and capture RAW output in a full context.
    Run {
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long)]
        engagement: String,
        #[arg(long)]
        session: String,
        #[arg(long)]
        bundle: Option<PathBuf>,
        #[arg(long)]
        acknowledge_plain_export: bool,
        /// Maximum bytes preserved independently for stdout and stderr; remaining bytes are drained.
        #[arg(long)]
        max_stream_bytes: Option<u64>,
        #[arg(required = true)]
        executable: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        argv: Vec<String>,
    },
    /// Export an existing execution with associated Knowledge and Replay records.
    Export {
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long)]
        engagement: String,
        #[arg(long)]
        execution: String,
        #[arg(long)]
        bundle: PathBuf,
        #[arg(long)]
        acknowledge_plain_export: bool,
    },
    /// Create a manual, versioned Knowledge Card for an execution.
    KnowledgeCreate {
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long)]
        engagement: String,
        #[arg(long)]
        execution: String,
        #[arg(long)]
        source_reviewed: bool,
        #[arg(long)]
        what: String,
        #[arg(long)]
        why: String,
        #[arg(long)]
        objective: String,
        #[arg(long)]
        how: String,
        #[arg(long)]
        observe: String,
        #[arg(long)]
        proves: String,
        #[arg(long)]
        does_not_prove: String,
        #[arg(long)]
        errors: String,
        #[arg(long)]
        validation: String,
        #[arg(long)]
        defensive_context: String,
        /// CLASSIFICATION|TITLE|LOCATOR; may be repeated.
        #[arg(long = "reference", required = true)]
        references: Vec<String>,
        #[arg(long = "related-technique")]
        related_techniques: Vec<String>,
    },
    /// Create a replay recipe foundation; this command never executes it.
    ReplayCreate {
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long)]
        engagement: String,
        #[arg(long)]
        execution: String,
        #[arg(long)]
        executable: String,
        #[arg(long = "arg")]
        argv_template: Vec<String>,
        /// NAME|SECRET|REQUIRED|DESCRIPTION; may be repeated.
        #[arg(long = "placeholder")]
        placeholders: Vec<String>,
        #[arg(long = "prerequisite")]
        prerequisites: Vec<String>,
        #[arg(long = "authorization-limit", required = true)]
        authorization_limits: Vec<String>,
    },
    /// Verify a bundle offline without executing its contents.
    Verify { bundle: PathBuf },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Profile {
    LabLearning,
    Professional,
    HighSensitivity,
    Custom,
}

impl From<Profile> for SecurityProfile {
    fn from(profile: Profile) -> Self {
        match profile {
            Profile::LabLearning => Self::LabLearning,
            Profile::Professional => Self::Professional,
            Profile::HighSensitivity => Self::HighSensitivity,
            Profile::Custom => Self::Custom,
        }
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::EngagementCreate {
            workspace,
            name,
            security_profile,
        } => {
            let engagement = create_engagement(&workspace, name, security_profile.into())?;
            println!("{}", engagement.id);
        }
        Commands::ScopeCreate {
            workspace,
            engagement,
            name,
            authorization_boundary,
        } => {
            let engagement_id = EngagementId::from_str(&engagement)?;
            let scope = create_scope(&workspace, &engagement_id, name, authorization_boundary)?;
            println!("{}", scope.id);
        }
        Commands::EnvironmentCreate {
            workspace,
            engagement,
            scope,
            name,
        } => {
            let engagement_id = EngagementId::from_str(&engagement)?;
            let scope_id = ScopeId::from_str(&scope)?;
            let environment = create_environment(&workspace, &engagement_id, &scope_id, name)?;
            println!("{}", environment.id);
        }
        Commands::TargetCreate {
            workspace,
            engagement,
            scope,
            environment,
            label,
            locator,
        } => {
            let engagement_id = EngagementId::from_str(&engagement)?;
            let scope_id = ScopeId::from_str(&scope)?;
            let environment_id = EnvironmentId::from_str(&environment)?;
            let target = create_target(
                &workspace,
                &engagement_id,
                &scope_id,
                &environment_id,
                label,
                locator,
            )?;
            println!("{}", target.id);
        }
        Commands::SessionCreate {
            workspace,
            engagement,
            scope,
            environment,
            target,
            name,
        } => {
            let engagement_id = EngagementId::from_str(&engagement)?;
            let scope_id = ScopeId::from_str(&scope)?;
            let environment_id = EnvironmentId::from_str(&environment)?;
            let target_id = TargetId::from_str(&target)?;
            let session = create_session(
                &workspace,
                &engagement_id,
                &scope_id,
                &environment_id,
                &target_id,
                name,
            )?;
            println!("{}", session.id);
        }
        Commands::Run {
            workspace,
            engagement,
            session,
            bundle,
            acknowledge_plain_export,
            max_stream_bytes,
            executable,
            argv,
        } => {
            let engagement_id = EngagementId::from_str(&engagement)?;
            let session_id = SessionId::from_str(&session)?;
            let manifest = execute(
                &workspace,
                &engagement_id,
                &session_id,
                executable,
                argv,
                max_stream_bytes,
            )?;
            println!("execution={}", manifest.execution.id);
            println!("capture_status={:?}", manifest.execution.capture_status);
            if let Some(bundle) = bundle {
                export_bundle(
                    &workspace,
                    &manifest,
                    &bundle,
                    PlainExportAuthorization {
                        acknowledged_plaintext: acknowledge_plain_export,
                    },
                )?;
                println!("bundle={}", bundle.display());
            }
        }
        Commands::Export {
            workspace,
            engagement,
            execution,
            bundle,
            acknowledge_plain_export,
        } => {
            let engagement_id = EngagementId::from_str(&engagement)?;
            let execution_id = ExecutionId::from_str(&execution)?;
            let manifest = load_execution_manifest(&workspace, &engagement_id, &execution_id)?;
            export_bundle(
                &workspace,
                &manifest,
                &bundle,
                PlainExportAuthorization {
                    acknowledged_plaintext: acknowledge_plain_export,
                },
            )?;
            println!("bundle={}", bundle.display());
        }
        Commands::KnowledgeCreate {
            workspace,
            engagement,
            execution,
            source_reviewed,
            what,
            why,
            objective,
            how,
            observe,
            proves,
            does_not_prove,
            errors,
            validation,
            defensive_context,
            references,
            related_techniques,
        } => {
            let engagement_id = EngagementId::from_str(&engagement)?;
            let execution_id = ExecutionId::from_str(&execution)?;
            let references = references
                .into_iter()
                .map(|value| parse_reference(&value))
                .collect::<Result<Vec<_>, _>>()?;
            let card = create_knowledge_card(
                &workspace,
                &engagement_id,
                &execution_id,
                KnowledgeCardInput {
                    review_status: if source_reviewed {
                        KnowledgeReviewStatus::SourceReviewed
                    } else {
                        KnowledgeReviewStatus::Draft
                    },
                    what,
                    why,
                    objective,
                    how,
                    observe,
                    proves,
                    does_not_prove,
                    errors,
                    validation,
                    defensive_context,
                    references,
                    related_techniques,
                },
            )?;
            println!("{}", card.id);
        }
        Commands::ReplayCreate {
            workspace,
            engagement,
            execution,
            executable,
            argv_template,
            placeholders,
            prerequisites,
            authorization_limits,
        } => {
            let engagement_id = EngagementId::from_str(&engagement)?;
            let execution_id = ExecutionId::from_str(&execution)?;
            let placeholders = placeholders
                .into_iter()
                .map(|value| parse_placeholder(&value))
                .collect::<Result<Vec<_>, _>>()?;
            let recipe = create_replay_recipe(
                &workspace,
                &engagement_id,
                &execution_id,
                ReplayRecipeInput {
                    executable,
                    argv_template,
                    placeholders,
                    prerequisites,
                    authorization_limits,
                },
            )?;
            println!("{}", recipe.id);
        }
        Commands::Verify { bundle } => {
            let report = verify_bundle(&bundle)?;
            for artifact in &report.artifacts {
                let status = if artifact.valid { "VALID" } else { "INVALID" };
                println!("{status} {}: {}", artifact.path, artifact.message);
            }
            if !report.valid {
                return Err("bundle integrity verification failed".into());
            }
            println!("VERIFICATION: VALID");
        }
    }
    Ok(())
}

fn parse_reference(value: &str) -> Result<KnowledgeReference, String> {
    let mut parts = value.splitn(3, '|');
    let classification = parts.next().unwrap_or_default();
    let title = parts.next().unwrap_or_default();
    let locator = parts.next().unwrap_or_default();
    if title.trim().is_empty() || locator.trim().is_empty() {
        return Err("reference must be CLASSIFICATION|TITLE|LOCATOR".to_owned());
    }
    let classification = match classification {
        "UPSTREAM_OFFICIAL" => SourceClassification::UpstreamOfficial,
        "STANDARD" => SourceClassification::Standard,
        "GOVERNMENT" => SourceClassification::Government,
        "PROJECT_DOCUMENTATION" => SourceClassification::ProjectDocumentation,
        "OPERATOR_NOTE" => SourceClassification::OperatorNote,
        "AI_DRAFT" => SourceClassification::AiDraft,
        "COMMUNITY" => SourceClassification::Community,
        _ => return Err(format!("unknown source classification: {classification}")),
    };
    Ok(KnowledgeReference {
        classification,
        locator: locator.to_owned(),
        title: title.to_owned(),
    })
}

fn parse_placeholder(value: &str) -> Result<ReplayPlaceholder, String> {
    let mut parts = value.splitn(4, '|');
    let name = parts.next().unwrap_or_default();
    let secret = parts
        .next()
        .unwrap_or_default()
        .parse::<bool>()
        .map_err(|_| "placeholder SECRET must be true or false".to_owned())?;
    let required = parts
        .next()
        .unwrap_or_default()
        .parse::<bool>()
        .map_err(|_| "placeholder REQUIRED must be true or false".to_owned())?;
    let description = parts.next().unwrap_or_default();
    if name.is_empty() || description.trim().is_empty() {
        return Err("placeholder must be NAME|SECRET|REQUIRED|DESCRIPTION".to_owned());
    }
    Ok(ReplayPlaceholder {
        name: name.to_owned(),
        description: description.to_owned(),
        secret,
        required,
    })
}
