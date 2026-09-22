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

## Bloque 06 — hardening previo a RFC 3161

Rama local: `feat/sp3-06-pre-rfc-hardening`, creada desde el bloque 05.

Implementado:

- revisión contextual antes de ejecutar y antes de reanudar, derivada exclusivamente de Core/App API;
- presentación de engagement, perfil, scope, límite de autorización, environment, target, session, executable y argumentos efectivos;
- selectores nativos para workspace y destino de exportación mediante el plugin oficial de diálogo Tauri;
- continuidad ordenada por `revision: u64`; timestamps quedan como metadata observacional;
- rechazo de revisiones duplicadas y overflow; gaps son tolerables porque cada snapshot es autocontenido;
- compatibilidad legacy: si solo existen snapshots sin revisión, se usa una vez el orden histórico de sus nombres y la siguiente escritura crea revisión 1;
- advertencias explícitas de reapertura por Security Profile, sin autenticación ficticia ni cifrado de workspace;
- aclaración normativa: `SHA-256(root-encoding)` es directamente el `messageImprint` Plain, sin doble hash;
- documentación de importación corregida para separar D-040 de los detalles todavía abiertos.

Capabilities Tauri: permanecen en `core:default`. Los diálogos se exponen mediante dos comandos Rust acotados; el frontend no recibe permisos genéricos de filesystem ni diálogo.

## Bloque 07 — foundation funcional RFC 3161

Rama local: `feat/sp3-07-rfc3161-foundation`, creada desde el bloque 06.

Implementado en Core:

- solicitud DER v1 con `messageImprint` SHA-256, nonce CSPRNG de 128 bits y `certReq=true`;
- objeto Plain sin double-hash y Encrypted v1 sobre SHA-256 de los bytes exactos;
- TSA HTTPS explícita, `ureq 3`, `rustls >=0.23.45`, `ring`, roots WebPKI explícitos, timeout acotado, sin proxy ni redirects;
- límite de 4 MiB y `application/timestamp-reply` obligatorio;
- parseo DER → `TimeStampResp` → CMS `SignedData` → `TSTInfo` con RustCrypto;
- checks de status, tipos CMS/TSTInfo, SHA-256, `messageImprint` y nonce;
- sidecar `.tsr` atómico, `create_new`, solo cuando alcanza `BOUND`;
- verificación offline sin red y reporte por check, no booleano.

Límite deliberado: no afirma `SIGNATURE_VALID`, `TRUSTED` ni `HISTORICALLY_VALIDATED`. Firma CMS, ESSCertIDv2, certificado, path RFC 5280, EKU, trust TSA y revocación histórica permanecen `NOT_EVALUATED`/`INDETERMINATE` hasta resolver los gates abiertos. Los roots TLS no son trust anchors TSA.

Dependencias incorporadas: `x509-tsp 0.1.0`, `der 0.7.10`, `cms 0.2.3`, `x509-cert 0.2.5`, `ureq 3.4.2`, `rustls >=0.23.45` y provider `ring`. No se implementó ASN.1, CMS, X.509 ni criptografía propia.

QA humano pendiente: usar una TSA de laboratorio configurada conscientemente, confirmar creación del `.tsr`, ausencia de sidecar ante fallo/binding incorrecto y que ninguna interfaz presente `BOUND` como firma/trust válido.

## Bloque 08 — flujo de usuario RFC 3161

Rama local: `feat/sp3-08-timestamp-user-flow`, creada desde el bloque 07.

Implementado:

- operaciones tipadas App API para solicitar y verificar timestamps;
- comandos CLI `timestamp-request` y `timestamp-verify` con modo Plain/Encrypted explícito;
- `tatacoa-verify` acepta opcionalmente sidecar y modo, verifica primero el bundle y luego el timestamp sin red;
- Desktop permite seleccionar `.tsr`, configurar TSA HTTPS/timeout, solicitar y verificar offline;
- la exportación precarga bundle/modo/sidecar, pero no contacta una TSA automáticamente;
- presentación de assurance y checks completos; aviso visible de que `BOUND` no equivale a firma o trust.

QA humano propuesto:

1. exportar Plain y Encrypted desde Desktop;
2. confirmar que no ocurre tráfico TSA al exportar;
3. configurar voluntariamente una TSA de laboratorio y solicitar el `.tsr`;
4. verificarlo offline mediante Desktop, CLI y verifier;
5. alterar bundle o sidecar y confirmar fallo cerrado;
6. confirmar que los checks de firma/trust/histórico no aparecen como `PASS`.

