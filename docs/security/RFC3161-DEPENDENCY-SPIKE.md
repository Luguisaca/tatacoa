# RFC 3161 — spike de dependencias

## Estado

**GATE DE FIRMA APROBADO — INTEGRACIÓN EN SP3-09.** Investigación iniciada el 2026-09-20. Validación Humana aprobó el stack RustCrypto estable, `ureq 3`, `rustls >=0.23.45`, `ring` y la allowlist D-044. El gate de validación histórica/revocación continúa abierto.

Toolchain TATACOA: Rust 1.98. Todas las opciones listadas declaran un MSRV compatible. Las fuentes primarias consultadas fueron metadata crates.io, documentación/repositorios upstream RustCrypto, rustls, reqwest/ureq y RustSec.

## Matriz

| Crate | Versión evaluada | Función posible | Mantenimiento / licencia / MSRV | Dependencias relevantes | Qué aporta | Qué no verifica / riesgos |
|---|---:|---|---|---|---|---|
| `x509-tsp` | 0.1.0 | Estructuras DER de `TimeStampReq`, `TimeStampResp` y `TSTInfo` | RustCrypto; Apache-2.0 OR MIT; 1.65 | `der 0.7`, `cms 0.2`, `cmpv2` | Codificación/decodificación tipada RFC 3161 | No verifica firma CMS, cadena, EKU, revocación, nonce ni política por sí solo; línea 0.1 y stack RustCrypto anterior |
| `der` | 0.7.10 / 0.8.2 | DER estricto | RustCrypto activo; Apache-2.0 OR MIT; 0.8.2 requiere 1.85 | `const-oid`, derives opcionales | Parser/encoder DER mantenido | No conoce semántica TSP/CMS/PKIX. `x509-tsp 0.1` exige 0.7; mezclar 0.8 duplica stack |
| `cms` | 0.2.3 / 0.3.0-pre.2 | Parsear CMS `SignedData` del token | RustCrypto; Apache-2.0 OR MIT; pre-release 0.3 requiere 1.85 | `der`, `spki`, `x509-cert`; builders opcionales amplían criptografía | Representación CMS y atributos | Parsear no equivale a verificar firma/certificado; 0.3 es pre-release y no coincide con `x509-tsp 0.1` |
| `x509-cert` | 0.2.x / 0.3.0 | Certificados y extensiones RFC 5280 | RustCrypto; Apache-2.0 OR MIT; 0.3 requiere 1.85 | `der`, `spki`, `signature` opcional | Parseo X.509 y acceso a EKU/extensiones | No ofrece por sí solo un validador completo de path/revocación/política TSA |
| `rustls` | **>=0.23.45** | TLS del transporte HTTP | Activo; Apache-2.0 OR ISC OR MIT; 1.71 | provider `ring` o `aws-lc-rs`, `rustls-webpki` | TLS y autenticación del servidor configurado | TLS no valida el token RFC 3161. Versiones 0.23.13–0.23.44 están afectadas por RUSTSEC-2026-0285 |
| `rustls-webpki` | **0.103.15 (mínimo aprobado 0.103.13)** | Firma y futura validación PKIX TSA, además del uso transitivo TLS | Activo; ISC; 1.71 | provider `ring` | Verificación de firmas y path validation | Requiere configuración TSA explícita; no aporta por sí solo política RFC 3161 ni validación histórica/revocación |
| `reqwest` | 0.12.24 o 0.13.5 | POST `application/timestamp-query` y límites HTTP | Activo; MIT OR Apache-2.0; 1.64/1.85 | Tokio, Hyper, rustls opcional | Cliente maduro, timeouts y límites | Superficie grande e introduce runtime async incluso usando API blocking. Debe usar `default-features=false` y rustls explícito; `default-tls` queda prohibido |
| `ureq` | 3.4.2 | Transporte HTTP síncrono acotado | Activo; MIT OR Apache-2.0; 1.85 | rustls opcional, `webpki-roots` opcional | Menor superficie y evita decidir async runtime | No aporta TSP/CMS; roots y provider deben configurarse explícitamente; redirects/proxy deben restringirse |
| `tsp-http-client` | 0.1.0 | Construcción + envío TSP | Actividad limitada; MPL-2.0; MSRV no declarado | `ureq`, `x509-tsp`, `cms`, `der`, `chrono`, `rand` | Solicita y serializa una respuesta | Upstream declara que **no verifica la firma**; ejemplo usa TSA HTTP fija; licencia y política de transporte requieren revisión. No recomendado como autoridad |
| `trackone-rfc3161` | 0.2.0-beta.1 | Verificación RFC 3161/5816 especializada | Beta, específica de TrackOne; MIT; 1.93 | RustCrypto + proceso externo OpenSSL | Verificación estricta con pin, CRLs y tiempo histórico en su perfil | Invoca `openssl` externo, exige perfil VTL/pin/CRLs y no es una biblioteca general portable para TATACOA |
| `openssl` | 0.10.81 | Alternativa CMS/X.509/PKIX nativa | Activo; Apache-2.0; 1.80 | OpenSSL del sistema o vendorizado | Primitivas y validación madura disponibles en OpenSSL | Binding/ABI y empaquetado Windows/Linux; APIs TSP no necesariamente cubiertas de extremo a extremo; vendorizado amplía build/supply chain |

