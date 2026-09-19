# Sprint 01 — Foundation / Alpha Roadmap

## Objetivo de salida

Entregar una vertical mínima verificable:

`ENGAGEMENT → CONTEXT → EXECUTION → ARTIFACT → SHA-256 → MANIFEST → EXPORT → VERIFY`

## Fase 1 — Bootstrap técnico

- workspace Rust;
- crates/módulos mínimos según spike;
- CLI inicial;
- IDs y errores tipados;
- tests unitarios;
- format/lint/test en CI;
- lockfile versionado;
- política `unsafe` restrictiva.

## Fase 2 — Context + Execution

- Engagement/context;
- snapshot de contexto;
- executable + argv;
- stdout/stderr streaming;
- estados COMPLETE/PARTIAL/TRUNCATED/FAILED;
- aislamiento entre engagements;
- Generic Execution Adapter.

## Fase 3 — Artifact + Manifest

- finalización atómica;
- SHA-256;
- metadata;
- manifest versionado;
- paths relativos;
- validación estructural;
- tests de tampering/traversal/symlink.

## Fase 4 — Bundle + Verifier

- export portable;
- verifier read-only/offline;
- digest/reference/DAG checks;
- resultados explícitos;
- hostile-bundle tests.

## Fase 5 — Security Profiles

- Plain;
- Encrypted tras spike criptográfico;
- política de downgrade;
- wrong-password/tamper negative tests;
- Public/Sanitized como derivación.

## Fase 6 — Knowledge + Replay foundation

- Knowledge Card manual;
- source classification;
- receta de replay con placeholders;
- vínculo execution/evidence/retest;
- sin IA automática.

## QA Alpha obligatorio

- fresh clone;
- build/check/test;
- Kali;
- Parrot;
- non-root;
- parallel sessions;
- isolation;
- tamper;
- traversal/symlink;
- malicious/huge output;
- recorder failure;
- secret boundary;
- redaction/provenance cuando entre en scope;
- offline verifier;
- zero telemetry;
- portable bundle;
- unknown adapter/failure;
- AI isolation;
- documentación actualizada.

## Definition of Alpha

Alpha no significa “feature complete”. Significa que la vertical fundamental funciona de extremo a extremo, sus propiedades declaradas tienen pruebas reproducibles y las limitaciones están documentadas.

## Post-V1 / investigación

GUI, Windows completo, reporting avanzado, colaboración, public-key recipients, signatures, RFC 3161, encrypted workspace, hardware keys/PKCS#11/TPM/KMS, plugin sandboxing y AI local.
