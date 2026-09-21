# RFC 3161 — contrato de timestamping de Sprint 03

## Estado

**OBJETO, MOMENTO, BOUND Y SIGNATURE_VALID IMPLEMENTADOS; TRUST EN IMPLEMENTACIÓN; HISTÓRICO EN GATE.**

RFC 3161 aporta una afirmación de tiempo confiable sobre un `messageImprint`; no demuestra autoría ni reemplaza integridad, provenance o Validación Humana.

## Objetos

- Encrypted v1: SHA-256 sobre todos los bytes exactos del archivo terminado.
- Plain: SHA-256 del digest raíz `tatacoa.plain-root.v1` descrito abajo.
- La respuesta RFC 3161 DER se conserva como sidecar `.tsr`; no modifica el objeto sellado.

## Plain root v1

No canonicaliza JSON ni depende del orden del filesystem. La entrada al SHA-256 raíz es:

1. ASCII `tatacoa.plain-root.v1` seguido de byte NUL;
2. número de entradas como `u32` big-endian;
3. por cada entrada, ordenada por bytes UTF-8 de path relativo: `u32` big-endian de longitud del path, bytes del path, `u64` big-endian del tamaño exacto y 32 bytes del SHA-256 del archivo.

Las entradas son `manifest.json` y cada `artifact.path` declarado. Paths duplicados, inseguros, ausentes o artifacts cuya integridad no corresponda al manifest producen error.

Los 32 bytes obtenidos por `SHA-256(root-encoding)` son directamente el `messageImprint` SHA-256 enviado en `TimeStampReq`. No se aplica un segundo SHA-256 sobre ese digest. En otras palabras, queda prohibido `SHA256(SHA256(root-encoding))`.

## Momento y fallo

El imprint se calcula después de finalizar atómicamente y verificar localmente el bundle. Solo una TSA configurada explícitamente puede recibirlo. Ausencia de configuración, red o respuesta válida se reporta sin invalidar, borrar ni modificar el bundle existente. Nunca existe fallback a hora local como trusted timestamp.

La solicitud debe usar SHA-256, nonce generado por TATACOA y `certReq=true`. La verificación debe comprobar status, imprint, algoritmo, nonce, firma/cadena/certificado de timestamp y política aceptada conforme a RFC 3161 y RFC 5816.

## Transporte y límites de SP3-07

- TSA sin valor predeterminado y configurada explícitamente mediante URL HTTPS;
- HTTP síncrono con timeout entre 1 ms y 120 segundos;
- TLS `rustls` con provider `ring` y roots WebPKI explícitos para autenticar el servidor TLS;
- sin redirects, proxy implícito ni downgrade a HTTP;
- media types RFC 3161 obligatorios;
- máximo 4 MiB para respuesta y sidecar;
- sidecar DER `.tsr` atómico y sin sobrescritura;
- verificación offline sin conexiones.

Los roots TLS no son trust anchors TSA. Una respuesta alcanza `BOUND` al coincidir estructura, SHA-256, imprint y —durante la solicitud— nonce; esto no demuestra firma, identidad, EKU, cadena, trust o revocación.

## Contrato `SIGNATURE_VALID`

Se exige un único `SignerInfo`, certificado firmante identificado inequívocamente, `signedAttrs` DER, `content-type=id-ct-TSTInfo`, `message-digest` recalculado localmente, binding `SigningCertificateV2`/ESSCertIDv2 y firma criptográfica correcta. El ESSCertID heredado basado en SHA-1 se rechaza. La solicitud solo persiste el sidecar cuando la respuesta alcanza al menos `SIGNATURE_VALID`.

Allowlist Alpha: RSA PKCS#1 v1.5 + SHA-256/384/512, RSA-PSS + SHA-256/384/512, ECDSA P-256 + SHA-256 y ECDSA P-384 + SHA-384. SHA-1 produce `FAIL`; DSA, Ed25519 y algoritmos no listados producen `UNSUPPORTED`. Una firma incorrecta con algoritmo soportado produce `FAIL`.

## Assurance y checks

Niveles: `PRESENT → BOUND → SIGNATURE_VALID → TRUSTED → HISTORICALLY_VALIDATED`.

Cada check usa `PASS`, `FAIL`, `NOT_EVALUATED`, `INDETERMINATE` o `UNSUPPORTED`. Al verificar solo un `.tsr` offline, el nonce queda `NOT_EVALUATED` porque no se conserva la petición original; el `messageImprint` sí se recalcula desde el objeto.

`TRUSTED` permanece separado: requiere path PKIX contra anchors TSA explícitos, validez al `genTime`, EKU `id-kp-timeStamping` y policy aceptada. El trust HTTPS nunca autoriza automáticamente una TSA. `HISTORICALLY_VALIDATED` continúa en gate: sin evidencia histórica suficiente de revocación el resultado es `INDETERMINATE`, y la verificación offline nunca obtiene recursos de red.
