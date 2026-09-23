# TATACOA — Roadmap evolutivo

## Propósito

Este roadmap conserva la dirección acumulativa del producto. **Un sprint no reemplaza la arquitectura ni borra decisiones anteriores:** entrega una parte del destino descrito en [PROJECT.md](PROJECT.md).

Antes de iniciar un sprint, agentes y personas deben revisar este roadmap, [DECISIONS.md](DECISIONS.md), [ARCHITECTURE.md](ARCHITECTURE.md) y el estado real del repositorio. Las capacidades futuras se diseñan de forma compatible con las bases ya aprobadas siempre que sea técnicamente y securitariamente válido.

Los estados usados son:

- **VALIDATED:** implementado y con la validación indicada.
- **IMPLEMENTED / QA PENDING:** implementado, sin aprobación humana completa.
- **APPROVED / PLANNED:** decisión aprobada, aún no implementada.
- **HORIZON:** dirección deseada; requiere diseño/decisión antes de implementación.

## Sprint 01 — Foundation

**Estado: VALIDATED dentro del alcance documentado.**

Vertical fundamental:

`ENGAGEMENT → CONTEXT → EXECUTION → ARTIFACT → SHA-256 → MANIFEST → EXPORT → VERIFY`

Entregó la base Rust, Core, CLI, verifier independiente/offline, captura, artifacts, manifest, bundles Plain, aislamiento y controles negativos iniciales. El detalle y límites de QA permanecen en [ALPHA-IMPLEMENTATION.md](ALPHA-IMPLEMENTATION.md).

La portabilidad Windows + Linux forma parte del baseline. Windows 11 x64 y WSL2 fueron validados en el alcance documentado; Kali Linux y Parrot OS continúan como targets específicos. En 2026-09-22 se validó su baseline de entorno para QA de cliente limpio, pero todavía no la ejecución de un artefacto distribuible de TATACOA.

## Sprint 02 — Alpha Expansion

**Estado: VALIDATED dentro del alcance documentado.**

Extiende Foundation con:

- Scope, Environment, Target y Session tipados;
- Security Profiles y política de export;
- Knowledge Card manual/versionada;
- Replay Recipe versionada;
- provenance y relaciones mínimas;
- hardening de aislamiento, referencias, paths y manifest;
- ampliación de pruebas negativas.

El detalle de implementación está en [SPRINT-02-IMPLEMENTATION.md](SPRINT-02-IMPLEMENTATION.md).

### Encrypted Bundle v1

**Estado: IMPLEMENTED / QA PENDING para validación integral de usuario.**

`tatacoa.encrypted.v1` añade confidencialidad persistente bajo las decisiones criptográficas congeladas en [DECISIONS.md](DECISIONS.md). Su estado exacto está en [ENCRYPTED-V1-IMPLEMENTATION.md](ENCRYPTED-V1-IMPLEMENTATION.md).

La implementación base y el hardening de password por Security Profile están integrados y cuentan con pruebas automatizadas y QA técnico/humano previo. La Validación Humana decidió no considerar cerrado Encrypted v1 como experiencia de producto hasta poder probarlo dentro del flujo real de la Usable Alpha de Sprint 03. Esto no invalida los PASS técnicos ya obtenidos ni implica certificación, validación FIPS o garantías fuera de los casos efectivamente probados.

## Sprint 03 — Usable Alpha

**Estado: IMPLEMENTED / QA PENDING — alcance técnico implementado hasta SP3-22; candidatos distribuibles y aprobación humana integral pendientes de QA.**

Objetivo: pasar de componentes implementados a una Alpha utilizable para trabajo y QA humano real de una pentester.

Dirección aprobada:

- `tatacoa-core` continúa como autoridad de dominio, lógica, seguridad, evidencia, políticas y cifrado;
- crear `tatacoa-app-api` como capa común de operaciones de usuario;
- mantener `tatacoa-cli` plenamente funcional e independiente de la GUI;
- crear `tatacoa-desktop` con **Tauri 2**;
- Desktop y CLI son interfaces alternativas del mismo producto, no implementaciones divergentes;
- CLI puede instalarse sin Desktop/Tauri;
- Desktop puede instalarse sin una instalación previa del CLI;
- operación local-first y offline-capable para funciones locales;
- GUI orientada al flujo real de una pentester, no a demostrar métodos internos del Core;
- integrar workspace/trabajo → contexto autorizado → Scope/Environment/Targets/Session → executions → artifacts/evidence → knowledge/replay/retest;
- permitir pausa, persistencia segura, reanudación y recuperación tras cierre voluntario, accidental o fallo, con guardado seguro suficiente para no perder el punto de trabajo;
- al abrir, retomar o realizar retest, ofrecer bajo demanda un resumen de continuidad: qué se hizo, ejecuciones y evidencia relevantes, contexto conocido, pendientes y desde dónde continuar;
- permitir apertura/importación autorizada de paquetes compatibles como proyectos continuables, preservando integridad, políticas y provenance original;
- mantener interoperabilidad de datos soportados entre CLI y Desktop.

