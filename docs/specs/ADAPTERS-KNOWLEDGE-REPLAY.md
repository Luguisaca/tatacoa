# Adapters, Knowledge y Replay

## Adapter contract

El Core gobierna. Un adapter interpreta y enriquece.

### Puede

- identificar compatibilidad;
- describir herramienta/versión observada;
- preparar argumentos dentro del contrato;
- solicitar artifacts;
- parsear output como entrada hostil;
- enriquecer metadata y conocimiento.

### No puede

- validar una vulnerabilidad;
- promover automáticamente Candidate a Evidence;
- cambiar scope/engagement;
- modificar RAW;
- eliminar artifacts;
- ocultar errores;
- acceder a otros engagements;
- usar red silenciosamente;
- elevar privilegios por conveniencia.

Primer adapter: **Generic Execution Adapter**. No comenzar Alpha por Nmap.

### Foundation SP3-12 de asistencia offline

La captura genérica no depende de conocer la herramienta. Tres capacidades permanecen separadas: hechos observados derivados del manifest de Execution, documentación local suministrada por un proveedor registrado explícitamente y adapter especializado opcional. La ausencia de las dos últimas produce `UNAVAILABLE` sin degradar captura ni integridad. `GENERIC`, `DOCUMENTED` y `ADAPTED` son niveles descriptivos de asistencia, no assurance ni estado de Evidence.

Los hechos observados señalan el campo de manifest/artifact de origen. Un hecho documentado exige un archivo local regular, no symlink, legible como UTF-8 y limitado a 64 KiB; señala su ruta y el SHA-256 de los bytes leídos. Ese digest permite identificar los bytes, no certifica su veracidad. Si el archivo no existe, es inválido o supera el límite, la documentación queda `UNAVAILABLE`. La interpretación humana se registra por separado como Knowledge borrador. Ninguna documentación se convierte automáticamente en conclusión, vulnerabilidad o Evidence.

La implementación inicial no registra proveedores/adapters de producción ni ejecuta probes. No presupone `--help`, `-h`, `--version`, `man` o sintaxis GNU. Los contratos de proveedor/adapter se prueban con fixtures locales; un adapter futuro debe justificar por herramienta cualquier identificación, version probe, help probe o parser antes de ejecutarlo. Un probe documental no pertenece al engagement, no altera provenance y no crea artifacts/evidencia de la Execution.

Lifecycle:

`PROPOSED → RESEARCHED → EXPERIMENTAL → VALIDATED → SUPPORTED → DEPRECATED`

No hay runtime de plugins arbitrarios de terceros en V1.

## Tool Knowledge Registry

Cada integración especializada debe poder documentar:

- fuente oficial/upstream;
- propósito;
- inputs/outputs;
- versión/compatibilidad;
- riesgos;
- interpretación;
- limitaciones;
- mappings relevantes.

Decisión de adapter:

`tool discovered → research → useful? → generic capture sufficient? → knowledge only | specialized adapter`

## Learning Mode

Debe poder explicar:

- WHAT;
- WHY;
- OBJECTIVE;
- HOW;
- OBSERVE;
- LIMITATIONS;
- REFERENCES;
- RELATED TECHNIQUES;
- DEFENSIVE CONTEXT.

Professional Mode podrá reducir explicaciones sin cambiar las propiedades de evidencia.

## Replay

Replay es una capacidad central, no un accesorio.

`Execution → Candidate → Validated Evidence → Replay Recipe → script/minitool → new Execution → Retest`

Una receta debe conservar:

- contexto requerido;
- parámetros;
- placeholders de secretos;
- prerequisites;
- límites de autorización;
- vínculo con ejecución/evidencia fuente;
- versión y procedencia.

Generar o reutilizar una receta no autoriza ejecutarla fuera de scope.

## Plataformas de aprendizaje

El diseño debe funcionar con laboratorios propios y plataformas autorizadas de práctica/formación. El contenido propietario de certificaciones o plataformas no se copia al repositorio; las Knowledge Cards se basan en práctica propia y fuentes permitidas.
