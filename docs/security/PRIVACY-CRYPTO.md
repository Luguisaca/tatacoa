# Privacidad y criptografía

## Estado

**Política aprobada; algoritmos/parametrización exacta de cifrado requieren spike antes de implementación.**

## Seguridad configurable

No toda evidencia requiere el mismo nivel de confidencialidad. Un laboratorio personal no debe asumir automáticamente las obligaciones de un engagement sensible.

Perfiles:

- `LAB_LEARNING`: cifrado opcional si política lo permite;
- `PROFESSIONAL`: cifrado recomendado/default de export;
- `HIGH_SENSITIVITY`: política puede exigirlo;
- `CUSTOM`: configuración explícita dentro de límites del sistema.

Precedencia:

`SYSTEM POLICY > ENGAGEMENT POLICY > USER DEFAULT > EXPORT CHOICE`

Un nivel superior puede impedir downgrade.

## Propiedades siempre activas

Independientemente del cifrado:

- manifest;
- digest/integridad;
- procedencia;
- contexto;
- registro explícito del modo de exportación.

## Modos de export

- Plain;
- Encrypted;
- Public/Sanitized derivado.

Una exportación pública/sanitizada nunca sustituye el RAW.

## Diseño criptográfico aprobado

Cuando exista cifrado:

- usar AEAD estandarizado mediante bibliotecas mantenidas;
- DEK aleatoria por material protegido;
- contraseña → KDF adecuada (Argon2id es candidato) → KEK → unwrap DEK;
- contraseña nunca usada directamente como clave;
- CSPRNG del sistema;
- disciplina estricta de nonce;
- header público mínimo;
- manifest interno protegido cuando corresponda;
- contraseña incorrecta o autenticación fallida no produce plaintext parcial;
- no master key/backdoor.

El algoritmo AEAD, parámetros Argon2id, formato envelope y manejo de memoria **no están congelados** hasta el spike de implementación y revisión de fuentes oficiales.

## Workspace

Alpha no promete cifrado nativo del workspace. Se recomendarán permisos del SO y full-disk encryption donde aplique. No afirmar protección at-rest que TATACOA no implemente.

## Post-V1

- destinatarios con clave pública;
- firmas;
- trusted timestamp RFC 3161;
- workspace cifrado;
- integración con hardware keys/PKCS#11/TPM/KMS tras investigación.

## Distinciones

- hash ≠ firma;
- firma ≠ trusted timestamp;
- timestamp ≠ cadena de custodia;
- cifrado ≠ autenticidad de autor;
- “usa un algoritmo estándar” ≠ “FIPS validated”.
