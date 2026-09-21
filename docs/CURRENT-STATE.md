# TATACOA — Estado actual

## Fecha de corte

2026-09-20

## Rama incremental actual

`feat/sp3-09-timestamp-signature`, heredada linealmente de SP3-01…SP3-08.

Este documento es la fuente operativa para saber dónde está el proyecto y qué sigue. No sustituye PROJECT, PRD, ROADMAP, ARCHITECTURE ni DECISIONS.

## Baseline

- Sprint 01: implementado y validado dentro de su alcance documentado.
- Sprint 02 / Alpha Expansion: implementado y validado dentro de su alcance documentado.
- Encrypted v1 + hardening de password: integrado y con QA técnico/humano previo; **validación integral de usuario pendiente** hasta disponer de la Usable Alpha.
- Sprint 03 / Usable Alpha: en implementación incremental; App API, Desktop E2E, capacidades de producto, continuidad, hardening y RFC 3161 hasta `SIGNATURE_VALID` están implementados con QA técnico. QA humano integral, `TRUSTED` y el gate histórico siguen pendientes.

Los PASS técnicos existentes se conservan. Cambiar el estado documental de Encrypted v1 no invalida pruebas previas: reconoce que todavía falta probarlo dentro de una experiencia real de producto.

## Dirección aprobada para Sprint 03

Sprint 03 convierte el baseline en una aplicación utilizable para trabajo y QA humano real.

Arquitectura aprobada:

`tatacoa-cli → tatacoa-core`

`tatacoa-desktop (Tauri 2) → tatacoa-app-api → tatacoa-core`

El diagrama es conceptual: el CLI permanece independiente de Desktop/Tauri y no debe duplicarse lógica de dominio.

Capacidades aprobadas:

- `tatacoa-app-api` como capa común de operaciones de usuario;
- `tatacoa-desktop` con Tauri 2;
- CLI-only y Desktop como modalidades de primera clase;
- flujo real de pentesting sobre contexto autorizado, executions, artifacts/evidence, knowledge, replay y retest;
- guardado seguro y continuidad;
- pausa y reanudación;
- recuperación tras cierre voluntario, cierre accidental o fallo;
- resumen bajo demanda al abrir/reanudar/retest;
- importación/apertura autorizada de paquetes compatibles como proyectos continuables;
- conservación de provenance original;
- integración de las capacidades ya construidas para poder validarlas como producto;
- spike Tauri de extremo a extremo;
- Windows + Linux según targets del proyecto;
- RFC 3161 como diseño + primera implementación funcional dentro de SP3.

## Flujo mínimo de producto a demostrar

`abrir/crear workspace → crear/abrir trabajo → definir contexto autorizado → registrar/visualizar ejecución → consultar artifact/evidence → guardar/pausar → cerrar → abrir TATACOA → autenticar/autorizar cuando corresponda → recuperar → pedir resumen → revalidar contexto actual → continuar`

Un cierre accidental debe recuperar desde el último estado seguro disponible sin convertir estado incompleto en evidencia validada.

Un paquete TATACOA compatible recibido de otra persona debe poder validarse y, si la persona está autorizada, abrirse como proyecto continuable. Continuar el trabajo no reescribe quién/qué originó acciones, artifacts o evidencia previos.

## RFC 3161

Objeto, momento, stack RustCrypto, transporte síncrono, TLS `rustls` y provider `ring` fueron aprobados. SP3-07 implementa solicitud, sidecar y verificación offline; SP3-08 los expone al usuario sin TSA predeterminada ni red silenciosa; SP3-09 valida el contrato CMS/RFC 3161 y eleva hasta `SIGNATURE_VALID`.

La allowlist de firma está cerrada. `TRUSTED` debe implementar path PKIX, anchor TSA explícito, validez al `genTime`, EKU y policy aceptada. El contrato definitivo de revocación/validación histórica permanece en gate; no se afirma `HISTORICALLY_VALIDATED`.

## Regla de ejecución para agentes

Una vez iniciada la implementación de SP3, el agente debe avanzar dentro del alcance aprobado y de las decisiones congeladas. No debe detenerse por decisiones menores de implementación ya cubiertas por arquitectura/documentación.

Debe detenerse ante:

- bloqueo real;
- contradicción documental que afecte arquitectura o seguridad;
- decisión humana, criptográfica o de seguridad no resuelta;
- operación destructiva/irreversible;
- merge, release, publicación o cambio de visibilidad/licencia;
- gate RFC 3161 indicado arriba.

## POST-V1 / horizonte que NO entra automáticamente en SP3

- reporting avanzado;
- colaboración;
- public-key recipients;
- signatures;
- encrypted workspace;
- hardware keys / PKCS#11 / TPM / KMS;
- plugin sandboxing;
- AI local.

RFC 3161 fue promovido desde este horizonte a SP3.

## Siguiente paso

Completar `TRUSTED` con configuración explícita y QA humano del flujo RFC 3161. Preparar infraestructura histórica sin elevar resultados sobre `INDETERMINATE`. La importación continuable conserva decisiones de materialización/formato todavía pendientes. Ningún merge a `main` ocurre sin aprobación humana explícita.
