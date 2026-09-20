# Sprint 03 — Usable Alpha / Application Layer & Desktop Foundation

## Estado

**APPROVED / PLANNED — NO IMPLEMENTADO.**

Este documento define el alcance aprobado. No declara que las capacidades aquí descritas existan todavía.

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

El spike no es una demo de tres pantallas ni autoriza a reducir el flujo de producto.

## Continuidad y retest

Pausar es una acción explícita útil, pero la continuidad no depende de que la persona recuerde pulsar “Pausar”. TATACOA debe persistir suficiente estado de forma segura para recuperarse también de cierres accidentales/fallos.

El resumen es bajo demanda y sirve tanto para continuar trabajo propio como para comprender un proyecto recibido o preparar un retest. No debe inventar hechos: deriva de estado, executions, provenance y evidencia disponibles, diferenciando hechos registrados de inferencias/pending.

## Importación y autorización

Un paquete se trata como entrada no confiable. Abrir/importar requiere validación estructural, integridad y políticas aplicables antes de exponerlo como proyecto continuable.

La autorización para acceder al contenido sensible sigue el modelo de seguridad de TATACOA. Importar nunca reescribe provenance ni atribuye al operador actual acciones históricas de otra persona.

## RFC 3161 — gate de diseño

RFC 3161 forma parte de SP3 como diseño + primera implementación funcional.

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
