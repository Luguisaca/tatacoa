# T·A·T·A·C·O·A

**Test · Analyze · Trace · Assure · Contextualize · Observe · Apply**

TATACOA is a local-first workspace for authorized security assessment evidence. It helps preserve context, executions, artifacts, provenance, knowledge and reproducible retest workflows without making automation or AI the authority for validation.

## Components

- `tatacoa-core` — domain authority for workspaces, evidence, policy, cryptography, replay and provenance.
- `tatacoa-app-api` — application-facing API over the core.
- `tatacoa-cli` — command-line interface.
- `tatacoa-desktop` — Tauri desktop client.
- `tatacoa-verifier` — independent verification utilities.

CLI and Desktop are first-class clients of the same core. Local operation does not require a SaaS account or permanent Internet connection. Features that inherently use external services must make that dependency explicit.

## Build and test

TATACOA currently targets Rust 1.98.

```sh
cargo fmt --check
cargo check --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo build --locked --workspace
```

Building the Tauri desktop client on Linux requires the platform libraries used by Tauri/WebKitGTK.

## Security and responsible use

Use TATACOA only for systems and environments you own or are explicitly authorized to assess. Treat imported bundles and artifacts as untrusted input. Automated output is not equivalent to a validated finding.

See `SECURITY.md` for vulnerability reporting and assurance boundaries.

## Project status

TATACOA is pre-release software. Interfaces and formats may evolve before a stable release. Public repository contents describe distributable product behavior; internal engineering history, experiments and private assessment evidence are intentionally not part of this distribution surface.

## License

TATACOA is distributed under the **PolyForm Noncommercial License 1.0.0**. See `LICENSE` and `NOTICE`.

**A LUGUISACA project.**
