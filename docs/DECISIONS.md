# Registro de decisiones congeladas

## Estado

Estas decisiones provienen del Discovery aprobado. Cambios sustanciales requieren Validación Humana y, cuando corresponda, ADR.

| ID | Decisión | Estado |
|---|---|---|
| D-001 | Producto local-first, sin telemetría por defecto | APPROVED |
| D-002 | Rust para Core/CLI/verifier | APPROVED |
| D-003 | Filesystem + manifests en Alpha; DB no obligatoria | APPROVED |
| D-004 | Verifier separado, read-only y offline | APPROVED |
| D-005 | Generic Execution Adapter primero | APPROVED |
| D-006 | Finding no es objeto primario V1 | APPROVED |
| D-007 | Validación Humana promueve evidencia | APPROVED |
| D-008 | RAW preservado; derivados tienen identidad propia | APPROVED |
| D-009 | SHA-256 baseline Alpha | APPROVED |
| D-010 | Security Profiles y cifrado configurable por política | APPROVED |
| D-011 | No custom crypto | APPROVED |
| D-012 | Kali + Parrot como QA objetivo; Windows 11 x64 como desarrollo y QA; WSL2 como desarrollo/integración Linux | APPROVED |
| D-013 | No plugins arbitrarios de terceros V1 | APPROVED |
| D-014 | Knowledge/Learning es parte del producto | APPROVED |
| D-015 | Replay es capacidad central | APPROVED |
| D-016 | Firma y RFC 3161 post-V1, arquitectura-ready | APPROVED |
| D-017 | Core V1 multiplataforma Windows + Linux; compatibilidad de adapters/herramientas se declara por separado | APPROVED |
| D-018 | Open specs para manifest/provenance/verifier | APPROVED |
| D-019 | Claims de certificación solo con evidencia formal | APPROVED |
| D-020 | IA puede asistir pero no validar automáticamente | APPROVED |
| D-021 | Encrypted v1 usa Argon2id v0x13 (64 MiB, t=3, p=4), KEK de 256 bits, Bundle Key aleatoria, HKDF-SHA-256, DEKs por objeto y AES-256-GCM con STREAM-BE32 | APPROVED |
| D-022 | `tatacoa.encrypted.v1` usa header fijo de 136 bytes, chunks de 1 MiB y los límites de lector documentados en `security/ENCRYPTED-V1-FORMAT-PROPOSAL.md` | APPROVED |
| D-023 | El CLI recibe passwords Encrypted desde TTY mediante prompt sin eco; export solicita confirmación y verify una entrada | APPROVED |

## Decisiones pendientes de spike

- estrategia async solo si la necesidad lo exige;
- política de release signing;
- SBOM tooling;
- fuzzing toolchain.
