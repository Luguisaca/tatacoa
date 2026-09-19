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
- WSL2, Kali y Parrot requieren QA separado. La CI propuesta cubre Windows y Ubuntu como señal temprana, no sustituye esos targets.

## Pruebas de desarrollo

Las pruebas automatizadas cubren el happy path, alteración de artifact, artifact faltante, límite/truncamiento, aislamiento entre engagements y rechazo de traversal/prefijos ambiguos. La matriz completa de QA permanece fuera del alcance de Codex para este Sprint.
