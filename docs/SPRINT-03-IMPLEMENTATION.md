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
