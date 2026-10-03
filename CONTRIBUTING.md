# Contributing to T·A·T·A·C·O·A

Thank you for your interest in improving TATACOA.

## Workflow

1. Start from an up-to-date `main`.
2. Create a focused branch for the change.
3. Keep commits logically scoped and reviewable.
4. Run the applicable checks locally.
5. Open a Pull Request against `main`.
6. Describe what changed, why, how it was validated, and any known security impact.

Suggested branch prefixes include `feat/`, `fix/`, `docs/`, `test/`, `chore/` and `security/`.

## Validation

For Rust changes, run the checks documented in `README.md`. Do not claim a test or QA result that was not actually executed.

## Dependencies

Before adding a dependency, confirm there is a concrete need and consider its maintenance status, provenance, known security issues, license and technical suitability.

## Security

Do not disclose vulnerabilities in a public Issue or Pull Request. Follow `SECURITY.md` for private reporting.

Never include secrets, real client data or material from systems you are not authorized to assess.
