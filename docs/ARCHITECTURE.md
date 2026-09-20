# TATACOA — Arquitectura de producto

## Estado

**BASELINE APROBADA + DIRECCIÓN ARQUITECTÓNICA APROBADA PARA SP3.**

Este documento distingue componentes existentes de componentes planificados. Una capacidad planificada nunca debe presentarse como implementada.

## Principio rector

La interfaz no es la autoridad del producto. Reglas de dominio, seguridad, evidencia, políticas, cifrado y invariantes pertenecen al Core o a capas comunes expresamente aprobadas.

CLI y Desktop deben ser instalaciones independientes de primera clase y operar sobre la misma semántica de dominio.

## Arquitectura objetivo

```text
                 TATACOA
                    │
          ┌─────────┴─────────┐
          │                   │
     tatacoa-cli        tatacoa-desktop
          │                   │
          │             tatacoa-app-api
          │                   │
          └─────────┬─────────┘
                    │
              tatacoa-core
                    │
       workspace / context / evidence
       policy / crypto / replay / provenance
```

El diagrama expresa responsabilidades, no obliga a que CLI dependa literalmente de `tatacoa-app-api` si hacerlo perjudica su instalación independiente. La regla es que **no se duplique lógica de dominio** y que las operaciones de usuario compartan contratos coherentes.

## Componentes existentes

### tatacoa-core
Autoridad para reglas de dominio, IDs, estados, invariantes, contexto, workspace, políticas, evidencia, provenance y criptografía aprobada.

### tatacoa-cli
Interfaz de línea de comandos. Debe continuar siendo funcional sin Desktop, Tauri ni servidor local obligatorio.

### tatacoa-verifier
Componente separado, read-only y offline. Trata bundles/manifests como entrada hostil y verifica estructura, digest y relaciones sin ejecutar su contenido.

### Execution / Evidence / Manifest / Export
Capacidades del Core para ejecución contextualizada, captura, finalización de artefactos, digest, manifest portable, provenance y políticas de export.

### Knowledge / Replay
Relaciona ejecuciones con conocimiento y recetas reproducibles. No es autoridad de validación.

## Componentes aprobados para SP3 — no implementados

### tatacoa-app-api
Capa común de operaciones de usuario entre las interfaces y el dominio. Debe reducir duplicación y evitar que Desktop introduzca reglas propias.

Su diseño debe preservar la independencia del CLI. No debe convertirse en daemon, servicio de red o backend cloud por defecto.

### tatacoa-desktop
Aplicación local construida con **Tauri 2**. Presenta el flujo de trabajo de la pentester y consume operaciones aprobadas sin reimplementar seguridad o dominio.

Desktop no requiere una instalación previa del CLI y no convierte Tauri en dependencia del CLI/Core.

## Modalidades de instalación

### CLI-only
Instalación de TATACOA orientada a terminal/automatización. No arrastra runtime gráfico ni requiere Desktop.

### Desktop
Instalación de la aplicación gráfica completa. Empaqueta lo necesario para operar sin exigir que el usuario instale el CLI por separado.

Ambas modalidades deben preservar interoperabilidad de los formatos/datos soportados y las mismas políticas/invariantes de seguridad.

## Local-first y offline

Las funciones locales de TATACOA no dependen de SaaS, cuenta cloud, telemetría ni conexión permanente.

Una capacidad futura que requiera red debe:

1. declarar explícitamente la dependencia;
2. fallar de forma comprensible y segura cuando no esté disponible;
3. no degradar silenciosamente garantías;
4. no convertir conectividad en requisito para funciones locales no relacionadas.

Esto es especialmente relevante para servicios externos potenciales como una TSA de RFC 3161.

## Persistencia y continuidad

Baseline actual: filesystem + manifests; no existe requisito de base de datos obligatoria.

La arquitectura debe evolucionar para permitir pausa/reanudación segura del trabajo, recuperación tras cierre/fallo y resumen de continuidad sin convertir estado incompleto en evidencia validada.

La persistencia de continuidad no puede alterar silenciosamente RAW, provenance, estados de captura o políticas de seguridad.

## Modelo de dominio

Vertical fundamental:

`ENGAGEMENT → CONTEXT → EXECUTION → ARTIFACT → SHA-256 → MANIFEST → EXPORT → VERIFY`

El contexto expandido incluye Scope, Environment, Target y Session. Knowledge y Replay relacionan la vertical con aprendizaje/retest.

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

- Rust: Core, CLI, verifier y lógica común.
- Tauri 2: Desktop aprobado para SP3.
- Tecnologías web de UI: se decidirán/confirmarán dentro del diseño Desktop sin trasladar autoridad de seguridad al frontend.
- Python: investigación, tooling de pruebas y adapters/parsers experimentales cuando sea útil.

No introducir frameworks, runtimes o dependencias por conveniencia. Toda dependencia debe responder a una necesidad aprobada.

## Portabilidad

El Core se diseña como multiplataforma Windows + Linux. Windows 11 x64 y WSL2 son targets de desarrollo/integración ya usados; Kali Linux x64 y Parrot OS x64 permanecen targets de QA específicos.

El Core debe aislar diferencias de plataforma y evitar asumir como universales rutas, shells, permisos, señales, ejecutables o semánticas propias de Windows/POSIX.

La compatibilidad del Core y la de un adapter/herramienta son conceptos separados.

## Evolución

La arquitectura debe considerar el horizonte de [ROADMAP.md](ROADMAP.md) sin implementar anticipadamente capacidades no aprobadas.

Si un sprint futuro requiere romper una decisión congelada, no se adapta silenciosamente: se documenta el conflicto, se evalúan migración/compatibilidad y seguridad, y se solicita Validación Humana/ADR.
