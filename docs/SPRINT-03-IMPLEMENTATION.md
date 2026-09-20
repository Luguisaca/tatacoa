# Sprint 03 — registro de implementación

## Estado

**EN IMPLEMENTACIÓN — QA técnico parcial.** Este documento registra bloques realmente construidos; no declara completa la Usable Alpha.

## Bloque 01 — App API y flujo de trabajo

Rama local: `feat/sp3-01-app-api-workflow`.

Implementado:

- nuevo crate `tatacoa-app-api`, independiente de Tauri;
- operación tipada para crear el contexto inicial completo de un trabajo;
- ejecución contextualizada mediante el Core;
- listado validado de engagements, sessions y executions existentes;
- resumen derivado de estado persistido, sin inventar hechos ni duplicar reglas de dominio;
- reapertura de un workspace existente y reconstrucción del resumen desde disco.

El Core continúa validando IDs, relaciones, paths, aislamiento y captura. App API no accede directamente a JSON ni replica esas reglas.

QA técnico:

- flujo E2E de creación → ejecución → reapertura → resumen: PASS automatizado;
- regresión completa del workspace: PASS;
- Clippy con warnings como error: PASS.

QA humano propuesto cuando exista Desktop:

1. crear un trabajo completo sin copiar IDs;
2. ejecutar una herramienta autorizada;
3. cerrar la aplicación;
4. reabrir el workspace;
5. comprobar que session, execution, estado de captura y artifacts aparecen en el resumen.

Pendiente del siguiente bloque: Desktop/Tauri, integración visual y contratos de continuidad/importación que requieren diseño específico.