Bloqueado deliberadamente para el siguiente incremento criptográfico: allowlist de algoritmos de firma y contrato de revocación/validación histórica.

## Bloque 09 — firma CMS/RFC 3161

Rama local: `feat/sp3-09-timestamp-signature`, creada desde el bloque 08.

Implementado en Core:

- contrato CMS completo para `SIGNATURE_VALID`: único `SignerInfo`, certificado inequívoco, atributos firmados, content-type, message-digest local, ESSCertIDv2 y firma criptográfica;
- allowlist D-044 con separación `FAIL`/`UNSUPPORTED` y validación estricta de parámetros RSA-PSS, RSA PKCS#1 v1.5 y ECDSA;
- `rustls-webpki 0.103.15`/`ring 0.17.14` para primitivas mantenidas; no se implementaron primitivas propias;
- persistencia de un sidecar solicitado solo cuando alcanza `SIGNATURE_VALID`;
- fixture local RSA/SHA-256 y pruebas negativas de tampering, parámetros, duplicados y algoritmos rechazados/no soportados.

Límite deliberado: `SIGNATURE_VALID` no implica confianza en la TSA. Path PKIX, anchor TSA explícito, validez al `genTime`, EKU y policy quedan para el siguiente bloque. `HISTORICALLY_VALIDATED` permanece cerrado y `historical_revocation` sigue `INDETERMINATE`.

QA humano propuesto: solicitar a una TSA explícita cuyo perfil esté en la allowlist, confirmar `SIGNATURE_VALID`; alterar el sidecar y confirmar `cms_signature=FAIL`; probar una TSA fuera de allowlist y confirmar `UNSUPPORTED`, nunca `TRUSTED`.

## Bloque 10 — confianza TSA explícita

Rama local: `feat/sp3-10-timestamp-trust`, creada desde el bloque 09.

Implementado:

- `TsaTrustPolicy` exige uno o más anchors DER y una o más policies OID aceptadas; no existe trust store TSA predeterminado;
- path PKIX validado con `rustls-webpki 0.103.15` al `genTime` del token, usando certificados CMS e intermedios DER explícitos;
- certificado firmante con EKU crítico y exclusivamente `id-kp-timeStamping`;
- separación de checks `policy`, `timestamping_eku` y `tsa_trust`; `TRUSTED` solo si los tres y `SIGNATURE_VALID` pasan;
- configuración explícita en Core, App API, CLI, verifier y Desktop; la verificación permanece offline;
- la allowlist Alpha se aplica también a firmas del path y conserva `UNSUPPORTED` separado de `FAIL`.

Límite deliberado: no se aportan CRL/OCSP, no se consulta red y no se afirma `HISTORICALLY_VALIDATED`; `historical_revocation=INDETERMINATE` incluso para un timestamp `TRUSTED`.

QA humano propuesto: convertir/exportar la cadena autorizada a DER, verificar con anchor y policy correctos y confirmar `TRUSTED`; repetir sin configuración (`SIGNATURE_VALID`), con policy ajena, anchor ajeno, certificado fuera de vigencia al `genTime`, EKU no crítico/no exclusivo y cadena incompleta, confirmando que nunca eleva a `TRUSTED`.

## HUMAN QA — hallazgos acumulativos

### HUMAN-QA-01 — Desktop sin wiring IPC al iniciar

Estado: **FIX APLICADO — REVALIDACIÓN HUMANA PENDIENTE**.

Durante QA humano acumulativo sobre SP3-10, Desktop renderizó correctamente pero los controles iniciales no ejecutaban acciones y no llegaban invocaciones al backend Rust.

Causa confirmada: el frontend Vanilla JS usa `window.__TAURI__.core.invoke`, mientras `app.withGlobalTauri` no estaba habilitado en `tauri.conf.json`. En Tauri 2 esa exposición global está deshabilitada por defecto.

Corrección aplicada en la misma rama SP3-10:

- habilitar `app.withGlobalTauri: true`;
- conservar CSP, capabilities y comandos existentes sin ampliar permisos;
- no modificar Core, App API, RFC 3161 ni reglas de seguridad/dominio.

