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
| D-016 | Firma permanece en horizonte; RFC 3161 entra en SP3 como diseño + primera implementación funcional. Objeto y momento fueron aprobados en D-038; firma/trust/histórico progresan según D-043 | APPROVED/PARTIAL |
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
| D-031 | Continuidad del trabajo — guardado seguro, pausa, reanudación, recuperación tras cierre/fallo y resumen bajo demanda para continuidad/retest — es capacidad de producto | APPROVED |
| D-035 | Un paquete TATACOA compatible recibido por una persona autorizada debe poder abrirse como proyecto continuable sin reescribir provenance ni confiar ciegamente en su contenido | APPROVED |
| D-036 | Sprint 03 debe avanzar dentro del alcance aprobado y detenerse solo ante bloqueo real, decisión humana/arquitectónica/de seguridad no resuelta o acción que requiera aprobación | APPROVED |
| D-037 | La validación integral de Encrypted v1 como producto se difiere hasta poder probarla dentro del flujo real de Usable Alpha; los PASS técnicos previos se conservan como evidencia | APPROVED |
| D-038 | RFC 3161 usa sidecar `.tsr`: Encrypted timestamp-ea los bytes exactos del archivo; Plain usa un digest raíz TATACOA versionado y formalmente especificado sobre manifest + objetos declarados. TSA solo explícita/configurada; su ausencia/fallo no invalida el bundle | APPROVED |
| D-039 | Continuidad usa persistencia explícita/versionada sin secretos y recuperación determinista; reanudar operaciones requiere nueva validación de autorización | APPROVED |
| D-040 | Importación aplica verify-before-trust, nunca ejecuta automáticamente y exige nueva validación de autorización antes de continuar operaciones | APPROVED |
| D-041 | Reapertura por perfil: LAB no exige autenticación adicional; PROFESSIONAL exige revalidación contextual antes de operar; HIGH_SENSITIVITY declara que el workspace local aún no tiene protección criptográfica persistente; CUSTOM niega capacidades de protección sin política. Reabrir nunca autoriza ejecutar | APPROVED |
| D-042 | RFC 3161 usa `x509-tsp 0.1`/RustCrypto estable, transporte síncrono `ureq 3` con `rustls >=0.23.45`, provider `ring`, WebPKI roots explícitos solo para TLS, HTTPS/timeout/TSA explícitos, sin redirects/proxy implícito. Confianza TLS y confianza TSA permanecen separadas | APPROVED |
| D-043 | El assurance RFC 3161 progresa `PRESENT → BOUND → SIGNATURE_VALID → TRUSTED → HISTORICALLY_VALIDATED`; checks usan `PASS/FAIL/NOT_EVALUATED/INDETERMINATE/UNSUPPORTED`. SP3-07 solo puede afirmar hasta `BOUND` mientras sigan abiertos los gates de algoritmos de firma y validación histórica | APPROVED/PARTIAL |
| D-032 | El roadmap es acumulativo: un sprint extiende la dirección del producto y no puede redefinir silenciosamente arquitectura/decisiones previas | APPROVED |
| D-033 | Capacidades futuras conocidas se mantienen como horizonte sin asignarlas automáticamente a un sprint ni tratarlas como implementadas | APPROVED |
| D-034 | El hardening de creación Encrypted amplía D-025 por Security Profile: LAB_LEARNING mínimo 12; PROFESSIONAL mínimo 14; HIGH_SENSITIVITY mínimo 16; PROFESSIONAL/HIGH_SENSITIVITY rechazan passwords evidentemente predecibles; CUSTOM permanece fail-closed. Se conserva máximo 1024 bytes, ausencia de reglas compositivas arbitrarias, ausencia de normalización Unicode silenciosa y compatibilidad de verify con passwords históricas | APPROVED |

## Decisiones pendientes de diseño/spike

- endurecimiento futuro de protección persistente del workspace más allá de la política de reapertura D-041;
- importación: permanecen abiertos materialización/formato, compatibilidad y versiones, estados, provenance recibido y condiciones exactas para convertir un paquete ya verificado en proyecto continuable; D-040 ya congela verify-before-trust, ausencia de autoejecución y revalidación de autorización;
- estrategia async solo si la necesidad lo exige;
- política de release signing;
- SBOM tooling;
- fuzzing toolchain.

Una decisión pendiente no autoriza implementación por inferencia.
