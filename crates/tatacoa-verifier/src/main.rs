#![forbid(unsafe_code)]

use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;
use tatacoa_verifier::verify_bundle;

#[derive(Debug, Parser)]
#[command(
    name = "tatacoa-verify",
    about = "Offline, read-only TATACOA bundle verifier"
)]
struct Cli {
    /// Portable bundle directory to verify.
    bundle: PathBuf,

    /// Emit a machine-readable JSON report.
    #[arg(long)]
    json: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match verify_bundle(&cli.bundle) {
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