Revalidación humana requerida: reiniciar Desktop y confirmar que `Seleccionar carpeta…` abre el diálogo nativo, que `Abrir workspace` alcanza el backend y que el flujo mínimo puede continuar. Este hallazgo no se considera PASS hasta completar esa comprobación.


### HUMAN-QA-02 — Security Profile incompatible entre Desktop y App API

Estado: **FIX APLICADO — REVALIDACIÓN HUMANA PENDIENTE**.

Durante la creación del primer trabajo del recorrido humano documentado, Desktop alcanzó correctamente `create_work`, pero envió `LabLearning` mientras el contrato serializado de `SecurityProfile` acepta `LAB_LEARNING`, `PROFESSIONAL`, `HIGH_SENSITIVITY` o `CUSTOM`. La operación fue rechazada antes de crear el trabajo.

Corrección aplicada en SP3-10:

- alinear los valores del selector Desktop con el contrato serializado existente: `LAB_LEARNING`, `PROFESSIONAL` y `HIGH_SENSITIVITY`;
- no modificar la enum, políticas ni autoridad del Core/App API;
- añadir una regresión Desktop que comprueba los valores públicos del selector y rechaza las variantes obsoletas observadas durante QA.

Revalidación humana requerida: repetir el formulario documentado de creación LAB_LEARNING sobre un workspace de QA nuevo/no inicializado y confirmar que el contexto completo se crea y aparece en el resumen. El hallazgo no se considera PASS hasta completar esa comprobación.


### HUMAN-QA-03 — revisión contextual no visible antes de ejecutar

Estado: **FIX APLICADO — REVALIDACIÓN HUMANA PENDIENTE**.

Durante QA humano, `Revisar y ejecutar` no presentó de forma observable la revisión contextual antes de alcanzar el intento de ejecución. El frontend dependía de `window.confirm()` como único gate visual.

Corrección aplicada en SP3-10:

- reemplazar el confirm nativo del navegador para ejecución por una revisión persistente dentro de la UI de TATACOA;
- mostrar Engagement, Security Profile, Scope, límite autorizado, Environment, Target, Session, executable y argumentos efectivos antes de habilitar la acción final;
- separar explícitamente `Revisar antes de ejecutar` de `Confirmar y ejecutar`, con opción de cancelar;
- conservar App API/Core como autoridad y no mover reglas de autorización al frontend;
- añadir regresión Desktop que exige la presencia del gate explícito.

Revalidación humana requerida: pulsar `Revisar antes de ejecutar`, comprobar que todavía no existe nueva execution, revisar todos los campos mostrados y solo entonces pulsar `Confirmar y ejecutar`.

### HUMAN-QA-04 — executable bare-name no resuelto en Desktop Windows

Estado: **EN INVESTIGACIÓN — SIN CAMBIO DE SEGURIDAD/RESOLUCIÓN**.

En el mismo recorrido, `rustc` devolvió `program not found` desde `std::process::Command`, aunque otra PowerShell del usuario resolvió `C:\\Users\\Inarix\\.cargo\\bin\\rustc.exe` y `rustc --version` correctamente.

La implementación Core continúa ejecutando `Command::new(executable).args(argv)` sin shell y no modifica el entorno. No se introduce `cmd.exe`, shell implícito, búsqueda propia ni mutación de PATH para ocultar el hallazgo.

Siguiente discriminación humana: repetir el gate ya corregido usando la ruta absoluta verificada de `rustc.exe`. Si la ruta absoluta ejecuta correctamente, clasificar el fallo bare-name como diferencia del entorno heredado por el proceso Desktop y diseñar por separado la UX/resolución permitida; si también falla, investigar el spawn de Windows antes de modificar Core.


### HUMAN-QA-05 — integración Desktop expone primitivas internas en vez del flujo de trabajo

Estado: **FAIL DE PRODUCTO/UX — CORRECCIÓN IMPLEMENTADA; REVALIDACIÓN HUMANA PENDIENTE**.

El QA humano acumulativo demostró que Execution/Artifact funcionan, incluida revisión contextual explícita, ejecución sin shell mediante ruta absoluta, captura COMPLETE y consulta de stdout/stderr. Sin embargo, al continuar hacia Knowledge/Replay/Export/Timestamp, Desktop presenta capacidades del dominio principalmente como formularios independientes.