### Decisiones de SP3 que requieren diseño antes de implementación

RFC 3161 deja el horizonte POST-V1 y entra en **alcance aprobado de Sprint 03 como diseño + primera implementación funcional de trusted timestamping** por su relevancia para integridad temporal. Antes de codificar esa parte debe cerrarse mediante investigación oficial y Validación Humana qué objeto(s) se timestamp-ean y en qué momento del ciclo. La selección/configuración de TSA, representación, verificación y comportamiento ante ausencia de red/fallo deben diseñarse de forma explícita; un fallo de TSA nunca puede falsear, destruir ni degradar silenciosamente la evidencia existente.

La autenticación/protección necesaria para reabrir trabajos protegidos, la persistencia de estado de continuidad y los límites exactos de importación también deben concretarse sin debilitar las decisiones de seguridad existentes.

### Gate de distribución de la Usable Alpha

El QA de cliente limpio de 2026-09-22 confirmó entornos preparados en Windows 10 Pro x64, Kali 2026.3 y Parrot 7.3, pero quedó **NOT TESTED / BLOCKED** para instalación porque SP3-12 no dispone todavía de una Release publicada con artefacto de usuario final. No se instalarán toolchains ni se compilará el repositorio en esos clientes para sustituir el artefacto faltante. La distribución debe demostrar Desktop independiente del CLI y del entorno de desarrollo antes de cerrar la Usable Alpha.

## Horizonte de producto

Estas capacidades expresan hacia dónde puede evolucionar TATACOA. **No son compromisos de sprint ni capacidades implementadas.** Su existencia aquí evita diseñar hoy fundamentos incompatibles con el destino conocido.

- integridad temporal verificable / RFC 3161;
- autenticidad mediante firmas cuando se apruebe el modelo;
- protección/cifrado del workspace cuando el modelo de amenaza lo justifique;
- reporting técnico derivado de evidencia/provenance;
- colaboración controlada sin convertir cloud en dependencia del producto;
- recipients de clave pública y mecanismos de custodia compatibles con políticas futuras;
- hardware-backed keys/PKCS#11/TPM/KMS cuando exista caso de uso aprobado;
- automatización y asistencia local de IA bajo Validación Humana;
- adapters adicionales sin convertirlos en autoridad de evidencia;
- sandboxing/extensibilidad solo después de definir un modelo de seguridad apropiado.
- estudiar digital forensics/incident response/investigación como extensión posible de los fundamentos de provenance, preservación, integridad y reproducibilidad; requiere diseño formal de adquisición, identidad del operador, tiempo, almacenamiento, transferencias y cadena de custodia antes de claims especializados;
- estudiar una evolución comercial posterior a la Alpha no comercial, separando capacidades/ediciones por garantías, soporte y validación sostenibles sin degradar artificialmente la utilidad de la Alpha; cualquier cambio de licencia o términos requiere decisión humana explícita.

No asignar automáticamente estas capacidades a SP4/SP5/etc. La planificación de cada sprint se realiza contra necesidades reales, estado del producto y decisiones humanas, preservando este horizonte.

## QA acumulativo

Cada sprint hereda los invariantes y controles aplicables de los anteriores. Un cambio nuevo no puede declarar obsoleto un control de seguridad previo por conveniencia.

Continúan como objetivos de QA, según aplique:

- fresh clone y builds reproducibles;
- format/lint/check/test;
- Windows y Linux soportados;
- aislamiento y concurrencia;
- tamper/traversal/symlink;
- salida hostil/grande y fallos de captura;
- límites de secretos;
- provenance y derivaciones;
- verifier offline;
- zero telemetry;
- bundles portables;
- fallos explícitos;
- aislamiento de IA;
- continuidad/recuperación cuando sea implementada;
- documentación sincronizada con comportamiento real.

## Regla de evolución

Cuando llegue SP4 o cualquier sprint posterior, **no se comienza preguntando “qué producto hacemos ahora”**. Se revisan misión, visión, arquitectura, decisiones, capacidades entregadas, horizonte y resultados de QA; luego se selecciona la siguiente entrega compatible.

Si una necesidad futura exige cambiar una decisión congelada, el conflicto debe hacerse explícito, justificarse y aprobarse mediante Validación Humana/ADR antes de modificar la arquitectura.
