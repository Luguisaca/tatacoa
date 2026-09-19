# Sprint 01 — implementación Alpha Foundation

## Estado

La primera vertical funcional está implementada en Rust para validación de desarrollo:

`ENGAGEMENT → CONTEXT → EXECUTION → ARTIFACT → SHA-256 → MANIFEST → EXPORT → VERIFY`

Este estado no equivale a QA final, certificación, garantía forense ni disponibilidad para producción.

## Componentes

- `tatacoa-core`: IDs tipados, engagement aislado, ejecución genérica, captura RAW, SHA-256, manifest y export Plain.
- `tatacoa-cli`: creación de engagement, ejecución/export y verificación integrada.
- `tatacoa-verifier`: biblioteca y binario independiente `tatacoa-verify`, offline y read-only respecto al bundle.

El Generic Execution Adapter usa `std::process::Command` con `executable + argv`. No introduce un shell implícito. `stdout` y `stderr` se drenan concurrentemente y se escriben en archivos sin acumular el output completo en memoria. El límite opcional `--max-stream-bytes` conserva hasta ese número de bytes por stream, continúa drenando el proceso y declara `TRUNCATED`.

## Uso mínimo

Compilar:

```text
cargo build --locked --workspace
```

Crear engagement:

```text
tatacoa engagement-create --workspace <workspace> --name <nombre>
```

Ejecutar y exportar un bundle Plain:

```text
tatacoa run --workspace <workspace> --engagement <eng_id> --bundle <bundle> -- <executable> <arg1> <arg2>
```

El separador `--` hace explícita la frontera entre argumentos de TATACOA y la invocación capturada.

Verificar offline:

```text
tatacoa-verify <bundle>
```

El verificador termina con código `0` para un bundle válido, `1` cuando detecta una discordancia de integridad y `2` cuando el bundle o manifest no se puede validar estructuralmente.

## Escritura y manifest

Los streams se escriben primero como archivos parciales exclusivos. Después se vacían, sincronizan, renombran como RAW final, se leen para calcular tamaño y SHA-256 y, al final, se publica el manifest mediante archivo temporal y rename.

El schema Alpha es `tatacoa.alpha.v1`. Registra:

- engagement y perfil de seguridad;
- ejecución, adapter, executable y argv separados;
- shell implícito desactivado;
- timestamps observacionales Unix y duración monotónica;
- exit status;
- estado de captura;
- artifacts RAW con ID lógico, rol, tamaño, path relativo y SHA-256.

Los timestamps son observaciones del host, no trusted timestamps. SHA-256 demuestra concordancia con el digest registrado; no demuestra autoría, firma ni cadena de custodia legal.

## Dependencias directas

| Crate | Uso | Motivo |
|---|---|---|
| `clap` | CLI tipada | Evita parsing manual ambiguo de subcomandos y argumentos. |
| `serde` | Modelo serializable | Contrato JSON tipado y campos desconocidos rechazados. |
| `serde_json` | Manifest/reportes JSON | Implementación mantenida del formato aprobado. |
| `sha2` | SHA-256 incremental | Implementación mantenida de RustCrypto; no se implementa criptografía propia. |
| `uuid` | IDs lógicos UUID v4 con prefijo | Reduce colisiones entre procesos y plataformas usando aleatoriedad del SO. |

No se añadió runtime async, framework, base de datos, crate de tiempo ni cifrado.

## Controles implementados

- IDs con prefijos `eng_`, `exe_` y `art_`;
- directorios separados por engagement y validación de referencias cruzadas;
- creación exclusiva para evitar sobrescritura silenciosa;
- paths de artifacts restringidos a `objects/<artifact-id>.bin`;
- rechazo de paths absolutos, traversal, prefijos Windows, separadores ambiguos y symlinks/reparse points observables;
- límite de 2 MiB para JSON hostil;
- schema, relaciones, IDs duplicados, algoritmo y formato de digest validados;
- hashing y copia por streaming;
- verificación sin ejecutar ni modificar artifacts;
- errores explícitos y sin `unsafe` en código propio.

## Limitaciones conocidas

