# T·A·T·A·C·O·A

**Test · Assess · Trace · Artifacts · Comprehend · Observe · Apply**

TATACOA es un producto de **LUGUISACA — luguisaca.com** para organizar el trabajo real de una evaluación de seguridad autorizada: contexto, ejecuciones, artefactos, evidencia verificable, conocimiento y reproducción/retest, con una arquitectura local-first y Validación Humana como autoridad.

> **Estado:** baseline Alpha de Sprint 01 + Sprint 02 / Alpha Expansion + `tatacoa.encrypted.v1` y su hardening de password integrado y validado dentro del alcance documentado. Sprint 03 — Usable Alpha está aprobado/planificado y todavía no está implementado.

## Misión

Ayudar a profesionales de seguridad a ejecutar, preservar, comprender, retomar y demostrar su trabajo autorizado con contexto, trazabilidad e integridad, reduciendo la pérdida de evidencia y conocimiento sin convertir la automatización o la IA en autoridad de validación.

## Visión

Construir un espacio de trabajo local-first para pentesting y evaluación de seguridad que acompañe el ciclo técnico completo —desde el contexto autorizado hasta evidencia, aprendizaje, replay y retest— y permita que el trabajo siga siendo verificable, portable y útil más allá de una sesión, una interfaz o un equipo.

La evolución de TATACOA debe ampliar esta base sin obligar a rediseñar el producto en cada sprint. El destino del producto y sus capacidades objetivo se mantienen en [PROJECT.md](docs/PROJECT.md) y [ROADMAP.md](docs/ROADMAP.md).

## Arquitectura de producto

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

Actualmente existen `tatacoa-core`, `tatacoa-cli` y `tatacoa-verifier`. `tatacoa-app-api` y `tatacoa-desktop` son capacidades aprobadas para Sprint 03 y **todavía no están implementadas**.

CLI y Desktop son instalaciones independientes de primera clase. El CLI no requiere GUI/Tauri; Desktop no requiere una instalación previa del CLI. Ambos deben compartir la autoridad de dominio del Core y preservar interoperabilidad de los datos soportados.

TATACOA no requiere SaaS, cuenta cloud ni conexión permanente para sus funciones locales. Las capacidades que por naturaleza dependan de servicios externos deben declararlo explícitamente y no convertir Internet en requisito general.

## Estado y documentación

| Documento | Propósito |
|---|---|
| [PROJECT.md](docs/PROJECT.md) | identidad, misión, visión, problema y principios estables |
| [PRD.md](docs/PRD.md) | requisitos del producto y de la Alpha |
| [ROADMAP.md](docs/ROADMAP.md) | evolución acumulativa y horizonte de capacidades |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | arquitectura actual y dirección aprobada |
| [DECISIONS.md](docs/DECISIONS.md) | decisiones congeladas y pendientes |
| [TRACEABILITY.md](docs/TRACEABILITY.md) | fuentes y trazabilidad de ingeniería |
| [ALPHA-IMPLEMENTATION.md](docs/ALPHA-IMPLEMENTATION.md) | implementación/QA de Sprint 01 |
| [SPRINT-02-IMPLEMENTATION.md](docs/SPRINT-02-IMPLEMENTATION.md) | implementación de Sprint 02 |
| [ENCRYPTED-V1-IMPLEMENTATION.md](docs/ENCRYPTED-V1-IMPLEMENTATION.md) | estado de Encrypted Bundle v1 |
| [ALPHA-USER-QA-GUIDE.md](docs/ALPHA-USER-QA-GUIDE.md) | recorrido de QA humano |
| [SECURITY.md](SECURITY.md) | política y reporte de vulnerabilidades |
| [AGENTS.md](AGENTS.md) | reglas para agentes y automatizaciones |

Los documentos de implementación de cada sprint son registros históricos de lo construido. PROJECT, PRD, ROADMAP, ARCHITECTURE y DECISIONS gobiernan la dirección vigente del producto.

## Seguridad y uso responsable

TATACOA está diseñado para evaluaciones de seguridad autorizadas y laboratorios legítimos. Preserva originales, trata entradas y bundles como no confiables, mantiene la Validación Humana y no convierte automáticamente resultados de herramientas o IA en evidencia validada.

Las vulnerabilidades del propio proyecto deben reportarse mediante el canal privado definido en [SECURITY.md](SECURITY.md), no mediante Issues públicos.

## IA

El proyecto es dirigido y revisado por personas. Herramientas basadas en IA pueden apoyar investigación, documentación, pruebas y desarrollo, pero no sustituyen la Validación Humana ni tienen autoridad para redefinir arquitectura, seguridad o roadmap.

## Licencia

TATACOA se distribuye bajo la **PolyForm Noncommercial License 1.0.0**. El texto jurídicamente aplicable se encuentra en [LICENSE](LICENSE) y el aviso de autoría en [NOTICE](NOTICE).

El repositorio público no concede derechos adicionales a los establecidos por la licencia. Cualquier permiso comercial, si se concede, requiere una licencia independiente otorgada por el licenciante.

> `LICENSE` conserva el texto oficial en inglés. Esta explicación no sustituye ni modifica sus términos.

**A LUGUISACA project — luguisaca.com**