Esto no satisface por sí solo el objetivo aprobado de SP3: una GUI orientada al flujo real de una pentester y no a exponer métodos internos del Core.

Hallazgos concretos:

- Knowledge Card exige transcribir manualmente información que TATACOA ya conoce de la Execution/contexto;
- tras intentar guardar Knowledge, el feedback no fue visible/localizable para la persona y la UI no ofrece una forma clara de consultar el resultado asociado;
- Replay se presenta como construcción manual separada aunque executable, argv, contexto y Execution de origen ya están registrados;
- la composición actual aumenta trabajo administrativo en lugar de acompañar ejecución, evidencia, aprendizaje, continuidad y retest.

Decisión humana durante QA:

- **no eliminar** Knowledge/Replay ni degradar Core;
- detener el QA integral de la UI actual en este punto;
- conservar los PASS técnicos y humanos ya obtenidos;
- corregir la capa de producto Desktop según el contrato de experiencia añadido a `SPRINT-03-PLAN.md`;
- distinguir datos automáticos, asistencia opcional y decisiones explícitas de seguridad/autorización;
- reanudar QA integral cuando el flujo corregido permita trabajar sin transcribir información ya conocida por TATACOA.

La corrección no autoriza IA local, reporting avanzado, auto-exploit, inferencia de autorización ni promoción automática a Evidence. Tampoco convierte Knowledge factual generado automáticamente en contenido validado.

Corrección implementada incrementalmente en `feat/sp3-11-execution-workflow`:

- `tatacoa-core` expone la lectura validada ya existente de Knowledge y Replay asociados, sin cambiar sus modelos, validadores ni autoridad;
- `tatacoa-app-api` compone una vista de trabajo de la Execution con manifest, artifacts, Knowledge y Replay asociados;
- Knowledge desde Execution deriva únicamente hechos registrados (`Execution`, adapter, artifacts, executable, argv, shell, estado de captura y exit code) y conserva como entrada humana relevancia, objetivo, interpretación, límites, validación, contexto defensivo y fuentes;
- Replay/Retest reutiliza por defecto executable, argv, contexto tipado y `source_execution_id`; placeholders, secretos, prerequisites y límites de autorización siguen siendo decisiones humanas, y cualquier override de invocación es deliberado;
- Desktop presenta esos elementos como un recorrido continuo, mantiene feedback visible después de guardar y lista los objetos creados dentro de la Execution;
- las ayudas visibles se ajustan al perfil vigente sin introducir política nueva: explicación adicional en `LAB_LEARNING`, menor fricción en `PROFESSIONAL` y minimización de datos adicionales en `HIGH_SENSITIVITY`;
- no existe promoción automática de artifacts: las pruebas confirman que permanecen en `CAPTURED`; crear una receta no la ejecuta.

QA técnico de la corrección:

- pruebas App API del flujo local end-to-end: PASS;
- regresión Desktop sobre reutilización de hechos, ausencia de campos redundantes, feedback y localización: PASS;
- sintaxis JavaScript (`node --check`): PASS;
- `cargo fmt --check`: PASS;
- `cargo check --locked --workspace --all-targets`: PASS;
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: PASS;
- `cargo test --locked --workspace`: PASS (54 PASS, 3 helpers/benchmarks ignorados deliberadamente);
- `cargo build --locked --workspace`: PASS usando `target/sp3-11-qa`; la salida predeterminada estaba bloqueada por una instancia Desktop abierta y no se cerró el proceso del usuario.

Revalidación humana requerida — HUMAN-QA-05 permanece FAIL hasta completarla:

1. abrir una Execution ya capturada y confirmar que contexto, executable, argv, resultado y artifacts aparecen sin transcripción;
2. guardar Knowledge introduciendo solo interpretación y fuente, comprobar el feedback visible y localizar la tarjeta en la misma Execution;
3. preparar Replay/Retest sin modificar la invocación, comprobar que reutiliza executable/argv y queda listado sin ejecutarse;
4. activar la modificación deliberada, comprobar que se distingue del origen y que placeholders/prerequisitos/límites siguen bajo decisión humana;
5. confirmar que ningún artifact fue promovido automáticamente desde `CAPTURED`.

No se continuó Export/Encrypted/RFC 3161 en este bloque, conforme al gate de revalidación solicitado.

## Bloque 12 — foundation de asistencia offline y nota profesional