- Solo existe export `PLAIN`. El cifrado continúa bloqueado por el spike criptográfico aprobado; no hay downgrade silencioso.
- La defensa de filesystem usa validación, `symlink_metadata` y canonicalización. Existe una ventana TOCTOU entre comprobación y apertura; Alpha todavía no usa APIs de handles específicas de cada SO.
- Si una captura o export falla antes del commit pueden permanecer archivos/directorios `.partial`; nunca se presentan como captura completa ni se reutilizan silenciosamente.
- El manifest usa structs con orden estable, pero todavía no define canonical JSON criptográfico.
- La foundation de Knowledge/Replay se limita a directorios reservados y vínculos de ejecución; no incluye automatización ni validación por IA.
- Provenance en este vertical cubre la relación directa `Execution → RAW Artifact`; el DAG de derivaciones avanzadas queda fuera de este incremento.
- El workspace no está cifrado y no se implementa `Encrypted` ni `Public/Sanitized`.
- WSL2/Ubuntu fue validado manualmente como target de integración Linux temprana. Kali y Parrot requieren QA separado. WSL2 no sustituye esos targets.

## Pruebas de desarrollo

Las pruebas automatizadas cubren el happy path, alteración de artifact, artifact faltante, límite/truncamiento, aislamiento entre engagements y rechazo de traversal/prefijos ambiguos.

## QA ejecutado — Windows 11 y WSL2/Ubuntu

Validación humana ejecutada sobre la rama `feat/sprint-01-alpha-foundation`. Los resultados describen únicamente los entornos realmente probados y no sustituyen el QA pendiente en Kali o Parrot.

### Windows 11 x64

- toolchain Rust 1.98.1;
- `cargo fmt --all -- --check`, `cargo check --workspace`, Clippy con warnings como error, tests y build: PASS;
- vertical funcional completa: engagement, ejecución, captura RAW, manifest, bundle Plain y verificación: PASS;
- verificador integrado e independiente sobre bundle válido: PASS;
- detección de tampering, artifact faltante, traversal y referencias semánticas inválidas: PASS;
- truncamiento explícito: PASS;
- schema no soportado y manifests estructuralmente inválidos rechazados sin panic: PASS.

### WSL2 — Ubuntu 24.04.5 LTS x86_64

Entorno observado: WSL2, kernel `6.18.33.2-microsoft-standard-WSL2`, usuario no-root y Rust 1.98.1 mediante rustup.

- fresh clone y checkout de la rama Alpha: PASS;
- format/check/Clippy/tests/build: PASS;
- superficie CLI y ayuda de `engagement-create`, `run` y `verify`: PASS;
- ejecución real con stdout/stderr y bundle Plain: PASS;
- verificador integrado e independiente sobre bundle válido: PASS;
- alteración de artifact detectada por ambos verificadores: PASS;
- sustitución hostil de artifact por symlink hacia fuera del bundle: rechazada por ambos verificadores;
- truncamiento real con `--max-stream-bytes 16`: PASS; el bundle truncado permanece íntegro y verificable;
- dos engagements independientes producen evidencia separada; el test E2E `cross_engagement_manifest_is_rejected` confirma el rechazo de referencias cruzadas;
- fallo de spawn de un ejecutable inexistente: error explícito, exit distinto de cero, sin panic y sin bundle residual;
- portabilidad bidireccional: bundle producido en Windows verificado en Linux y bundle producido en Linux verificado en Windows, con verificador integrado e independiente: PASS.

### Pendientes antes de cerrar QA Alpha

- QA específico en Kali Linux x64;
- QA específico en Parrot OS x64;
- parallel sessions/concurrencia manual;
- malicious/huge output más allá del caso controlado de truncamiento;
- revisión explícita de secret boundary, zero telemetry y AI isolation contra implementación y documentación;
- recorder/adapter failure adicionales cuando exista una superficie distinta del Generic Execution Adapter;
- redaction/provenance cuando esas capacidades entren en scope.

WSL2 es evidencia de integración Linux temprana; no se declara como sustituto de Kali ni Parrot.
