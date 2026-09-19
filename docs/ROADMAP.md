# Sprint 01 — Foundation / Alpha Roadmap

## Objetivo de salida

Entregar una vertical mínima verificable:

`ENGAGEMENT → CONTEXT → EXECUTION → ARTIFACT → SHA-256 → MANIFEST → EXPORT → VERIFY`

El Core V1 se desarrolla como multiplataforma Windows + Linux desde Sprint 01.

## Fase 1 — Bootstrap técnico

- workspace Rust;
- crates/módulos mínimos según spike;
- CLI inicial;
- IDs y errores tipados;
- tests unitarios;
- format/lint/test en CI;
- lockfile versionado;
- política `unsafe` restrictiva;
- bootstrap verificable en Windows 11 x64 y Linux sin introducir supuestos innecesarios de plataforma.

## Fase 2 — Context + Execution

- Engagement/context;
- snapshot de contexto;
- executable + argv;
- stdout/stderr streaming;
- estados COMPLETE/PARTIAL/TRUNCATED/FAILED;
- aislamiento entre engagements;
- Generic Execution Adapter;
- separación explícita entre comportamiento portable del Core y capacidades específicas de plataforma.

## Fase 3 — Artifact + Manifest

- finalización atómica;
- SHA-256;
- metadata;
- manifest versionado;
- paths relativos;
- validación estructural;
- tests de tampering/traversal/symlink;
- tratamiento portable de paths y filesystem sin asumir semántica exclusiva de Windows o POSIX.

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
- Windows 11 x64;
- WSL2 Linux x64 como integración Linux temprana;
- Kali Linux x64;
- Parrot OS x64;
- non-root donde aplique;
- parallel sessions;
- isolation;
- tamper;
- traversal/symlink según semántica de plataforma;
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

### Estado de QA ejecutado

- Windows 11 x64: PASS para build/check/test y vertical funcional evaluada.
- WSL2 Ubuntu 24.04.5 LTS x86_64, non-root: PASS para fresh clone, build/check/test, captura/export/verificación, tamper, symlink hostil, truncamiento, aislamiento funcional, fallo controlado de spawn y portabilidad bidireccional Windows ↔ Linux.
- Kali Linux x64: pendiente.
- Parrot OS x64: pendiente.
- Parallel sessions/concurrencia manual: pendiente.
- Malicious/huge output: parcialmente cubierto por truncamiento automatizado y manual; prueba ampliada pendiente.
- Secret boundary, zero telemetry y AI isolation: revisión explícita pendiente antes del cierre Alpha.
- Redaction/provenance: se valida cuando entre en scope; no se declara implementado en Sprint 01.

El detalle reproducible de los controles ejecutados está en [ALPHA-IMPLEMENTATION.md](ALPHA-IMPLEMENTATION.md).

WSL2 no sustituye el QA específico de Kali o Parrot.

## Definition of Alpha

Alpha no significa “feature complete”. Significa que la vertical fundamental funciona de extremo a extremo, sus propiedades declaradas tienen pruebas reproducibles y las limitaciones están documentadas en los targets realmente ejecutados.

## Post-V1 / investigación

GUI, reporting avanzado, colaboración, public-key recipients, signatures, RFC 3161, encrypted workspace, hardware keys/PKCS#11/TPM/KMS, plugin sandboxing y AI local.