Rama `feat/sp3-12-offline-tool-assistance`, desde SP3-11. Estado: **implementado; QA humano pendiente**. HUMAN-QA-05 continúa **FAIL de producto/UX** hasta que Luis revalide el recorrido.

- Core agrega un contrato de asistencia read-only: hechos observados con referencia al campo del manifest/artifact, documentación local opcional con localizador y digest del contenido, y adapters opcionales. Ninguno ejecuta probes ni modifica Execution, RAW, Evidence o provenance. Sin proveedor/adaptor registrado, `UNAVAILABLE` es explícito y la captura genérica conserva su operatividad Windows/Linux.
- App API expone asistencia genérica y una acción de nota breve. La nota se persiste como Knowledge Card existente en `DRAFT`, fuente `OPERATOR_NOTE`, con campos no evaluados marcados explícitamente; no infiere vulnerabilidades ni valida Evidence. La ficha estructurada anterior permanece disponible de forma opcional.
- Desktop presenta primero hechos observados y estado de ayuda documental; después ofrece añadir nota profesional. Replay/Retest conserva reutilización de origen y override deliberado. `GENERIC/DOCUMENTED/ADAPTED` no se presentan como confianza o garantía de herramienta.
- No se añadieron dependencias ni se registró un catálogo cerrado. Los proveedores/adapters de prueba son fixtures, no soporte de producción. No se asume ningún flag de help/version universal.

