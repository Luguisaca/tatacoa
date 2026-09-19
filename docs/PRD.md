# PRD — Product Requirements Baseline

## Estado

**APROBADO para Alpha.**

## Objetivo

Plataforma local de trabajo y aprendizaje para auditorías de seguridad que preserva evidencia verificable y transforma cada prueba autorizada en conocimiento reproducible.

## Modelo de dominio

`ENGAGEMENT → SCOPE → ENVIRONMENT → TARGET → SESSION → EXECUTION → ARTIFACT → CANDIDATE → VALIDATION → EVIDENCE`

Flujos derivados:

- `EVIDENCE → REPLAY → nueva EXECUTION → RETEST`
- `ARTIFACT/EVIDENCE → DERIVATION → REDACTED/DERIVED ARTIFACT → EXPORT`
- `EXECUTION/EVIDENCE → KNOWLEDGE CARD`

El Finding no es la unidad primaria de V1. La unidad fundamental es una **Execution contextualizada** que produce artefactos.

## Estados de evidencia

`CAPTURED → CANDIDATE → REVIEWED → VALIDATED | DISCARDED → EVIDENCE`

La promoción a evidencia validada requiere Validación Humana.

## Requisitos funcionales Alpha

- crear Engagement y contexto;
- registrar Environment/Target/Session necesarios para una Execution;
- ejecutar un programa con ejecutable + argv;
- capturar stdout/stderr y estado de captura;
- generar Artifact con ID lógico y SHA-256;
- producir manifest versionado;
- aislar engagements;
- exportar bundle portable;
- verificar bundle offline y detectar alteraciones;
- soportar Security Profile y export Plain/Encrypted cuando la política lo permita;
- Generic Execution Adapter;
- Knowledge Card manual;
- preparar Replay sin afirmar todavía automatización completa.

## Estados de captura

`COMPLETE | PARTIAL | TRUNCATED | FAILED | UNKNOWN`

Un fallo nunca se representa silenciosamente como captura completa.

## Reproducción

Resultado de reproducción:

`REPRODUCED | NOT_REPRODUCED | CHANGED | ERROR`

Replay debe conservar contexto, parámetros, placeholders de secretos y requisitos de seguridad. Una receta reproducible puede evolucionar deliberadamente a script/miniherramienta con procedencia.

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

## No objetivos V1

- SaaS obligatorio;
- cloud obligatorio;
- blockchain;
- scanner propio;
- auto-exploit;
- pentest autónomo;
- multi-tenant empresarial;
- SIEM;
- cientos de adapters;
- findings confirmados automáticamente;
- reporting empresarial completo;
- GUI como requisito del Alpha;
- mobile;
- plugins arbitrarios de terceros.

## Plataformas

QA inicial: Kali Linux y Parrot OS. Arquitectura preparada para ampliar compatibilidad; Windows completo queda post-V1 salvo decisión posterior.
