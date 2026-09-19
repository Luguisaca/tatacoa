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

Kali y Parrot son los primeros entornos de QA. El Core debe evitar supuestos innecesarios de distribución para conservar portabilidad futura.
