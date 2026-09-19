# Propuesta de formato `tatacoa.encrypted.v1`

## Estado

**PROPUESTA PARA VALIDACIÓN HUMANA. NO CONGELADA NI IMPLEMENTADA COMO FORMATO PERSISTENTE.**

Este documento concreta los puntos todavía pendientes del spike criptográfico. La foundation interna implementada no serializa este layout y no habilita todavía exportaciones Encrypted.

Todos los enteros se codifican unsigned, big-endian. Todo campo `reserved` debe ser cero. No se aceptan algoritmos, parámetros ni tamaños alternativos en v1.

## Header público fijo

Longitud exacta propuesta: 136 bytes.

| Offset | Bytes | Campo | Valor v1 |
|---:|---:|---|---|
| 0 | 16 | magic/schema | `TATACOAENCV1` seguido de cuatro bytes `00` |
| 16 | 2 | header_length | `136` |
| 18 | 2 | flags | `0` |
| 20 | 1 | kdf_id | `1` = Argon2id v0x13 |
| 21 | 1 | wrap_aead_id | `1` = AES-256-GCM |
| 22 | 1 | object_aead_id | `1` = AES-256-GCM |
| 23 | 1 | stream_id | `1` = STREAM-BE32 |
| 24 | 4 | argon2_memory_kib | `65536` |
| 28 | 4 | argon2_iterations | `3` |
| 32 | 4 | argon2_lanes | `4` |
| 36 | 4 | plaintext_chunk_bytes | `1048576` propuesto |
| 40 | 16 | argon2_salt | aleatorio por exportación |
| 56 | 12 | bundle_key_wrap_nonce | aleatorio por exportación |
| 68 | 7 | directory_stream_nonce | aleatorio por exportación |
| 75 | 5 | reserved | cero |
| 80 | 8 | directory_ciphertext_length | longitud acotada |
| 88 | 48 | wrapped_bundle_key | 32 bytes cifrados + tag GCM de 16 bytes |

Los bytes `[0, 88)` son el AAD usado al proteger la Bundle Key. Así se autentican schema, algoritmos, parámetros, salt, nonces, tamaño de chunk y longitud del directorio. Los 136 bytes completos son AAD del directorio cifrado.

El header no contiene Engagement ID, nombres, rutas, Artifact IDs, cantidades de objetos ni metadata operacional. El tamaño total del archivo y el tamaño del directorio continúan siendo observables.

## Payload protegido

Después del header aparecen, sin padding:

1. stream cifrado del directorio, de `directory_ciphertext_length` bytes;
2. streams cifrados de objetos, concatenados en el orden autenticado por el directorio;
3. EOF exacto; bytes adicionales se rechazan.

El directorio tiene su propia DEK:

- tipo HKDF: `directory`;
- ID HKDF: `root`.

Su plaintext comienza con:

| Bytes | Campo |
|---:|---|
| 8 | magic `TATDIR01` |
| 4 | object_count |
| variable | `object_count` descriptores |

Cada descriptor usa:

| Bytes | Campo |
|---:|---|
| 1 | object_type: `1` manifest, `2` artifact |
| 1 | flags: `0` |
| 2 | object_id_length |
| 8 | plaintext_length |
| 8 | ciphertext_length |
| 7 | stream_nonce |
| 1 | reserved: cero |
| variable | object_id UTF-8 canónico |

Debe existir exactamente un objeto manifest y debe ser el primer descriptor. Knowledge y Replay continúan dentro del manifest protegido mientras ese sea el contrato Plain vigente. Tipos desconocidos se rechazan en v1.

Cada DEK se deriva mediante HKDF-SHA-256 desde la Bundle Key con información canónica:

`TATACOA\0encrypted\0v1\0dek\0 || len(type):u32 || type || len(id):u32 || id`

El AAD de cada objeto es:

`header[0..136] || TATACOA\0encrypted\0v1\0object\0 || descriptor_serializado`

El AAD del directorio es:

`header[0..136] || TATACOA\0encrypted\0v1\0directory\0root`

## Framing STREAM

Se propone `StreamBE32<AES-256-GCM>` de `aead-stream`:

