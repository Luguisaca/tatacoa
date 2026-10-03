#![forbid(unsafe_code)]

use clap::{Parser, ValueEnum};
use std::path::PathBuf;
use std::process::ExitCode;
use tatacoa_core::{
    SecretPassword, TimestampObject, TimestampReport, TsaTrustPolicy, verify_timestamp_sidecar,
    verify_timestamp_sidecar_with_trust,
};
use tatacoa_verifier::{verify_bundle, verify_encrypted_bundle};

#[derive(Debug, Parser)]
#[command(
    name = "tatacoa-verify",
    about = "Offline, read-only TATACOA bundle verifier"
)]
struct Cli {
    /// Plain bundle directory or Encrypted v1 bundle file to verify.
    bundle: PathBuf,

    /// Emit a machine-readable JSON report.
    #[arg(long)]
    json: bool,

    /// RFC 3161 DER sidecar to inspect offline after bundle integrity passes.
    #[arg(long, requires = "timestamp_mode")]
    timestamp_sidecar: Option<PathBuf>,

    /// Object mode bound by the timestamp sidecar.
    #[arg(long, value_enum, requires = "timestamp_sidecar")]
    timestamp_mode: Option<TimestampMode>,

    /// Explicit DER TSA trust anchor; may be repeated.
    #[arg(long = "tsa-trust-anchor-der", requires = "timestamp_sidecar")]
    trust_anchors: Vec<PathBuf>,

    /// Explicit DER TSA intermediate; may be repeated.
    #[arg(long = "tsa-intermediate-der", requires = "timestamp_sidecar")]
    trust_intermediates: Vec<PathBuf>,

    /// Accepted TSA policy OID; may be repeated.
    #[arg(long = "tsa-policy", requires = "timestamp_sidecar")]
    accepted_policies: Vec<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum TimestampMode {
    Plain,
    Encrypted,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = if cli.bundle.is_file() {
        let password = match rpassword::prompt_password("Encrypted bundle password: ")
            .map_err(|error| error.to_string())
            .and_then(|password| {
                SecretPassword::for_verification(password).map_err(|error| error.to_string())
            }) {
            Ok(password) => password,
            Err(error) => {
                eprintln!("VERIFICATION ERROR: {error}");
                return ExitCode::from(2);
            }
        };
        verify_encrypted_bundle(&cli.bundle, &password)
    } else {
        verify_bundle(&cli.bundle)
    };
    match result {
        Ok(report) => {
            let timestamp = if report.valid {
                match inspect_timestamp(&cli) {
                    Ok(value) => value,
                    Err(error) => {
                        eprintln!("TIMESTAMP VERIFICATION ERROR: {error}");
                        return ExitCode::from(2);
                    }
                }
            } else {
                None
            };
            if cli.json {
                let combined = serde_json::json!({
                    "bundle": report,
                    "timestamp": timestamp,
                });
                match serde_json::to_string_pretty(&combined) {
                    Ok(json) => println!("{json}"),
                    Err(error) => {
                        eprintln!("verification report error: {error}");
                        return ExitCode::from(2);
                    }
                }
            } else {
                for artifact in &report.artifacts {
                    let status = if artifact.valid { "VALID" } else { "INVALID" };
                    println!("{status} {}: {}", artifact.path, artifact.message);
                }
                println!(
                    "VERIFICATION: {}",
                    if report.valid { "VALID" } else { "INVALID" }
                );
                if let Some(timestamp) = &timestamp {
                    print_timestamp(timestamp);
                }
            }
            if report.valid {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(error) => {
            eprintln!("VERIFICATION ERROR: {error}");
            ExitCode::from(2)
        }
    }
}

fn inspect_timestamp(cli: &Cli) -> Result<Option<TimestampReport>, tatacoa_core::Error> {
    let (Some(sidecar), Some(mode)) = (&cli.timestamp_sidecar, cli.timestamp_mode) else {
        return Ok(None);
    };
    let object = match mode {
        TimestampMode::Plain => TimestampObject::PlainBundle(&cli.bundle),
        TimestampMode::Encrypted => TimestampObject::EncryptedBundle(&cli.bundle),
    };
    if cli.trust_anchors.is_empty()
        && cli.trust_intermediates.is_empty()
        && cli.accepted_policies.is_empty()
    {
        return verify_timestamp_sidecar(object, sidecar).map(Some);
    }
    let anchors = cli
        .trust_anchors
        .iter()
        .map(std::fs::read)
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|source| tatacoa_core::Error::io("read TSA trust anchor DER", source))?;
    let intermediates = cli
        .trust_intermediates
        .iter()
        .map(std::fs::read)
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|source| tatacoa_core::Error::io("read TSA intermediate DER", source))?;
    let policy = TsaTrustPolicy::new(anchors, intermediates, cli.accepted_policies.clone())?;
    verify_timestamp_sidecar_with_trust(object, sidecar, &policy).map(Some)
}

fn print_timestamp(report: &TimestampReport) {
    println!("TIMESTAMP ASSURANCE: {:?}", report.assurance);
    for check in &report.checks {
        println!("{:?} {}: {}", check.status, check.name, check.detail);
    }
}
