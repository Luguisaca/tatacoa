# Sprint 02 — Alpha Expansion

## Estado

Sprint 02 amplía la vertical aprobada sin sustituir las garantías de Sprint 01:

`ENGAGEMENT → SCOPE → ENVIRONMENT → TARGET → SESSION → EXECUTION`

La implementación también incorpora Knowledge Card manual, Replay Recipe foundation, políticas operativas para export Plain y validación de relaciones/provenance. Este estado queda listo para QA independiente; no implica producción, certificación ni validación automática de evidencia.

## Contexto operacional

Los IDs `scp_`, `env_`, `tgt_` y `ses_` son tipados. Cada registro conserva `engagement_id` y las referencias de sus padres. Antes de ejecutar, Core carga el contexto completo y rechaza:

- objetos pertenecientes a otro engagement;
- Environment fuera del Scope declarado;
- Target fuera de su Environment/Scope;
- Session con relaciones inconsistentes;
- Execution cuyo snapshot no coincide con sus IDs de contexto.

El manifest emitido es `tatacoa.alpha.v2`. El verifier continúa leyendo `tatacoa.alpha.v1`, pero un manifest v1 no puede declarar relaciones exclusivas de Sprint 02.

## Security Profiles y Plain export

Plain continúa siendo el único modo implementado. La política no intenta cifrar ni degrada silenciosamente:

| Perfil | Política de export Plain |
|---|---|
| `LAB_LEARNING` | Permitido. |
| `PROFESSIONAL` | Requiere `--acknowledge-plain-export`. |
| `HIGH_SENSITIVITY` | Denegado mientras Encrypted no exista. |
| `CUSTOM` | Denegado hasta disponer de una política Custom aprobada. |

El rechazo ocurre antes de crear el staging del bundle.

## Knowledge Card

`tatacoa.knowledge.v1` implementa manualmente los campos aprobados: WHAT, WHY, OBJECTIVE, HOW, OBSERVE, PROVES, DOES_NOT_PROVE, ERRORS, VALIDATION, DEFENSIVE_CONTEXT, REFERENCES y RELATED_TECHNIQUES.

Las referencias usan las clasificaciones aprobadas. Una tarjeta `SOURCE_REVIEWED` no puede conservar una referencia `AI_DRAFT`; esto evita presentar un borrador de IA como fuente revisada. La tarjeta sigue sin promover artifacts a Evidence.

## Replay Recipe

`tatacoa.replay.v1` conserva:

- Execution de origen y snapshot de contexto;
- executable y `argv_template` separados;
- prerequisites;
- placeholders tipados como secretos/requeridos;
- límites de autorización obligatorios.

Los placeholders usan tokens `{{NAME}}`. La receta se registra y exporta, pero no se ejecuta automáticamente. No se almacenan valores/defaults de secretos en el modelo de placeholder.

## Provenance y Evidence

Cada Artifact declara clasificación, estado de evidencia y provenance. Para los RAW capturados:

- clasificación `RAW`;
- provenance `CAPTURE`;
- cero Artifact sources;
- estado inicial `CAPTURED`.

El verifier valida fuentes existentes y ausencia de ciclos para relaciones derivadas. Un manifest que afirme `VALIDATED` sin un registro de Validación Humana es rechazado. Sprint 02 no implementa promoción a Evidence.

## CLI

Los comandos añadidos son:

- `scope-create`;
- `environment-create`;
- `target-create`;
- `session-create`;
- `knowledge-create`;
- `replay-create`;
- `export` para exportar una Execution existente con sus registros asociados.

`run` requiere `--session`. `--bundle` es opcional para permitir crear Knowledge/Replay antes de la exportación final.

## Pruebas de desarrollo

Además de la regresión completa de Sprint 01, se cubren:

- contexto completo y aislamiento entre engagements;
- Knowledge y Replay exportados/verificados;
- rechazo de AI_DRAFT como source-reviewed;
- política Plain para Professional y High Sensitivity;
- rechazo de Evidence validada sin revisión humana;
- rechazo de ciclos de provenance;
- compatibilidad de lectura/verificación con manifest v1.

## Límites

- No se implementó cifrado, AEAD, Argon2id ni bundle Encrypted.
- No se implementaron firmas, RFC 3161, DB ni runtime async.
- `CUSTOM` todavía no dispone de un lenguaje/configuración de política.
- Knowledge y Replay se incluyen dentro del manifest portable; sus directorios del bundle permanecen reservados.
- Replay no ejecuta recetas ni autoriza operaciones.
- Evidence promotion y registro de revisor humano quedan fuera de este incremento.
- Se conservan las limitaciones filesystem/TOCTOU documentadas en Sprint 01.

