# Security Policy

TATACOA is pre-release security software. This policy describes how to report security issues; it does not imply certification, forensic accreditation or compliance with a particular standard.

## Reporting a vulnerability

Please do not disclose exploitable vulnerabilities, credentials, secrets, unnecessary personal data, client information or sensitive evidence in a public Issue, Pull Request or Discussion.

Use **GitHub Private Vulnerability Reporting** when it is enabled for this repository. If that channel is unavailable, contact `contacto@luguisaca.com` privately.

For ordinary non-security bugs, use GitHub Issues when enabled or contact `bugs@luguisaca.com`.

When safe to share, include the affected commit or version, reproduction conditions, observed impact and a minimal proof. Do not send third-party secrets or data that are unnecessary to investigate the report.

## Areas of interest

Security reports are especially relevant when they affect:

- evidence integrity or substitution;
- engagement/workspace isolation;
- path traversal, symlinks or access outside a workspace;
- command execution or injection;
- secret handling;
- parsing of hostile input;
- manifests, provenance or verification;
- artifact derivation and redaction;
- export or encryption;
- replay;
- dependencies and software supply chain.

## Remediation

A report is not considered resolved solely because it stops reproducing. Fixes should include root-cause review and, when practical, a regression test.

## Supported versions and disclosure

TATACOA is currently pre-release. A stable-version support policy, response targets, advisories and coordinated disclosure process will be defined before a stable public release.

## Security claims

Do not interpret this project as FIPS validated, ISO certified, forensically certified, tamper-proof, unhackable, government approved or equivalent unless a future release provides explicit independent evidence for that exact claim.
