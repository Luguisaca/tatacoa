# RFC 3161 — contrato de timestamping de Sprint 03

## Estado

**OBJETO Y MOMENTO APROBADOS; transporte/verificación en implementación.**

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
