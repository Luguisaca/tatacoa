# Implementación Encrypted v1

## Estado

La rama de desarrollo implementa exportación y verificación persistente de `tatacoa.encrypted.v1`. El trabajo tiene pruebas automatizadas, pero continúa pendiente de revisión y QA humano independiente. No se declara certificación, validación FIPS ni protección del workspace.

La guía reproducible para validarlo como usuario se encuentra en [`ALPHA-USER-QA-GUIDE.md`](ALPHA-USER-QA-GUIDE.md).

## Superficie implementada

- `tatacoa-core` escribe un único archivo Encrypted v1 mediante staging exclusivo y rename final; nunca reemplaza un destino existente.
- El manifest, Knowledge, Replay y todos los artifacts quedan dentro del payload autenticado y cifrado.
- El lector autentica el header, recupera la Bundle Key, autentica el directorio y procesa cada objeto en el orden declarado.
- Los artifacts descifrados se envían directamente a un sink SHA-256; el verifier no los persiste ni los ejecuta.
- Se exige coincidencia exacta de IDs, tamaños, digest, número y orden de objetos, además de EOF exacto.
- Plain conserva su formato de directorio y su semántica anterior.

## CLI

`tatacoa export` y `tatacoa run --bundle` aceptan `--encrypted`. Sin selección explícita se aplica el default del Security Profile. `--encrypted` y `--acknowledge-plain-export` son mutuamente excluyentes.

- `LAB_LEARNING`: Plain por defecto; `--encrypted` habilita Encrypted.
- `PROFESSIONAL`: Encrypted por defecto; Plain requiere `--acknowledge-plain-export`.
- `HIGH_SENSITIVITY`: solo Encrypted; Core rechaza Plain incluso ante llamada directa.
- `CUSTOM`: ambos modos permanecen denegados.

La password se solicita por TTY sin eco y se confirma durante export. Verify solicita una sola vez. No existe flag de password ni selección de algoritmo, KDF, parámetros o chunk.

`tatacoa verify RUTA` y `tatacoa-verify RUTA` distinguen un bundle Plain (directorio) de Encrypted v1 (archivo regular). El verificador permanece offline y no ejecuta contenido.

## Fallo cerrado y límites

Password incorrecta, parámetros no canónicos, modificación del header/directorio/objeto, truncamiento, reordenamiento, datos finales, digest incoherente o fallo AEAD terminan en error. No se devuelve manifest antes de autenticar y verificar todos los objetos.

Se aplican los límites congelados del formato: envelope de 1 TiB, directorio cifrado de 16 MiB, 65 536 objetos, manifest de 16 MiB, artifact individual de 256 GiB y 262 144 chunks por stream. El procesamiento de artifacts usa memoria acotada al chunk de 1 MiB; Argon2id conserva el costo fijo aprobado.

## Límites que permanecen

- El workspace local no está cifrado.
- Encrypted aporta confidencialidad e integridad con password; no firma autoría ni aporta timestamp confiable.
- No hay recuperación de password, master key, backdoor ni destinatarios de clave pública.
- La resistencia efectiva depende también de la entropía de la password y de la seguridad del host.
- Public/Sanitized, firmas, RFC 3161 y cifrado del workspace continúan fuera de este incremento.

## Hardening de password — candidato pendiente de integración

La rama `qa/encrypted-v1-hardening` contiene un incremento de hardening posterior a la implementación base de Encrypted v1. Su estado es **candidato de QA; pendiente de validación humana e integración**. No debe interpretarse como comportamiento aprobado de `main` hasta completar QA y revisión de integración.

Cambios bajo evaluación:

- `LAB_LEARNING`: conserva mínimo de 12 caracteres para facilitar aprendizaje y pruebas controladas.
- `PROFESSIONAL`: mínimo de 14 caracteres y rechazo de passwords evidentemente predecibles.
- `HIGH_SENSITIVITY`: mínimo de 16 caracteres y rechazo de passwords evidentemente predecibles.
- `CUSTOM`: continúa fail-closed mientras no exista una política aprobada.
- La validación se aplica en `tatacoa-core`; CLI consume la política del Security Profile del engagement.
- No se añade un canal de password por argumentos, variables de entorno o archivos.
- No se modifican los parámetros criptográficos congelados de Encrypted v1: Argon2id, AES-256-GCM, formato del envelope ni parámetros del KDF.
- Se corrige la guía de error de `HIGH_SENSITIVITY` para indicar el uso de exportación Encrypted.

La política evita imponer reglas arbitrarias de composición. Una passphrase larga y no predecible puede ser válida sin exigir combinaciones artificiales de mayúsculas, números o símbolos.

### Gate de integración

Antes de integrar este hardening debe existir evidencia de:

1. CI verde en Windows y Linux.
2. QA humano de exportación y verificación para `LAB_LEARNING`, `PROFESSIONAL` y `HIGH_SENSITIVITY`.
3. Rechazo confirmado de passwords cortas y patrones evidentemente débiles en los perfiles protegidos.
4. Aceptación confirmada de passphrases suficientemente largas y no predecibles.
5. Verificación de que una password incorrecta continúa fallando de forma cerrada y sin modificar el bundle.
6. Regresión satisfactoria de Encrypted v1 y Security Profiles.
7. Working tree limpio y revisión final del diff antes de cualquier integración.

Hasta completar este gate, el estado documental es **QA PENDING / INTEGRATION PENDING**.