## Advisories y mantenimiento

- RustSec RUSTSEC-2026-0285 exige `rustls >=0.23.45` para la rama estable 0.23.
- Para `rustls-webpki` se toma como piso aprobado 0.103.13; SP3-09 fija 0.103.15 estable con features `ring` y `std`. No se adopta el prerelease 0.104.
- No se identificó en la consulta manual un advisory específico vigente para `der`, `cms`, `x509-cert`, `x509-tsp`, `reqwest` o `ureq`; esto **no sustituye** `cargo audit`/RustSec sobre el lockfile resultante.
- `cargo-audit` no está instalado en el entorno actual; no se instaló por conveniencia. La integración deberá ejecutar una auditoría automatizada del lockfile además de esta revisión manual.
- `cms 0.3.0-pre.2` y `trackone-rfc3161 0.2.0-beta.1` son prereleases y elevan el riesgo de estabilidad.

## Evaluación

La ruta RustCrypto coherente disponible hoy es `x509-tsp 0.1 + der 0.7 + cms 0.2 + x509-cert 0.2`. Sirve para representación y parseo, pero deja por implementar —usando primitivas mantenidas— la verificación completa de CMS, binding RFC 5816, EKU `id-kp-timeStamping`, cadena, política, nonce y estrategia de revocación/tiempo histórico. La línea RustCrypto más reciente (`der 0.8`, `cms 0.3`, `x509-cert 0.3`) todavía no tiene una versión compatible publicada de `x509-tsp`.

Para transporte, `ureq 3.4.2` con rustls explícito es la opción de menor superficie y no fuerza la decisión async pendiente. `reqwest` es viable técnicamente con `default-features=false`, backend rustls explícito y límites estrictos, pero agrega Tokio/Hyper y más dependencias.

`rustls-webpki` valida el servidor TLS, no debe reutilizarse como afirmación de que el certificado firmante TSA fue validado con todas las reglas de RFC 3161/5816. La verificación offline del `.tsr` debe separar confianza TLS de confianza TSA.

## Selección aprobada para el prototipo

1. `x509-tsp 0.1.0`, `der 0.7.10`, `cms 0.2.3`, `x509-cert 0.2.x` para representación compatible.
2. `ureq 3.4.2` con features mínimas y rustls explícito para POST síncrono; sin TSA default, redirects automáticos ni conexión durante verificación offline.
3. Mantener trust anchors/política TSA como configuración explícita. No usar el almacén WebPKI del transporte como trust store implícito de la TSA.
4. SP3-10 usa `rustls-webpki 0.103.15` para path PKIX al `genTime` con anchor/policy TSA explícitos y EKU RFC 3161. Revocación histórica permanece fuera de esa afirmación y en gate.

## Impacto estimado

La ruta recomendada añade dos grupos: formatos RustCrypto y transporte `ureq`/rustls. Introducirá versiones 0.7/0.2 del stack de formatos en paralelo si otras dependencias futuras adoptan 0.8/0.3. El transporte agrega rustls/provider y roots configurados, pero evita Tokio/Hyper. Windows y Linux están soportados por estas opciones puramente Rust; la alternativa OpenSSL introduce requisitos de empaquetado nativo.

## Gates humanos que permanecen abiertos

Se resolvieron stack, transporte y provider TLS. Todavía se requiere decidir:

1. contrato/mecanismo definitivo de revocación y validación histórica para afirmar `HISTORICALLY_VALIDATED`.