- nonce base aleatoria de 7 bytes por objeto;
- STREAM completa el nonce GCM de 96 bits con contador big-endian de 32 bits y flag final de un byte;
- contador inicial cero e incremento interno de uno;
- cada segmento contiene hasta `plaintext_chunk_bytes` y añade un tag de 16 bytes;
- incluso un objeto vacío emite un único segmento final con tag;
- el último segmento se procesa exclusivamente con `encrypt_last`/`decrypt_last`;
- no se persisten nonces GCM por chunk porque se reconstruyen desde nonce base, posición y flag final.

La DEK es distinta por tipo e ID. Cada stream usa una nonce base generada por el CSPRNG del sistema. TATACOA no acepta nonces suministradas por usuario y no reinicia un stream bajo la misma DEK.

El framing STREAM hace que modificación, reordenamiento, duplicación, omisión o cambio del flag final provoquen fallo de autenticación.

## Tamaño de chunk propuesto

Se propone **1 MiB de plaintext por chunk**.

Benchmark local de desarrollo sobre 64 MiB, build release, Windows x64:

| Chunk | Segmentos | Encrypt | Decrypt |
|---:|---:|---:|---:|
| 64 KiB | 1024 | 84 ms | 86 ms |
| 256 KiB | 256 | 77 ms | 71 ms |
| 1 MiB | 64 | 73 ms | 72 ms |
| 4 MiB | 16 | 67 ms | 64 ms |

1 MiB queda cerca del throughput de 4 MiB, limita el buffer normal a aproximadamente 1 MiB más tag y reduce en 16 veces la cantidad de invocaciones frente a 64 KiB. El benchmark es orientativo y no constituye una garantía de rendimiento.

## Límites propuestos del lector

Todos se comprueban con aritmética checked y antes de reservar memoria basada en datos no confiables:

| Recurso | Límite v1 propuesto |
|---|---:|
| Header | exactamente 136 bytes |
| Password recibida | 1024 bytes |
| Archivo envelope | 1 TiB |
| Directorio cifrado | 16 MiB |
| Objetos | 65.536 |
| Object ID | 64 bytes UTF-8 |
| Manifest plaintext | 16 MiB |
| Artifact plaintext individual | 256 GiB |
| Chunks por objeto | 262.144, más un único chunk final vacío cuando aplique |
| Plaintext por chunk | exactamente máximo 1 MiB |
| Ciphertext por chunk | máximo 1 MiB + 16 bytes |

La suma declarada de longitudes debe coincidir exactamente con el tamaño real del archivo. El lector no carga artifacts completos: procesa como máximo un chunk y escribe únicamente a staging privado.

## Orden de validación y fallo cerrado

1. Comprobar tamaño físico, magic, versión, header exacto, IDs de algoritmos, flags/reserved y límites estructurales.
2. Rechazar parámetros Argon2id distintos de los valores fijos antes de ejecutar la KDF.
3. Derivar KEK y autenticar/desenvolver la Bundle Key. Contraseña incorrecta y header modificado producen el mismo error genérico.
4. Autenticar completamente el directorio antes de aceptar sus descriptores.
5. Validar IDs, conteos, longitudes, sumas y overflow antes de procesar objetos.
6. Descifrar cada objeto por chunks hacia staging. Un chunk no se escribe hasta que su tag haya sido validado.
7. No publicar ningún archivo ni manifest hasta autenticar el último chunk de todos los objetos y comprobar EOF exacto.
8. Ante cualquier fallo se descarta staging y no se intenta interpretar plaintext parcial.

Los errores externos no distinguen contraseña incorrecta, Bundle Key alterada, tag inválido o modificación autenticada. Los mensajes no incluyen password, KEK, Bundle Key, DEK ni bytes de plaintext.

## Referencias del spike

- RFC 9106, Argon2id.
- RFC 5869, HKDF.
- NIST SP 800-38D, AES-GCM y unicidad de IV.
- RustCrypto `argon2`, `hkdf`, `aes-gcm` y `aead-stream`.
- `zeroize` y `getrandom` para manejo de memoria y CSPRNG del sistema.
