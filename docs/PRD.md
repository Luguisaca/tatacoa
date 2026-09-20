# PRD — Product Requirements Baseline

## Estado

**APROBADO para la evolución de Alpha.**

Este PRD separa requisitos estables del producto, capacidades ya implementadas y capacidades aprobadas para la siguiente entrega. El estado operativo exacto se mantiene en los documentos de implementación y en [ROADMAP.md](ROADMAP.md).

## Objetivo

Plataforma local-first de trabajo y aprendizaje para auditorías de seguridad autorizadas que preserva evidencia verificable, conserva contexto y transforma cada prueba en conocimiento reproducible sin sustituir la Validación Humana.

## Requisitos estables del producto

- funcionamiento local sin SaaS obligatorio;
- cero telemetría por defecto;
- contexto y alcance autorizado;
- preservar RAW y provenance;
- integridad y verificación portable/offline;
- Validación Humana para promoción de evidencia;
- reproducibilidad/retest;
- minimización de datos y fallos explícitos;
- interfaces que no dupliquen ni debiliten la autoridad de dominio;
- continuidad del trabajo como capacidad de producto;
- evolución compatible con el roadmap y decisiones congeladas.

## Modelo de dominio

`ENGAGEMENT → SCOPE → ENVIRONMENT → TARGET → SESSION → EXECUTION → ARTIFACT → CANDIDATE → VALIDATION → EVIDENCE`

Flujos derivados:

- `EVIDENCE → REPLAY → nueva EXECUTION → RETEST`
- `ARTIFACT/EVIDENCE → DERIVATION → REDACTED/DERIVED ARTIFACT → EXPORT`
- `EXECUTION/EVIDENCE → KNOWLEDGE CARD`

Finding no es la unidad primaria. La unidad fundamental es una **Execution contextualizada** que produce artefactos.

## Estados de evidencia

`CAPTURED → CANDIDATE → REVIEWED → VALIDATED | DISCARDED → EVIDENCE`

La promoción a evidencia validada requiere Validación Humana.

## Capacidades Alpha implementadas

El alcance implementado de SP1/SP2 incluye, según sus documentos de estado:

- Engagement y contexto tipado;
- Environment/Target/Session;
- ejecución mediante executable + argv;
- captura stdout/stderr y estados explícitos;
- Artifact con ID lógico y SHA-256;
- manifest versionado;
- aislamiento de engagements;
- bundle portable y verificación offline;
- Security Profiles;
- export Plain y Encrypted según política;
- Generic Execution Adapter;
- Knowledge Card manual/versionada;
- Replay Recipe foundation/versionada;
- provenance y relaciones implementadas en el alcance documentado.

`tatacoa.encrypted.v1` está implementado pero pendiente de QA humano independiente. Implementado no significa aprobado.

## Capacidades aprobadas para SP3 — no implementadas

La siguiente etapa busca una **Usable Alpha**:

- `tatacoa-app-api` como capa común de operaciones de usuario;
- `tatacoa-desktop` con Tauri 2;
- `tatacoa-cli` continúa plenamente funcional e instalable sin Desktop;
- Desktop instalable sin requerir CLI preinstalado;
- flujo gráfico basado en trabajo real de pentesting;
- interoperabilidad de datos soportados entre CLI y Desktop;
- pausa/reanudación y persistencia segura del trabajo;
- recuperación tras cierre/fallo sin falsear estados de evidencia;
- resumen/contexto de continuidad para retest y reanudación;
- apertura/importación autorizada de paquetes compatibles preservando políticas, integridad y provenance.

RFC 3161 entra en investigación/diseño activo durante esta evolución por su relevancia para integridad temporal. Su mecanismo concreto no se considera aprobado hasta completar investigación y decisión humana.

## Estados de captura

`COMPLETE | PARTIAL | TRUNCATED | FAILED | UNKNOWN`

Un fallo nunca se representa silenciosamente como captura completa.

## Reproducción

`REPRODUCED | NOT_REPRODUCED | CHANGED | ERROR`

Replay conserva contexto, parámetros, placeholders de secretos y requisitos de seguridad. Una receta puede evolucionar deliberadamente a script/miniherramienta con procedencia.

## Knowledge Card

Campos mínimos:

`WHAT / WHY / OBJECTIVE / HOW / OBSERVE / PROVES / DOES_NOT_PROVE / ERRORS / VALIDATION / DEFENSIVE_CONTEXT / REFERENCES / RELATED_TECHNIQUES`

Fuentes:

`UPSTREAM_OFFICIAL | STANDARD | GOVERNMENT | PROJECT_DOCUMENTATION | OPERATOR_NOTE | AI_DRAFT | COMMUNITY`

Contenido factual asistido por IA debe pasar de `AI_DRAFT` a revisión de fuente antes de considerarse verificado.

## Perfiles de seguridad

- `LAB_LEARNING`
- `PROFESSIONAL`
- `HIGH_SENSITIVITY`
- `CUSTOM`

Precedencia:

`SYSTEM POLICY > ENGAGEMENT POLICY > USER DEFAULT > EXPORT CHOICE`

Integridad, manifest y procedencia permanecen activos. El cifrado puede ser opcional en laboratorio y exigido por política en contextos sensibles.

## No objetivos actuales

- SaaS o cloud obligatorios;
- blockchain;
- scanner propio;
- auto-exploit;
- pentest autónomo;
- SIEM;
- findings confirmados automáticamente;
- mobile como requisito actual;
- plugins arbitrarios de terceros;
- convertir IA en autoridad de validación.

Reporting avanzado, colaboración controlada, firmas, RFC 3161 y otras capacidades del horizonte **no son “no objetivos permanentes”**: su estado y secuencia se gobiernan desde [ROADMAP.md](ROADMAP.md).

## Plataformas

El Core es multiplataforma Windows + Linux desde el diseño.

Targets:

- Windows 11 x64: desarrollo y QA;
- WSL2 Linux x64: desarrollo/integración Linux;
- Kali Linux x64: QA objetivo;
- Parrot OS x64: QA objetivo.

WSL2 no sustituye QA específico en Kali/Parrot. Las capacidades dependientes de una herramienta o plataforma deben declarar compatibilidad sin reducir la portabilidad del Core.
