#![forbid(unsafe_code)]

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;
use tatacoa_core::{EngagementId, SecurityProfile, create_engagement, execute, export_bundle};
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
    /// Execute a program without an implicit shell, capture RAW output, and export a Plain bundle.
    Run {
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long)]
        engagement: String,
        #[arg(long)]
        bundle: PathBuf,
        /// Maximum bytes preserved independently for stdout and stderr; remaining bytes are drained.
        #[arg(long)]
        max_stream_bytes: Option<u64>,
        #[arg(required = true)]
        executable: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        argv: Vec<String>,
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
        Commands::Run {
            workspace,
            engagement,
            bundle,
            max_stream_bytes,
            executable,
            argv,
        } => {
            let engagement_id = EngagementId::from_str(&engagement)?;
            let manifest = execute(
                &workspace,
                &engagement_id,
                executable,
                argv,
                max_stream_bytes,
            )?;
            export_bundle(&workspace, &manifest, &bundle)?;
            println!("execution={}", manifest.execution.id);
            println!("capture_status={:?}", manifest.execution.capture_status);
            println!("bundle={}", bundle.display());
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
