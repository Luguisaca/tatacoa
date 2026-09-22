# TATACOA — Estado actual

## Fecha de corte

2026-09-22

## Rama incremental actual

`feat/sp3-12-offline-tool-assistance`, heredada linealmente de SP3-01…SP3-11. HUMAN-QA-05 permanece FAIL hasta revalidación de usuario.

Este documento es la fuente operativa para saber dónde está el proyecto y qué sigue. No sustituye PROJECT, PRD, ROADMAP, ARCHITECTURE ni DECISIONS.

## Baseline

- Sprint 01: implementado y validado dentro de su alcance documentado.
- Sprint 02 / Alpha Expansion: implementado y validado dentro de su alcance documentado.
- Encrypted v1 + hardening de password: integrado y con QA técnico/humano previo; **validación integral de usuario pendiente** hasta disponer de la Usable Alpha.
- Sprint 03 / Usable Alpha: en implementación incremental; App API, Desktop E2E, capacidades de producto, continuidad, hardening y RFC 3161 hasta `TRUSTED` están implementados con QA técnico. QA humano integral y el gate histórico siguen pendientes.

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

La allowlist de firma está cerrada. SP3-10 implementa `TRUSTED` mediante path PKIX, anchor TSA DER explícito, validez al `genTime`, EKU crítico/exclusivo y policy aceptada; CLI, verifier y Desktop reciben esa configuración de forma explícita. El contrato definitivo de revocación/validación histórica permanece en gate; no se afirma `HISTORICALLY_VALIDATED`.

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

## Hallazgo de QA humano de producto

El QA humano acumulativo de Desktop confirmó el flujo contexto autorizado → revisión explícita → ejecución sin shell → captura → artifacts → consulta de stdout/stderr. También detectó una desviación de producto: Knowledge/Replay y otras capacidades aparecen principalmente como formularios que exponen primitivas internas y obligan a reintroducir información ya registrada.

El hallazgo se clasifica como **FAIL de producto/UX para la Usable Alpha actual**, no como invalidación del Core ni de los PASS técnicos anteriores. `SPRINT-03-PLAN.md` contiene ahora el contrato operativo: Desktop debe acompañar el trabajo de la pentester, reutilizar automáticamente contexto/Execution/evidencia conocidos, reservar interacción humana para interpretación/validación/decisiones reales y mantener explícitos los gates de autorización/seguridad.

La corrección incremental del flujo principal está implementada en `feat/sp3-11-execution-workflow`: la vista de Execution reúne contexto, invocación, resultado, artifacts y objetos asociados; Knowledge reutiliza los hechos capturados y solicita interpretación humana; Replay/Retest reutiliza por defecto executable, argv, contexto y vínculo de provenance con la Execution de origen. Los cambios deliberados de invocación siguen siendo explícitos, Replay no se ejecuta y Evidence no cambia de estado automáticamente.

El QA técnico de esta corrección es PASS. **HUMAN-QA-05 continúa como FAIL de producto/UX pendiente de revalidación humana**; los PASS técnicos y humanos anteriores se conservan. Desktop muestra feedback persistente dentro de la Execution y permite localizar el Knowledge/Replay recién creado, pero la suficiencia del recorrido solo puede cerrarse con una nueva prueba de usuario.

SP3-12 añade asistencia genérica offline de lectura: hechos de Execution con origen de manifest, niveles funcionales `GENERIC/DOCUMENTED/ADAPTED` y contratos opcionales de documentación local/adapters. No hay proveedor o adapter de producción registrado ni probes automáticos; una herramienta desconocida sigue ejecutándose/capturándose genéricamente y la ayuda documental indica `UNAVAILABLE`. Desktop muestra primero los hechos y permite guardar una nota breve como Knowledge borrador sin recorrer todos sus campos estructurados. Esto está implementado para QA técnico, no aprobado como experiencia de usuario.

## Siguiente paso

Revalidar HUMAN-QA-05/SP3-12 sobre el flujo Execution → Artifact/Evidence → comprensión/contexto → Replay/Retest de `feat/sp3-12-offline-tool-assistance`. Export/Encrypted v1/RFC 3161 no avanzan hasta completar esa revalidación humana.

`HISTORICALLY_VALIDATED` permanece bloqueado por gate humano y la importación continuable conserva decisiones de materialización/formato pendientes. Ningún merge a `main` ocurre sin aprobación humana explícita.

## Cierre QA de clientes limpios — 2026-09-22

Se completó el baseline de preparación de Windows 10 Pro x64, Parrot Security 7.3 x86_64 y Kali 2026.3 x86_64. Los entornos quedan preservados como clientes no preparados para desarrollo: no se instalaron Rust/Cargo para hacer pasar el QA. WebView2 está presente en Windows y WebKitGTK 4.1 en los dos Linux.

Resultado: **baseline de entorno PASS**. La instalación/ejecución de TATACOA como cliente nuevo queda **NOT TESTED / BLOCKED** porque en el estado revisado de SP3-12 no existe una Release publicada con artefacto distribuible de usuario final. Este bloqueo es de distribución/QA y no invalida los PASS técnicos anteriores.

HUMAN-QA-05/SP3-12 sigue pendiente. Antes de retomarlo sobre los clientes limpios debe existir un candidato distribuible trazable al commit probado; no se debe sustituir ese requisito clonando/compilando el repositorio en las máquinas de cliente.

### Horizonte aprobado para análisis posterior a SP3

La Validación Humana pidió conservar dos líneas de producto sin incorporarlas silenciosamente a Sprint 03:

1. estudiar la extensión de los fundamentos de trazabilidad, provenance, preservación e integridad hacia digital forensics/incident response/investigación y otros trabajos profesionales reproducibles. Las garantías criptográficas existentes no deben presentarse como cadena de custodia ni suficiencia forense/legal. Antes de cualquier claim o implementación especializada se requiere investigación formal de adquisición, provenance, identidad del operador, fuentes de tiempo, almacenamiento, transferencias, cadena de custodia, estándares y aplicabilidad;
2. mantener la Alpha bajo PolyForm Noncommercial 1.0.0 y estudiar posteriormente ediciones/capacidades comerciales. La separación futura debe basarse también en qué garantías, soporte y validación puede sostener responsablemente cada edición; la Alpha debe seguir siendo útil y no convertirse en una demo artificialmente limitada.

Estas líneas son **HORIZON / RESEARCH**, no autorización para Codex de implementar funciones forenses, cambiar licencia, crear paywalls o redefinir Sprint 03.

## Siguiente paso actualizado

El siguiente trabajo de implementación debe partir del estado real de SP3-12 y resolver el gate que permita generar/probar un artefacto distribuible de la Usable Alpha en cliente limpio, preservando Desktop independiente del CLI y sin convertir toolchains de desarrollo en requisitos de usuario. Después se retoma HUMAN-QA-05/SP3-12 sobre el flujo Execution → Artifact/Evidence → comprensión/contexto → Replay/Retest.

Export/Encrypted v1/RFC 3161 permanecen detrás de la revalidación humana ya documentada. HISTORICALLY_VALIDATED continúa bloqueado por su gate y la importación continuable conserva sus decisiones pendientes. Ningún merge, release o publicación ocurre sin aprobación humana explícita.
