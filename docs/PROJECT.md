# Visión, origen y problema

## Estado

**APROBADO — baseline de Sprint 01.**

## Origen

TATACOA nació de investigar una necesidad recurrente en pruebas de seguridad: no basta con ejecutar herramientas ni guardar capturas. Una auditoría o laboratorio produce contexto, comandos, stdout/stderr, archivos, notas y decisiones que deben poder relacionarse entre sí.

El Discovery descartó competir como gestor genérico de findings o simple grabador de terminal. El foco aprobado es un **motor verificable de procedencia, integridad, autenticidad futura, transformación y reproducción de evidencia de pruebas de seguridad**, acompañado por un motor de aprendizaje aplicado.

## Problema

Los flujos manuales pueden provocar:

- evidencia dispersa entre terminales, carpetas, capturas y notas;
- pérdida del contexto exacto de ejecución;
- mezcla accidental de targets, entornos o engagements;
- dificultad para distinguir RAW de evidencia derivada o redactada;
- revalidación costosa porque no se conserva una receta reproducible;
- exposición accidental de secretos o datos sensibles;
- aprendizaje superficial: se ejecuta una técnica sin documentar qué demuestra, limitaciones o validación;
- dependencia de una aplicación para “creer” la evidencia en lugar de poder verificarla externamente.

## Usuarios

TATACOA se diseña para:

1. profesionales que realizan evaluaciones de seguridad autorizadas;
2. personas que estudian y practican en laboratorios propios o plataformas autorizadas;
3. revisores que necesitan comprobar integridad, procedencia y contexto sin confiar ciegamente en la aplicación.

## Principios

- local-first;
- cero telemetría por defecto;
- alcance autorizado;
- Validación Humana;
- preservar originales;
- procedencia explícita;
- verificación portable/offline;
- reproducibilidad;
- minimización de datos;
- secure-by-design;
- estándares como insumo trazable de ingeniería, no como marketing;
- aprender haciendo y poder demostrar lo aprendido.

## Tres motores

### Scope / Context
Evita que una ejecución exista sin contexto operativo identificable.

### Evidence
Captura, identifica, preserva, deriva, exporta y verifica artefactos.

### Knowledge / Learning
Relaciona la ejecución con objetivos, observaciones, límites, validación, contexto defensivo y fuentes.

Replay conecta los tres: permite repetir una prueba, hacer retest y convertir una técnica validada en receta, script o miniherramienta trazable sin convertirla automáticamente en un exploit genérico.

## Criterio de éxito

El receptor de un bundle no debe tener que “creerle” a TATACOA. Debe poder verificar lo verificable con una especificación portable y un verificador independiente.
