# AGENTS.md — T·A·T·A·C·O·A

## Alcance

Estas reglas aplican a cualquier agente de IA o automatización que trabaje sobre este repositorio.

T·A·T·A·C·O·A significa **Test · Analyze · Trace · Assure · Contextualize · Observe · Apply** y es un producto de LUGUISACA para trabajo de seguridad autorizado, evidencia verificable, pruebas reproducibles, continuidad y aprendizaje aplicado.

## Orden obligatorio de contexto

Antes de proponer o implementar cambios sustanciales, el agente debe:

1. inspeccionar rama, HEAD y estado real del repositorio;
2. leer `README.md`;
3. leer `docs/PROJECT.md`, `docs/ROADMAP.md`, `docs/ARCHITECTURE.md` y `docs/DECISIONS.md`;
4. leer `docs/CURRENT-STATE.md`, PRD, seguridad, specs y el plan/documentos del sprint aplicables;
5. distinguir explícitamente **implementado**, **pendiente de QA**, **aprobado/planificado**, **horizonte** y **decisión pendiente**;
6. continuar el roadmap vigente en vez de inventar uno nuevo.

Una capacidad documentada como futura/horizonte no constituye permiso para implementarla.

## Autoridad

1. Las instrucciones explícitas de la Validación Humana tienen prioridad sobre propuestas del agente.
2. `main` es protegida por proceso: ningún agente trabaja directamente sobre ella salvo autorización explícita y acotada.
3. No hacer merge, release, publicación, despliegue, cambio de licencia ni visibilidad sin aprobación humana explícita.
4. No ampliar alcance silenciosamente.
5. Ante decisión irreversible, destructiva, criptográfica, de seguridad o arquitectura no aprobada, detenerse y solicitar revisión.
6. Codex/agentes ejecutan tareas concretas ya analizadas/aprobadas; no tienen autoridad autónoma para redefinir roadmap, misión, arquitectura o modelo de seguridad.

## Conservación arquitectónica

- `tatacoa-core` es la autoridad de dominio, lógica, seguridad, evidencia, políticas y cifrado.
- CLI y Desktop son interfaces independientes de primera clase sobre una semántica común.
- El CLI no debe depender de Tauri/GUI.
- Desktop no debe exigir CLI preinstalado.
- `tatacoa-app-api` está aprobado para SP3 como capa común de operaciones; no implica backend cloud/daemon/red.
- Una interfaz nueva no puede convertirse en autoridad del producto ni duplicar reglas de seguridad.
- TATACOA es local-first/offline-capable; no introducir SaaS, cuenta cloud, telemetría o conexión permanente como requisito implícito.
- Si una capacidad requiere red por naturaleza, declararlo y diseñar su ausencia/fallo explícitamente.

## Forma de trabajo

- Trabajar en ramas dedicadas y cambios pequeños, revisables y trazables.
- Mantener commits con propósito único y mensajes descriptivos.
- No reescribir historial, forzar pushes ni eliminar ramas/datos sin autorización.
- No introducir dependencias por conveniencia.
- Preferir implementaciones simples y verificables sobre abstracciones prematuras.
- Un sprint extiende el producto; no invalida decisiones anteriores silenciosamente.
- En SP3, avanzar dentro del alcance aprobado de `docs/SPRINT-03-PLAN.md` sin pedir aprobación por detalles menores ya cubiertos; detenerse solo ante bloqueo real, gate humano, conflicto arquitectónico/de seguridad o acción que requiera autorización.
- Si una necesidad nueva entra en conflicto con arquitectura/decisión congelada, documentar el conflicto y esperar Validación Humana/ADR.

## Investigación y fuentes

Para decisiones técnicas o de seguridad externas:

- priorizar documentación oficial del lenguaje, biblioteca, estándar o proveedor;
- para seguridad, priorizar estándares y organismos reconocidos;
- distinguir requisitos del proyecto, recomendaciones externas y decisiones;
- registrar decisiones relevantes mediante documentación/ADR;
- no presentar recomendación/estándar/borrador como certificación o cumplimiento.

RFC 3161 está aprobado para SP3 como diseño + primera implementación funcional. Antes de codificar el sellado, el agente debe investigar fuentes oficiales y detenerse para Validación Humana sobre el objeto(s) a timestamp-ear y el momento exacto del ciclo. No seleccionar por inferencia una TSA, política u objeto de sellado; fallos/ausencia de TSA deben ser explícitos y nunca degradar silenciosamente evidencia existente.

## Seguridad

- TATACOA está diseñado para pruebas autorizadas.
- Nunca incorporar credenciales, tokens, claves, secretos o datos reales de clientes.
- Tratar artefactos, manifests, imports y salida de herramientas como entrada no confiable.
- No crear criptografía propia.
- No debilitar controles para hacer pasar una prueba.
- No convertir automáticamente resultados de herramientas/IA en vulnerabilidades o evidencia validada.
- Preservar RAW; derivaciones nunca sustituyen silenciosamente originales.
- Fallos de seguridad, integridad, captura o continuidad deben ser explícitos.

## Código

- Rust es la tecnología principal para Core, CLI y verifier.
- Tauri 2 está aprobado para `tatacoa-desktop` en SP3.
- Core mantiene baseline multiplataforma Windows + Linux.
- Evitar `unsafe` propio salvo justificación y revisión.
- Ejecutar herramientas mediante executable + args cuando sea posible; shell debe ser explícito.
- No implementar parsers especializados antes de definir contrato/límites.
- Todo comportamiento de seguridad relevante requiere pruebas negativas.

## Documentación

- Español de Colombia es el idioma principal interno, salvo artefactos interoperables/públicos que deban estar en inglés.
- Describir estado real, no capacidades futuras como existentes.
- Separar requisitos, decisiones, hipótesis, pendientes y resultados QA.
- Mantener PROJECT/ROADMAP/ARCHITECTURE/DECISIONS coherentes cuando una decisión aprobada cambie la dirección.
- Documentos de sprint son registros históricos; no reescribirlos para fingir que decisiones futuras ya existían.
- No inventar métricas, certificaciones, compatibilidad, resultados ni garantías.
- `LICENSE` y `NOTICE` no se modifican ni reinterpretan sin autorización humana.
- Claims como “FIPS validated”, “ISO certified”, “forensically certified”, “tamper-proof”, “unhackable” o equivalentes están prohibidos sin evidencia formal.

## IA y conocimiento

- IA puede asistir investigación, documentación, pruebas y desarrollo, pero no es autoridad de validación.
- Contenido factual asistido por IA debe poder rastrearse a fuentes revisables.
- Conocimiento comunitario puede orientar, no reemplaza fuentes oficiales para decisiones críticas.

## Definition of Done mínima

Un cambio no termina porque compile. Según alcance incluye:

- coherencia con misión, roadmap y decisiones aprobadas;
- pruebas aplicables y negativas;
- manejo explícito de errores;
- revisión de seguridad;
- documentación sincronizada;
- ausencia de secretos;
- QA reproducible;
- aprobación humana antes de integrar a `main`.

## Regla de conservación

Si una instrucción nueva entra en conflicto con una decisión congelada, no reinterpretarla silenciosamente. Señalar el conflicto y esperar decisión humana.
