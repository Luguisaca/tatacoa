# T·A·T·A·C·O·A

**Test · Assess · Trace · Artifacts · Comprehend · Observe · Apply**

*A local-first security assessment platform for verifiable evidence, reproducible testing and applied learning.*

## Por qué existe

Una prueba de seguridad puede terminar repartida entre terminales, archivos, capturas, notas y memoria humana. Eso dificulta demostrar qué se ejecutó, contra qué contexto, qué artefacto produjo un resultado, si el artefacto cambió y cómo repetir la prueba meses después.

El problema también existe al estudiar: ejecutar una técnica no significa comprender qué demuestra, qué no demuestra, cómo validarla ni cómo reproducirla.

TATACOA une ambos frentes:

**Probar → Evaluar → Trazar → Preservar evidencia → Comprender → Observar → Aplicar lo aprendido.**

## Objetivo

Crear una plataforma local-first para auditorías autorizadas, laboratorios y aprendizaje que:

- preserve contexto, ejecución y artefactos;
- mantenga identidad criptográfica y procedencia;
- permita verificación independiente y offline;
- facilite replay, retest y construcción trazable de scripts/miniherramientas;
- convierta la práctica en conocimiento documentado;
- mantenga la Validación Humana como autoridad sobre resultados y evidencia.

## Alpha

La primera vertical aprobada es:

`ENGAGEMENT → CONTEXT → EXECUTION → ARTIFACT → SHA-256 → MANIFEST → EXPORT → VERIFY`

El Alpha incluye perfiles de seguridad configurables y un Generic Execution Adapter. No incluye scanner propio, explotación autónoma, SaaS, GUI, reporting empresarial ni validación automática de vulnerabilidades.

## Documentación

- [Visión y problema](docs/PROJECT.md)
- [PRD y alcance](docs/PRD.md)
- [Arquitectura](docs/ARCHITECTURE.md)
- [Threat model](docs/security/THREAT-MODEL.md)
- [Modelo de evidencia y procedencia](docs/specs/EVIDENCE-AND-PROVENANCE.md)
- [Privacidad y criptografía](docs/security/PRIVACY-CRYPTO.md)
- [Adapters, aprendizaje y replay](docs/specs/ADAPTERS-KNOWLEDGE-REPLAY.md)
- [Decisiones congeladas](docs/DECISIONS.md)
- [Trazabilidad y referencias](docs/TRACEABILITY.md)
- [Roadmap Alpha](docs/ROADMAP.md)
- [Implementación Alpha Foundation](docs/ALPHA-IMPLEMENTATION.md)

Las reglas para agentes, contribución, seguridad y gobierno están en `AGENTS.md`, `CONTRIBUTING.md`, `SECURITY.md` y `docs/GOVERNANCE.md`.

## Estado

**Sprint 01 — Alpha Foundation en implementación y QA.** La primera vertical funcional en Rust fue validada en Windows 11 x64 y en WSL2/Ubuntu x86_64 como integración Linux temprana, incluyendo verificación offline, controles negativos y portabilidad bidireccional de bundles entre ambos entornos. Kali Linux y Parrot OS continúan pendientes y WSL2 no los sustituye. El detalle y los límites de los resultados están en `docs/ALPHA-IMPLEMENTATION.md`.

A LUGUISACA project.
