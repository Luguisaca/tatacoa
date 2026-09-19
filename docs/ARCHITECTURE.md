# Arquitectura baseline

## Estado

**APROBADA para iniciar Alpha; detalles de implementación pueden evolucionar mediante ADR.**

## Vertical Alpha

`ENGAGEMENT → CONTEXT → EXECUTION → ARTIFACT → SHA-256 → MANIFEST → EXPORT → VERIFY`

## Componentes lógicos

### Core
Reglas de dominio, IDs, estados, invariantes y políticas. No debe depender de parsers específicos de herramientas.

### Execution
Lanza procesos como usuario normal mediante ejecutable + argv. Shell solo cuando sea explícito. Registra contexto y captura stdout/stderr de forma streaming.

### Evidence
Finaliza artefactos, calcula digest, registra metadatos, procedencia y derivaciones.

### Manifest
Representación portable, versionada y determinista. JSON UTF-8 es el candidato aprobado para Alpha.

### Verifier
Componente separado, read-only y offline. Trata bundle y manifest como entrada hostil. Verifica estructura, digest y relaciones sin ejecutar contenido.

### Security Profile / Export
Aplica política de confidencialidad a exportaciones sin desactivar integridad/procedencia.

### Generic Execution Adapter
Primer adapter obligatorio. Los adapters especializados enriquecen; no gobiernan evidencia ni validan vulnerabilidades.

### Knowledge / Replay
Relaciona ejecuciones con conocimiento y recetas reproducibles. No es autoridad de validación.

## Persistencia

Alpha: filesystem + manifests. No existe requisito de base de datos obligatoria.

Escritura segura conceptual:

`write → finalize → hash → manifest commit`

Solo después de completar la secuencia una captura puede marcarse `COMPLETE`.

## IDs

Prefijos reservados:

`eng_ scp_ env_ tgt_ ses_ exe_ art_ evd_ drv_ rpl_ knw_ bnd_`

El ID lógico de Artifact es independiente del digest criptográfico.

## Bundle portable

```text
bundle/
├── manifest.json
├── objects/
├── knowledge/
├── replay/
└── verification/
```

Los paths almacenados son relativos al bundle. El verificador debe rechazar traversal, relaciones inválidas y estructuras fuera de política.

## Tecnología

- Rust: Core, CLI, verifier, manifest, evidence y ejecución genérica.
- Python: investigación, tooling de pruebas y adapters/parsers experimentales cuando sea útil.
- TypeScript/Node: posible UI futura; no dependencia del Alpha.

Dependencias Rust no están aprobadas por nombre hasta realizar el spike correspondiente. No introducir frameworks o async runtime sin necesidad demostrada.

## Portabilidad

El Core V1 se diseña y desarrolla como **multiplataforma Windows + Linux**. La portabilidad forma parte del baseline de V1, no de una ampliación post-V1.

Targets iniciales:

- Windows 11 x64: desarrollo y QA nativo;
- WSL2 Linux x64: desarrollo e integración Linux temprana;
- Kali Linux x64: QA objetivo;
- Parrot OS x64: QA objetivo.

WSL2 es un entorno auxiliar de desarrollo/integración y no reemplaza el QA específico en Kali o Parrot.

El Core debe aislar diferencias de plataforma y evitar asumir como universales rutas, shells, permisos, señales, ejecutables, separadores o semánticas propias de Windows o POSIX.

La compatibilidad del Core y la compatibilidad de un adapter/herramienta son conceptos separados. Un adapter puede declarar plataformas soportadas o requisitos exclusivos de un sistema operativo sin convertir esa restricción en una limitación del Core.
