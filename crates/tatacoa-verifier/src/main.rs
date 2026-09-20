#![forbid(unsafe_code)]

use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;
use tatacoa_core::SecretPassword;
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
            if cli.json {
                match serde_json::to_string_pretty(&report) {
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
