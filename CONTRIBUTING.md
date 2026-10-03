# Contribuir a T·A·T·A·C·O·A

## Principio

Los cambios deben ser revisables, trazables y coherentes con el alcance autorizado del proyecto. El tamaño del commit debe responder a una unidad lógica de cambio: el código favorece incrementos pequeños; un baseline documental coherente puede agruparse cuando dividirlo reduzca claridad o agilidad.

## Flujo

1. Partir de `main` actualizado.
2. Crear una rama dedicada al cambio.
3. Revisar documentación y decisiones aplicables antes de implementar.
4. Realizar commits por unidades lógicas y revisables.
5. Ejecutar las comprobaciones aplicables.
6. Abrir Pull Request hacia `main`.
7. No hacer merge sin revisión y aprobación de la Validación Humana.

## Ramas

Usar nombres descriptivos, por ejemplo:

- `feat/<tema>`
- `fix/<tema>`
- `docs/<tema>`
- `test/<tema>`
- `chore/<tema>`
- `security/<tema>`

Los sprints pueden usar ramas como `feat/sprint-01-foundation` cuando agrupen una fase controlada de trabajo.

## Commits

Preferir mensajes en formato:

`tipo: descripción breve`

Tipos habituales: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`, `security`.

No mezclar cambios no relacionados en el mismo commit.

## Pull Requests

Cada PR debe explicar qué cambia, por qué cambia, cómo fue validado, impacto de seguridad conocido y pendientes deliberadamente fuera de alcance.

Un PR no debe afirmar PASS de QA si las pruebas correspondientes no fueron ejecutadas.

## Dependencias

Antes de añadir una dependencia debe existir una necesidad concreta. Para componentes sensibles se revisarán como mínimo mantenimiento, procedencia, seguridad conocida, licencia y adecuación técnica.

## Seguridad

No publicar vulnerabilidades del propio proyecto mediante un Issue público cuando exista un canal privado de reporte. Ver `SECURITY.md`.

No incluir secretos, datos reales de clientes ni material fuera del alcance autorizado.

## Autoridad de integración

La aprobación para integrar a `main` corresponde a la Validación Humana. Un agente no debe interpretar una solicitud de revisión como autorización de merge.
