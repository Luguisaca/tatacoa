# Privacidad y criptografía

## Estado

**Política y contrato Encrypted v1 aprobados. Implementación persistente en curso.**

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

Encrypted v1 queda congelado con:

- Argon2id v0x13: 64 MiB, 3 iteraciones, 4 lanes, salt aleatoria de 16 bytes y KEK de 32 bytes;
- Bundle Key aleatoria de 256 bits por exportación;
- HKDF-SHA-256 con separación de dominio para una DEK independiente por objeto;
- AES-256-GCM y STREAM-BE32 con chunks de 1 MiB;
- header binario fijo de 136 bytes y límites de lector documentados en `ENCRYPTED-V1-FORMAT-PROPOSAL.md`;
- password por prompt TTY sin eco; nunca mediante argumento del proceso.

Las claves se zeroizan cuando las bibliotecas mantenidas lo permiten. Esto reduce permanencia en memoria, pero no constituye una garantía sobre copias realizadas por el sistema operativo, allocator o hardware.

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
