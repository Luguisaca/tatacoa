# Sprint 03 — registro de implementación

## Estado

**EN IMPLEMENTACIÓN — QA técnico parcial.** Este documento registra bloques realmente construidos; no declara completa la Usable Alpha.

## Bloque 01 — App API y flujo de trabajo

Rama local: `feat/sp3-01-app-api-workflow`.

Implementado:

- nuevo crate `tatacoa-app-api`, independiente de Tauri;
- operación tipada para crear el contexto inicial completo de un trabajo;
- ejecución contextualizada mediante el Core;
- listado validado de engagements, sessions y executions existentes;
- resumen derivado de estado persistido, sin inventar hechos ni duplicar reglas de dominio;
- reapertura de un workspace existente y reconstrucción del resumen desde disco.

El Core continúa validando IDs, relaciones, paths, aislamiento y captura. App API no accede directamente a JSON ni replica esas reglas.

QA técnico:

- flujo E2E de creación → ejecución → reapertura → resumen: PASS automatizado;
- regresión completa del workspace: PASS;
- Clippy con warnings como error: PASS.

QA humano propuesto cuando exista Desktop:

1. crear un trabajo completo sin copiar IDs;
2. ejecutar una herramienta autorizada;
3. cerrar la aplicación;
4. reabrir el workspace;
5. comprobar que session, execution, estado de captura y artifacts aparecen en el resumen.

Pendiente del siguiente bloque: Desktop/Tauri, integración visual y contratos de continuidad/importación que requieren diseño específico.

## Bloque 02 — Desktop E2E inicial

Rama local: `feat/sp3-02-desktop-e2e`, creada desde el bloque 01.

Implementado:

- nuevo crate `tatacoa-desktop` sobre Tauri 2, sin depender del binario CLI;
- interfaz local para abrir un workspace, listar trabajos, crear el contexto autorizado completo, consultar el resumen persistido y ejecutar una herramienta tras confirmación explícita;
- comunicación Desktop → `tatacoa-app-api` → Core mediante comandos y estructuras tipadas;
- CSP restrictiva, `connect-src 'none'`, capability limitada a la ventana principal y APIs Core predeterminadas;
- renderizado de datos no confiables mediante `textContent`/nodos de texto, sin interpolarlos como HTML;
- recurso de aplicación Windows y configuración de empaquetado Tauri.

QA técnico:

- sintaxis JavaScript: PASS;
- `cargo check --locked --workspace --all-targets`: PASS;
- Clippy del workspace con warnings como error: PASS;
- pruebas completas del workspace: PASS;
- build del workspace, incluyendo Desktop: PASS en Windows 11 x64.

QA humano propuesto:

1. abrir el Desktop y seleccionar un workspace local de prueba;
2. crear un trabajo y verificar que aparecen engagement y session sin copiar IDs;
3. ejecutar una herramienta inocua dentro del alcance autorizado;
4. cerrar y reabrir el Desktop y confirmar que el resumen se reconstruye desde disco.

Este bloque aún no cubre consulta del contenido de artifacts, Knowledge/Replay, exportación desde Desktop, continuidad explícita ni importación. Los contratos exactos de continuidad, protección de reapertura e importación siguen siendo decisiones pendientes documentadas y no se implementan por inferencia.

## Bloque 03 — capacidades de producto existentes en Desktop

Rama local: `feat/sp3-03-product-capabilities`, creada desde el bloque 02.

Implementado:

- detalle validado de cada execution y sus artifacts;
- vista previa de texto limitada a 1 MiB, calculada mientras se verifica tamaño y SHA-256 del artifact completo; si el objeto cambió después de la captura, el Core niega la vista previa;
- creación manual de Knowledge Cards con clasificación explícita de fuente y revisión humana opcional;
- creación de Replay Recipes con prerequisites, placeholders y límites de autorización, sin ejecución automática;
- exportación Plain o Encrypted v1 desde Desktop, gobernada por las políticas existentes del Security Profile;
- confirmación local de password Encrypted, limpieza inmediata de campos y request Rust sin `Debug`/`Clone` para reducir exposición accidental;
- el frontend continúa sin autoridad sobre IDs, relaciones, provenance, política de exportación, password policy o criptografía.

QA técnico:

- flujo App API ampliado: crear → ejecutar → verificar/leer artifact → crear Knowledge → crear Replay → exportar → reabrir/resumir: PASS;
- prueba negativa: artifact modificado después de captura no puede previsualizarse: PASS;
- sintaxis JavaScript, check y Clippy estricto: PASS.

QA humano propuesto:

1. abrir una execution y comparar stdout/stderr con la herramienta ejecutada;
2. crear una Knowledge Card como borrador y comprobar que no se presenta como Evidence validada;
3. crear una Replay Recipe con límites explícitos y comprobar que guardarla no la ejecuta;
4. probar Plain en cada perfil y confirmar los rechazos/acknowledgements esperados;
5. probar Encrypted v1, confirmar que los campos password se limpian y verificar el bundle con password correcta e incorrecta.

Límite siguiente: continuidad/recuperación, protección para reabrir trabajos e importación continuable tienen decisiones pendientes explícitas en `DECISIONS.md`. No se define su formato o política por inferencia.

## Bloque 04 — foundation de continuidad

Rama local: `feat/sp3-04-continuity-foundation`, creada desde el bloque 03.

Implementado:

- snapshots append-only `tatacoa.continuity.v1`, atómicos, explícitos y sin campos de secretos;
- estados ACTIVE/PAUSED, session actual y pendientes indicados por la persona;
- inspección determinista de capturas `.partial` y señal explícita de recuperación requerida;
- compatibilidad de lectura con engagements anteriores a continuidad;
- reanudación fail-closed si no se declara revalidación actual de autorización;
- controles Desktop para pausar, reconstruir el resumen persistido y reanudar con confirmación.

Pendiente deliberado: la protección/autenticación concreta para reabrir por Security Profile no está definida por las políticas actuales. No se infiere cifrado de workspace ni una credencial nueva.

## Bloque 05 — objeto de timestamp Plain

Rama local: `feat/sp3-05-timestamp-root`, creada desde el bloque 04.

Implementado:

- especificación byte a byte de `tatacoa.plain-root.v1`, sin canonicalización JSON implícita;
- cálculo determinista SHA-256 sobre `manifest.json` y cada objeto declarado, ordenados por path UTF-8;
- verificación de tamaño y digest declarados antes de producir el digest raíz;
- pruebas de determinismo y rechazo de artifact alterado.

Pendiente: transporte RFC 3161, parseo/verificación CMS/PKIX y sidecar `.tsr`. Incorporarlos requiere seleccionar y revisar dependencias mantenidas; no se implementa ASN.1/CMS o validación X.509 propia.
