# Implementación Encrypted v1

## Estado

`tatacoa.encrypted.v1` está integrado en el baseline Alpha consolidado y fue validado dentro del alcance documentado mediante pruebas automatizadas y QA humano. No se declara certificación, validación FIPS, garantía forense ni protección del workspace.

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

## Hardening de password — integrado y validado

El hardening posterior a la implementación base de Encrypted v1 quedó integrado en `main` y validado dentro del alcance de QA de la Alpha.

Política vigente de creación:

- `LAB_LEARNING`: mínimo de 12 caracteres.
- `PROFESSIONAL`: mínimo de 14 caracteres y rechazo de passwords evidentemente predecibles.
- `HIGH_SENSITIVITY`: mínimo de 16 caracteres y rechazo de passwords evidentemente predecibles.
- `CUSTOM`: fail-closed mientras no exista una política aprobada.
- máximo común: 1024 bytes UTF-8.
- verify conserva compatibilidad con passwords históricas no vacías dentro del máximo.

La validación permanece en `tatacoa-core`; CLI consume el Security Profile del engagement. No existe canal de password por argumentos, variables de entorno o archivos. No se modificaron Argon2id, AES-256-GCM, HKDF, STREAM-BE32, el envelope ni los parámetros congelados del KDF.

La política evita reglas arbitrarias de composición: una passphrase suficientemente larga y no predecible puede ser válida sin exigir combinaciones artificiales de mayúsculas, números o símbolos.

### Cierre de QA e integración

El gate del incremento se completó con CI aplicable y QA humano del baseline consolidado, incluyendo Security Profiles, exportación/verificación, aceptación y rechazo de passwords según política, fallo cerrado ante password incorrecta y regresión de Encrypted v1. El estado vigente es **VALIDATED dentro del alcance documentado**.

La guía reproducible permanece en [`ALPHA-USER-QA-GUIDE.md`](ALPHA-USER-QA-GUIDE.md). La validación no amplía las garantías más allá de los entornos y casos efectivamente probados.
