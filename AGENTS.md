# AGENTS.md — T·A·T·A·C·O·A

## Alcance

Estas reglas aplican a cualquier agente de IA o automatización que trabaje sobre este repositorio.

T·A·T·A·C·O·A significa **Test · Assess · Trace · Artifacts · Comprehend · Observe · Apply** y es una plataforma local-first para evaluación de seguridad, evidencia verificable, pruebas reproducibles y aprendizaje aplicado.

## Autoridad

1. Las instrucciones explícitas de la Validación Humana para una tarea tienen prioridad sobre propuestas del agente.
2. `main` es una rama protegida por proceso: ningún agente debe trabajar directamente sobre ella.
3. No hacer merge, release, publicación, despliegue ni cambio de visibilidad sin aprobación explícita de la Validación Humana.
4. No ampliar el alcance de una tarea silenciosamente.
5. Ante una decisión irreversible, destructiva, criptográfica, de seguridad o arquitectura no aprobada, detenerse y solicitar revisión.

## Forma de trabajo

- Trabajar en ramas dedicadas y cambios pequeños, revisables y trazables.
- Mantener commits con propósito único y mensajes descriptivos.
- Inspeccionar el estado real del repositorio antes de modificarlo.
- Leer la documentación aplicable antes de implementar.
- No reescribir historial, forzar pushes ni eliminar ramas o datos sin autorización explícita.
- No introducir dependencias por conveniencia. Cada dependencia debe tener una necesidad identificable y ser revisada antes de incorporarse.
- Preferir implementaciones simples y verificables sobre abstracciones prematuras.

## Investigación y fuentes

Para decisiones técnicas o de seguridad que dependan de información externa:

- Priorizar documentación oficial del lenguaje, biblioteca, estándar o proveedor.
- Para controles y prácticas de seguridad, priorizar estándares y organismos reconocidos.
- Distinguir requisitos del proyecto, recomendaciones externas y decisiones de implementación.
- Registrar decisiones relevantes mediante documentación o ADR cuando corresponda.
- No presentar una recomendación, estándar o borrador como certificación o cumplimiento del producto.

## Seguridad

- TATACOA está diseñado para pruebas de seguridad autorizadas.
- Nunca incorporar credenciales, tokens, claves, secretos o datos reales de clientes al repositorio.
- Tratar artefactos, manifests, archivos importados y salida de herramientas como entrada no confiable.
- No crear criptografía propia.
- No debilitar validaciones o controles para hacer pasar una prueba.
- No convertir automáticamente resultados de herramientas o IA en vulnerabilidades o evidencia validada.
- Preservar originales: una derivación, redacción o transformación nunca sustituye silenciosamente el artefacto RAW.
- Los fallos de seguridad, integridad o captura deben ser explícitos; nunca degradar silenciosamente a un estado menos seguro.

## Código

- Rust es la tecnología principal aprobada para Core, CLI y verificador.
- El diseño debe ser compatible inicialmente con Kali Linux y Parrot OS y mantener portabilidad futura.
- Evitar `unsafe` en código propio salvo justificación técnica documentada y revisión específica.
- Ejecutar herramientas mediante ejecutable + argumentos cuando sea posible; el uso de shell debe ser explícito.
- No implementar parsers especializados antes de que el contrato genérico y sus límites estén definidos.
- Todo comportamiento de seguridad relevante requiere pruebas negativas además de pruebas de éxito.

## Documentación

- Español de Colombia es el idioma principal de documentación interna del proyecto, salvo artefactos que deban ser interoperables o públicos en inglés.
- La documentación debe describir el estado real, no capacidades futuras como si existieran.
- Mantener separados: requisitos, decisiones, hipótesis, pendientes y resultados de QA.
- No inventar métricas, certificaciones, compatibilidad, resultados de pruebas ni garantías.
- Claims como “FIPS validated”, “ISO certified”, “forensically certified”, “tamper-proof”, “unhackable”, “government approved” o equivalentes están prohibidos sin evidencia formal aplicable.

## IA y conocimiento

- La IA puede asistir investigación, documentación, pruebas y desarrollo, pero no es autoridad de validación.
- Contenido generado por IA que dependa de hechos externos debe poder rastrearse a fuentes revisables.
- El conocimiento comunitario puede orientar investigación, pero no reemplaza fuentes oficiales para decisiones críticas.

## Definition of Done mínima

Un cambio no se considera terminado solo porque compila. Según su alcance debe incluir:

- código/documentación coherente con requisitos aprobados;
- pruebas aplicables;
- manejo explícito de errores;
- revisión de impacto de seguridad;
- documentación actualizada cuando cambie comportamiento o arquitectura;
- ausencia de secretos;
- QA reproducible;
- aprobación humana antes de integrar a `main`.

## Regla de conservación

Si una instrucción nueva entra en conflicto con una decisión congelada del proyecto, no reinterpretarla silenciosamente. Señalar el conflicto y esperar decisión humana.
