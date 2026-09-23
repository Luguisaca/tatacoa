# Sprint 03 — Usable Alpha / Application Layer & Desktop Foundation

## Estado

**IMPLEMENTACIÓN INCREMENTAL EN CURSO — QA HUMANO INTEGRAL PENDIENTE.**

Este documento conserva el alcance aprobado de Sprint 03. El estado de cada bloque realmente construido y sus gates se registra en `SPRINT-03-IMPLEMENTATION.md` y `CURRENT-STATE.md`; una capacidad descrita aquí no se considera aprobada como producto hasta superar el QA correspondiente.

## Objetivo

Convertir el baseline de TATACOA en una Alpha realmente utilizable por una pentester para comenzar QA humano del producto de extremo a extremo, preservando Core, CLI, seguridad, evidencia y decisiones anteriores.

El éxito no es “Tauri compila”. El éxito es que una persona pueda iniciar/abrir un trabajo autorizado, operar sobre él, consultar su evidencia/contexto, interrumpirlo y retomarlo sin perder continuidad ni provenance.

## Entregables

1. Crear `tatacoa-app-api` como capa de operaciones de usuario sobre `tatacoa-core`.
2. Mantener `tatacoa-cli` funcional e instalable sin GUI/Tauri.
3. Crear `tatacoa-desktop` con Tauri 2, instalable sin CLI preinstalado.
4. Mantener `tatacoa-core` como única autoridad de dominio, seguridad, evidencia, políticas y cifrado.
5. Implementar una GUI orientada al flujo real de una pentester, no a exponer métodos internos.
6. Integrar el flujo de workspace/trabajo, contexto autorizado, Scope/Environment/Target/Session, Execution, Artifact/Evidence, Knowledge, Replay y Retest según capacidades existentes.
7. Implementar continuidad: guardado seguro, pausa, reanudación y recuperación tras cierre/fallo.
8. Permitir resumen bajo demanda al abrir, reanudar o hacer retest: acciones realizadas, ejecuciones/evidencia relevantes, contexto conocido, pendientes y punto de continuidad.
9. Al reanudar, distinguir estado histórico guardado de estado actual del entorno/autorización y permitir/requerir revalidación cuando corresponda.
10. Abrir/importar paquetes TATACOA compatibles como proyectos continuables para personas autorizadas, validando antes de confiar y preservando provenance original.
11. Integrar las capacidades existentes —Security Profiles, Knowledge Cards, Replay Recipes, bundles y Encrypted v1— dentro de la experiencia utilizable para habilitar QA real.
12. Realizar un spike Tauri de extremo a extremo y convertir sus conclusiones aprobadas en la implementación.
13. Verificar build/empaquetado y comportamiento aplicable en Windows y Linux según los targets documentados.
14. Implementar RFC 3161 / trusted timestamping después de superar su gate de diseño humano.
15. Sincronizar documentación y QA con el comportamiento realmente implementado.

## Contrato de experiencia operativa Desktop

La Usable Alpha no puede convertir el modelo interno de TATACOA en trabajo administrativo para la pentester. Desktop debe **acompañar el trabajo técnico**, capturar lo que ya conoce y pedir intervención humana solo cuando aporte información que no puede inferirse con seguridad o cuando una decisión de seguridad/autorización lo requiera.

### Automático por defecto

Cuando la información ya existe en el contexto o nace de una operación ejecutada por TATACOA, Desktop debe reutilizarla sin pedir que la persona la transcriba:

- engagement, Scope, Environment, Target y Session activos;
- fecha/hora observada, executable, argv y resultado de una Execution;
- stdout/stderr y artifacts producidos, con identidad, digest, provenance y estado de captura;
- relaciones entre Execution, artifacts, export, replay/retest y demás objetos derivados cuando el Core pueda establecerlas;
- estado de continuidad derivable de datos persistidos.

Automático no significa validado: TATACOA no inventa hechos ni promueve por sí solo resultados a Evidence validada.

### Asistido y opcional

Knowledge/Learning y Replay/Retest son capacidades centrales, pero su representación interna no debe imponerse como formulario obligatorio del flujo normal.

- La experiencia debe partir de la Execution/evidencia ya registrada y ofrecer acciones comprensibles como **entender resultado**, **añadir nota/contexto**, **preparar reproducción/retest** o **comparar**.
- Los campos estructurados de Knowledge pueden conservarse en Core, pero Desktop debe precargar/derivar lo que ya conoce y solicitar al humano únicamente interpretación, validación, intención o contexto que no pueda inferir legítimamente.
- Replay debe reutilizar executable, argumentos, contexto y provenance de la Execution de origen; la persona decide placeholders, secretos, prerequisites, límites de autorización y cualquier cambio deliberado.
- En `LAB_LEARNING`, la interfaz puede hacer más visible la explicación y aprendizaje. En `PROFESSIONAL`, debe minimizar fricción sin perder trazabilidad. `HIGH_SENSITIVITY` prioriza las políticas aplicables y minimización de exposición.

La asistencia factual, incluida IA futura, nunca sustituye fuentes revisables ni Validación Humana y no entra automáticamente en SP3 si no está aprobada.

### Explícito por seguridad o decisión humana

Debe permanecer visible y deliberado aquello que TATACOA no puede asumir:

- definición inicial y cambios de Scope/límite de autorización;
- revalidación de autorización al reanudar/ejecutar cuando corresponda;
- promoción/revisión humana de Evidence;
- decisiones de exportación que la política permita elegir y acknowledgements requeridos;
- passwords, trust anchors, TSA/policies y otros secretos/configuración de confianza explícita;
- cambios deliberados para replay/retest que alteren la operación original.

### Feedback y continuidad

Toda acción iniciada por la persona debe producir feedback visible y persistente en su contexto. Guardar Knowledge, preparar Replay, exportar o timestamp-ear no puede depender de un mensaje fuera de pantalla ni dejar al usuario sin forma de localizar el resultado.

Al abrir o retomar un trabajo, Desktop debe priorizar una vista de continuidad derivada del estado real: qué se hizo, ejecuciones/evidencia relevantes, qué quedó pendiente y desde dónde continuar. La persona no debe reconstruir manualmente esa historia a partir de formularios internos.

### Criterio UX del sprint

Una capacidad del Core **no se considera integrada como producto únicamente porque exista un formulario que invoque su método**. Para SP3 debe estar incorporada al flujo real de trabajo con reutilización de contexto, feedback, trazabilidad y fricción proporcional a la decisión humana requerida.

## Spike obligatorio

Debe probar como mínimo:

`crear/abrir workspace → crear trabajo → definir contexto autorizado → registrar/visualizar ejecución → abrir evidencia → guardar/pausar → cerrar → volver a abrir → recuperar → resumir → continuar`

También debe comprobar:

- comunicación tipada Desktop → app-api → Core;
- ausencia de autoridad de seguridad en frontend;
- manejo de password sin logs ni persistencia indebida;
- capabilities/CSP y superficie mínima aplicables a Tauri;
- ausencia de dependencia general de red;
- artifacts de texto e imagen, incluidos tamaños relevantes;
- cierre durante una operación y recuperación segura;
- accesibilidad básica;
- build/empaquetado Windows/Linux.
- artefacto distribuible trazable al commit probado y QA de cliente limpio sin toolchain de desarrollo como requisito de usuario.

El spike no es una demo de tres pantallas ni autoriza a reducir el flujo de producto.

## Continuidad y retest

Pausar es una acción explícita útil, pero la continuidad no depende de que la persona recuerde pulsar “Pausar”. TATACOA debe persistir suficiente estado de forma segura para recuperarse también de cierres accidentales/fallos.

El resumen es bajo demanda y sirve tanto para continuar trabajo propio como para comprender un proyecto recibido o preparar un retest. No debe inventar hechos: deriva de estado, executions, provenance y evidencia disponibles, diferenciando hechos registrados de inferencias/pending.

## Importación y autorización

Un paquete se trata como entrada no confiable. Abrir/importar requiere validación estructural, integridad y políticas aplicables antes de exponerlo como proyecto continuable.

La autorización para acceder al contenido sensible sigue el modelo de seguridad de TATACOA. Importar nunca reescribe provenance ni atribuye al operador actual acciones históricas de otra persona.

La Validación Humana aprobó para SP3 que el paquete recibido se conserve intacto, con provenance original, y que su apertura continuable ocurra solo después de verificación y revalidación de autorización. Importar no promueve automáticamente ningún Artifact a Evidence.

## RFC 3161 — gate de diseño

RFC 3161 forma parte de SP3 como diseño + primera implementación funcional.

Para esta Usable Alpha, `HISTORICALLY_VALIDATED` permanece bloqueado por decisión humana. El alcance implementado llega hasta `TRUSTED` bajo la política explícita aprobada; ausencia de evaluación de revocación histórica se informa como `INDETERMINATE`.

Antes del código se debe resolver con fuentes oficiales y aprobación humana:

- objeto(s) exactos a timestamp-ear;
- momento exacto del ciclo de evidencia/bundle.

El diseño debe además especificar TSA/política configurable, representación persistida, verificación posterior, interacción con bundles/importación y comportamiento cuando TSA/red no estén disponibles. No se degrada evidencia existente por no obtener timestamp.

## Definition of Done de Sprint 03

- flujo mínimo utilizable demostrado de extremo a extremo;
- CLI continúa funcional sin Desktop;
- Desktop funciona sin CLI preinstalado;
- Core conserva autoridad e invariantes;
- continuidad funciona tras pausa, cierre normal y escenario de fallo cubierto;
- paquete compatible puede validarse/abrirse según contrato aprobado;
- Encrypted v1 puede probarse dentro del flujo real de usuario;
- RFC 3161 supera su gate y su alcance aprobado queda implementado/probado, o el sprint queda explícitamente BLOCKED si una decisión humana pendiente impide hacerlo;
- pruebas positivas y negativas aplicables;
- format/lint/check/test y QA reproducible;
- seguridad y documentación sincronizadas;
- ningún resultado técnico se presenta como certificación;
- aprobación humana antes de merge a `main`.

## Fuera de alcance

No implementar por inferencia reporting avanzado, colaboración, public-key recipients, signatures, encrypted workspace, hardware keys/PKCS#11/TPM/KMS, plugin sandboxing o AI local.

## Regla para Codex/agentes

Avanzar dentro de este alcance salvo bloqueo real o gate humano. No pedir autorización por cada detalle de implementación cubierto por decisiones existentes. No inventar roadmap, reducir el producto a un prototipo ni declarar completado algo que la persona todavía no puede usar según este flujo.
