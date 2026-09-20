# Registro de decisiones congeladas

## Estado

Estas decisiones provienen del Discovery y de decisiones humanas posteriores aprobadas. Cambios sustanciales requieren Validación Humana y, cuando corresponda, ADR.

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
| D-016 | Firma permanece en horizonte; RFC 3161 pasa a investigación/diseño activo para la evolución de Alpha, sin mecanismo concreto aprobado todavía | SUPERSEDED/PARTIAL |
| D-017 | Core V1 multiplataforma Windows + Linux; compatibilidad de adapters/herramientas se declara por separado | APPROVED |
| D-018 | Open specs para manifest/provenance/verifier | APPROVED |
| D-019 | Claims de certificación solo con evidencia formal | APPROVED |
| D-020 | IA puede asistir pero no validar automáticamente | APPROVED |
| D-021 | Encrypted v1 usa Argon2id v0x13 (64 MiB, t=3, p=4), KEK de 256 bits, Bundle Key aleatoria, HKDF-SHA-256, DEKs por objeto y AES-256-GCM con STREAM-BE32 | APPROVED |
| D-022 | `tatacoa.encrypted.v1` usa header fijo de 136 bytes, chunks de 1 MiB y límites documentados en `security/ENCRYPTED-V1-FORMAT-PROPOSAL.md` | APPROVED |
| D-023 | CLI recibe passwords Encrypted desde TTY mediante prompt sin eco; export solicita confirmación y verify una entrada | APPROVED |
| D-024 | LAB_LEARNING usa Plain por defecto y permite Encrypted; PROFESSIONAL usa Encrypted por defecto y Plain exige acknowledgement; HIGH_SENSITIVITY solo permite Encrypted; CUSTOM niega ambos hasta aprobar su política | APPROVED |
| D-025 | Password de creación Encrypted exige mínimo 12 caracteres Unicode y máximo 1024 bytes UTF-8, sin reglas compositivas ni normalización silenciosa; verify conserva compatibilidad con passwords no vacías dentro del máximo | APPROVED |
| D-026 | `tatacoa-core` continúa como autoridad de dominio, lógica, seguridad, evidencia, políticas y cifrado | APPROVED |
| D-027 | Crear `tatacoa-app-api` en SP3 como capa común de operaciones de usuario, sin convertirla por defecto en servicio de red/cloud | APPROVED |
| D-028 | Crear `tatacoa-desktop` con Tauri 2 en SP3 | APPROVED |
| D-029 | CLI y Desktop son interfaces/instalaciones independientes de primera clase; CLI no requiere GUI/Tauri y Desktop no requiere CLI preinstalado | APPROVED |
| D-030 | Funciones locales deben poder operar sin SaaS/cuenta cloud/conexión permanente; dependencias de red futuras deben ser explícitas | APPROVED |
| D-031 | Continuidad del trabajo — pausa, persistencia segura, reanudación, recuperación y contexto/resumen — es capacidad de producto | APPROVED |
| D-032 | El roadmap es acumulativo: un sprint extiende la dirección del producto y no puede redefinir silenciosamente arquitectura/decisiones previas | APPROVED |
| D-033 | Capacidades futuras conocidas se mantienen como horizonte sin asignarlas automáticamente a un sprint ni tratarlas como implementadas | APPROVED |

## Decisiones pendientes de diseño/spike

- RFC 3161: objeto(s) a timestamp-ear, momento del flujo, política/TSA, representación, verificación y comportamiento offline;
- modelo de protección/autenticación para reabrir trabajos protegidos;
- persistencia exacta del estado de continuidad y recuperación;
- límites/contratos de importación de paquetes compatibles;
- estrategia async solo si la necesidad lo exige;
- política de release signing;
- SBOM tooling;
- fuzzing toolchain.

Una decisión pendiente no autoriza implementación por inferencia.