QA técnico SP3-12 en Windows 11 x64: `cargo fmt --check`, `cargo check --locked --workspace --all-targets`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace`, `cargo build --locked --workspace --target-dir target/sp3-11-qa` y `node --check` del frontend: **PASS**. La salida aislada evita reemplazar una instancia Desktop abierta. Las pruebas cubren proveedor local válido, fuente faltante → `UNAVAILABLE`, ausencia de adapter/probe, nota vacía rechazada, Knowledge `DRAFT` con `OPERATOR_NOTE`, manifest sin cambios y Evidence `CAPTURED`. Linux no se ejecutó en este host: solo está instalado el target Rust `x86_64-pc-windows-msvc`; el contrato usa `std` multiplataforma y requiere QA Linux posterior.

QA Windows propuesto: ejecutar/capturar una herramienta de prueba mediante ruta absoluta; abrir la Execution y comprobar hechos con fuentes `manifest.*`, estado documental `UNAVAILABLE` y ausencia de probes/red; guardar una nota breve y localizarla como Knowledge `DRAFT`; verificar que artifacts permanecen `CAPTURED`; preparar Replay sin ejecutarlo y comprobar origen/override explícito. Repetir con un executable desconocido permitido por el scope: la asistencia debe continuar `GENERIC`, nunca impedir la captura.

## Bloque 13 — recorrido Desktop y continuidad utilizable

Rama `feat/sp3-13-desktop-workflow-continuity`, desde SP3-12. **Implementado; QA humano pendiente.**

- navegación por trabajo, continuidad, ejecución, resultados y entrega; el resumen presenta actividad reciente, pendientes, última Execution y recuperación requerida sin obligar a reconstruir IDs;
- sesión de continuidad recuperada cuando existe y contexto autorizado visible al elegirla; la revisión antes de ejecutar sigue usando App API/Core;
- la ejecución recién capturada abre directamente su resultado, sin buscarla en una tabla;
- artifacts, notas, hechos y asistencia muestran primero información de tarea; IDs, hashes, rutas, fuente técnica y provenance quedan en detalles accesibles;
- reanudación pasa por una revisión contextual persistente con cancelación/confirmación, sin sustituir la revalidación de Core;
- feedback de creación, captura, pausa y reanudación queda en el contexto de trabajo. No se cambian políticas ni estados de Evidence.

QA técnico de frontera: sintaxis JavaScript, `cargo fmt --check`, `cargo check --locked --workspace --all-targets`, Clippy estricto y pruebas de App API/Desktop: PASS. HUMAN-QA-05 sigue FAIL hasta recorrido humano completo. Los bloques siguientes integran Replay y entrega sin alterar criptografía.

## Bloque 14 — preparación de Replay/Retest desde Execution

Rama `feat/sp3-14-replay-retest-flow`, desde SP3-13. **Implementado; QA humano pendiente.**

- la preparación normal no exige transcribir executable, argv, contexto, placeholders, prerequisitos ni límites adicionales: App API reutiliza invocación y, cuando no se indican límites adicionales, el límite de autorización del Scope registrado; el Core conserva la exigencia de límites explícitos en toda receta;
- las decisiones no triviales permanecen en opciones avanzadas, incluido override deliberado; parsing de booleanos de placeholders rechaza valores ambiguos;
- cada receta queda visible junto a su Execution y ofrece “usar como base para retest”; ello solo prepara el formulario de ejecución, no ejecuta la herramienta. El gate de revisión contextual vuelve a mostrar el Scope actual, límites/prerequisitos de receta y cualquier cambio de invocación antes de la confirmación final;
- recetas con placeholders o argumentos que el editor no puede representar exactamente no se convierten silenciosamente en una ejecución; se reporta el límite;
- comparación read-only entre executions del mismo engagement muestra estado, exit code y diferencias de digest/tamaño registrados sin deducir vulnerabilidad, reproducción ni Evidence validada. La relación de comparación no se persiste como nuevo modelo de retest.

QA de frontera: `node --check`, `cargo fmt`, check, Clippy estricto y tests App API/Desktop: PASS. La prueba negativa incluye aislamiento entre engagements para asistencia y creación de Replay. HUMAN-QA-05 continúa FAIL hasta revalidación humana acumulativa.

## Bloque 15 — entrega guiada y timestamp explícito

Rama `feat/sp3-15-guided-export-timestamp`, desde SP3-14. **Implementado; QA humano pendiente.**

- App API expone orientación descriptiva de exportación derivada exclusivamente de las funciones de política de Core. La exportación real vuelve a autorizar en Core. En LAB_LEARNING el modo inicial es Plain; en PROFESSIONAL y HIGH_SENSITIVITY es Encrypted; CUSTOM no ofrece exportación sin política aprobada. Plain en PROFESSIONAL exige reconocimiento explícito y HIGH_SENSITIVITY lo prohíbe.
- Desktop presenta el destino y protección dentro de la entrega de la Execution activa, conserva feedback del bundle sin borrar el resultado capturado ni resetear el modo y limpia inputs de password tras el intento.
- El sidecar RFC 3161 permanece opcional. Exportar no contacta TSA; solicitar exige URL explícita. La verificación es offline y separada. El resultado visible nombra el assurance reportado, y los checks/policies/anchors de confianza siguen disponibles en detalles técnicos. Ausencia o fallo de TSA no invalida el bundle; `HISTORICALLY_VALIDATED` continúa bloqueado.
- Pruebas negativas: aislamiento entre engagements para orientación de exportación; Desktop no habilita modo prohibido, no inicia TSA en exportación y no borra el resultado de Execution. El contrato criptográfico/formatos no se modificó.

QA de frontera: `node --check`, `cargo fmt --check`, check, Clippy estricto y pruebas App API/Desktop: PASS. HUMAN-QA-05 y Sprint 03 **no** se declaran PASS; se requiere el guion humano integral de producto antes de cualquier integración.

## HUMAN QA — baseline de clientes limpios y gate de distribución (2026-09-22)

Estado: **BASELINE DE ENTORNO PASS / INSTALACIÓN DE CLIENTE NUEVO NOT TESTED — BLOCKED POR ARTEFACTO DISTRIBUIBLE AUSENTE**.

Se prepararon clientes separados del entorno de desarrollo para validar posteriormente la distribución real de la Usable Alpha:

- Windows 10 Pro x64, build 19045, con Microsoft Edge WebView2 Runtime presente; Git presente; Rust/Cargo/Node/npm ausentes. Python está disponible como utilidad local de transferencia/QA y no constituye dependencia aprobada de TATACOA.
- Parrot Security 7.3 x86_64, KDE/Wayland, WebKitGTK 4.1 presente; Git, Node/npm y Python presentes; Rust/Cargo ausentes.
- Kali GNU/Linux Rolling 2026.3 x86_64, XFCE/X11, WebKitGTK 4.1 presente; Git presente; Rust/Cargo/Node/npm ausentes.

Los tres entornos superaron el baseline de preparación aplicable. Esto no equivale a compatibilidad funcional de TATACOA porque todavía no se instaló ni ejecutó en ellos un candidato distribuible.

La implementación funcional continuó posteriormente desde SP3-12 mediante SP3-13, SP3-14 y SP3-15. Sus PASS técnicos permanecen válidos, pero no sustituyen HUMAN-QA-05 ni la validación humana integral de Sprint 03.

Por decisión de Validación Humana:

- no instalar toolchains para sustituir el artefacto faltante;
- no clonar/compilar el repositorio en el cliente limpio para declarar artificialmente PASS de distribución;
- preservar los clientes preparados para probar el artefacto real;
- tratar la ausencia del artefacto como gate de distribución BLOCKED, sin invalidar los PASS técnicos anteriores.

Siguiente condición: producir un candidato distribuible trazable al commit probado para los targets aprobados, registrar su SHA-256 y dependencias/runtime reales y someterlo a instalación o despliegue, primer arranque, recorrido funcional, cierre/reapertura y desinstalación cuando aplique.

La política de firma de release continúa pendiente y no se considera resuelta por este bloque.

HUMAN-QA-05 y Sprint 03 permanecen pendientes de Validación Humana.

## Bloque 16 — candidato portable Windows para HUMAN QA

Rama `codex/sp3-16-windows-distribution`, desde `integrate/sp3-15-human-qa-handoff` (`a4a5c9f`). Se incorpora `scripts/package-windows-alpha.ps1` para empaquetar, sin publicar, binarios release Windows x64 de Desktop y CLI en ZIP separados. Cada paquete incluye licencia, aviso, instrucciones de uso y commit de origen; `SHA256SUMS.txt` registra hashes. El script exige árbol limpio, no reemplaza un destino existente y no llama a servicios remotos.

El ZIP Desktop es una modalidad portable de despliegue para un cliente con Microsoft Edge WebView2 Runtime. No requiere CLI, Rust ni Node preinstalados. No se afirma que sea instalador firmado; la política de firma de release permanece pendiente. La prueba de instalación/despliegue, primer arranque, flujo funcional, cierre/reapertura y retiro en Windows 10 limpio sigue reservada para HUMAN QA. Linux Kali/Parrot requieren artefactos nativos construidos y comprobados en Linux; el único target Rust instalado en este host es `x86_64-pc-windows-msvc`.

QA técnico del código fuente antes del empaquetado: `cargo fmt --check`, `cargo check --locked --offline --workspace --all-targets`, `cargo clippy --locked --offline --workspace --all-targets -- -D warnings` y `cargo test --locked --offline --workspace`: PASS. El build release y los SHA-256 del candidato quedan registrados en el reporte de distribución local. Estos controles no cierran HUMAN-QA-05, Encrypted v1 como experiencia ni el QA integral SP3.

La importación continuable de bundles sigue bloqueada por la decisión de materialización/formato, compatibilidad, estados y provenance recibido enumerada en `DECISIONS.md`. D-040 ya impone verify-before-trust y revalidación de autorización, pero no resuelve ese contrato. No se implementa por inferencia. La validación histórica RFC 3161 también permanece en gate.

## Bloque 17 — decisiones humanas de cierre técnico

Rama `codex/sp3-17-approved-contracts`, desde SP3-16. La Validación Humana cerró el contrato de importación conservando paquete y provenance original, sin promoción automática de Evidence; excluyó `HISTORICALLY_VALIDATED` de esta Alpha; y definió modalidades Windows Desktop portable + instalable, Linux Desktop `.deb` + AppImage y CLI separado. `DECISIONS.md`, arquitectura, plan y estado reflejan aprobación, no implementación. QA documental: `git diff --check` PASS. Esta decisión sustituye el bloqueo de diseño señalado arriba, sin alterar el registro histórico del bloque 16.

## Bloque 18 — empaquetado Linux local reproducible

Rama `codex/sp3-18-linux-distribution`, desde SP3-17. `scripts/package-linux-alpha.sh` compila en Linux x64 desde lockfile sin red y prepara Desktop `.deb`, Desktop AppImage cuando existe `appimagetool`, y CLI separado `.tar.gz`. El host WSL2 de desarrollo dispone de Rust, `dpkg-deb` y WebKitGTK 4.1; `appimagetool` no está presente. Los clientes Kali/Parrot permanecen limpios y solo recibirán artefactos, nunca toolchain. Los resultados de build y SHA-256 se registran separadamente; la ausencia de AppImage impide afirmar completo el set Linux.
