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

Las reglas para agentes, contribución, seguridad y gobierno están en `AGENTS.md`, `CONTRIBUTING.md`, `SECURITY.md` y `docs/GOVERNANCE.md`.

## Estado

**Sprint 01 — Foundation / baseline documental.** `main` publica actualmente la base documental aprobada del proyecto. La implementación técnica de Sprint 01 se desarrolla y valida de forma separada antes de su integración. La presencia de diseños, requisitos o roadmap en este repositorio no implica que esas capacidades estén implementadas, soportadas o validadas.

TATACOA es un proyecto en desarrollo activo, dirigido y revisado por personas. Herramientas basadas en inteligencia artificial pueden utilizarse como apoyo para investigación, documentación, pruebas y desarrollo; no sustituyen la Validación Humana ni convierten resultados en evidencia validada.

Para bugs ordinarios pueden utilizarse GitHub Issues cuando estén habilitados. Las vulnerabilidades de seguridad no deben publicarse en Issues; deben seguir el canal privado indicado en `SECURITY.md`.

## Licencia

TATACOA está disponible bajo la **PolyForm Noncommercial License 1.0.0**. El texto jurídicamente aplicable se encuentra en `LICENSE` y el aviso de autoría en `NOTICE`.

Es una licencia source-available para usos no comerciales. El acceso público al repositorio no concede derechos adicionales a los establecidos por la licencia. Cualquier permiso comercial, si se concede, requiere una licencia independiente otorgada por el licenciante.

> **Nota sobre idioma:** `LICENSE` conserva el texto oficial de PolyForm Noncommercial License 1.0.0 en inglés. Esta explicación en español facilita la comprensión y no sustituye, modifica ni traduce jurídicamente sus términos.

A LUGUISACA project.
