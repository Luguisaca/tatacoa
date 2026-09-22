# TATACOA — Estado actual

## Fecha de corte

2026-09-21

## Rama incremental actual

`feat/sp3-15-guided-export-timestamp`, heredada linealmente de SP3-01…SP3-14. HUMAN-QA-05 permanece FAIL hasta revalidación de usuario.

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

SP3-13 reorganiza Desktop para hacer visibles continuidad, actividad, contexto autorizado y resultado de la ejecución recién capturada. La información técnica de artifacts y hechos permanece accesible por detalles progresivos. La reanudación tiene revisión contextual persistente. QA técnico de este incremento es PASS; QA humano sigue pendiente.

SP3-14 prepara Replay/Retest con reutilización de invocación y límite registrado del Scope sin exigir campos avanzados en el camino normal. Las recetas pueden precargar una nueva ejecución, pero nunca la ejecutan; la revisión contextual y la confirmación siguen obligatorias. Hay comparación read-only de metadatos registrados, sin conclusión automática ni vínculo persistido de retest. QA técnico PASS; QA humano pendiente.

SP3-15 integra la entrega desde la Execution abierta: Desktop consulta la política efectiva de Core para ofrecer modos de exportación y elegir el predeterminado, muestra el reconocimiento Plain solo cuando corresponde y conserva el resultado sin tapar la captura. El timestamp queda como paso opcional, explícito y sin red para la verificación; sus opciones de confianza y checks detallados permanecen accesibles. No cambia el contrato criptográfico ni eleva `HISTORICALLY_VALIDATED`. QA técnico de frontera PASS; QA humano integral pendiente.

## Siguiente paso

Revalidar HUMAN-QA-05 y el recorrido integral SP3-13…SP3-15 como usuario en Windows: Workspace → contexto autorizado → ejecución → artifacts/ayuda → nota opcional → Replay/Retest → continuidad → exportación → timestamp/verificación. No declarar Usable Alpha PASS sin esa prueba humana. La restricción anterior de no avanzar Export/Encrypted/RFC 3161 antes de HUMAN-QA-05 fue sustituida por la autorización explícita del bloque de integración Desktop; los contratos de seguridad permanecen intactos.

`HISTORICALLY_VALIDATED` permanece bloqueado por gate humano y la importación continuable conserva decisiones de materialización/formato pendientes. Ningún merge a `main` ocurre sin aprobación humana explícita.

## Gate de distribución y clientes limpios — 2026-09-22

Se completó el baseline de preparación para QA de distribución sobre Windows 10 Pro x64, Parrot Security 7.3 x86_64 y Kali 2026.3 x86_64.

Resultado: **BASELINE DE ENTORNO PASS / INSTALACIÓN DE CLIENTE NUEVO NOT TESTED — BLOCKED POR ARTEFACTO DISTRIBUIBLE AUSENTE**.

Los clientes se preservan deliberadamente sin convertirlos en entornos de desarrollo. No se instalarán Rust/Cargo/Node/npm ni se clonará/compilará el repositorio como sustituto de un artefacto de usuario final.

WebView2 está presente en el cliente Windows y WebKitGTK 4.1 en los clientes Linux evaluados. Estas comprobaciones establecen preparación del entorno, no compatibilidad funcional de TATACOA.

Desde el baseline SP3-12 la implementación avanzó incrementalmente por SP3-13, SP3-14 y SP3-15, con QA técnico PASS en sus respectivas fronteras. Este avance no convierte HUMAN-QA-05 ni Sprint 03 en PASS humano.

El siguiente gate de distribución consiste en producir un candidato de Usable Alpha trazable al commit probado, con artefactos adecuados para los targets aprobados y SHA-256 registrado. La política de firma de release continúa siendo una decisión separada y no debe asumirse resuelta.

El candidato debe poder probarse como producto sin entorno de desarrollo: instalación o despliegue según el formato, primer arranque, recorrido funcional, cierre/reapertura y desinstalación cuando aplique.

SP3-16 añade empaquetado local reproducible de candidatos portables Windows x64 separados para Desktop y CLI, con commit de origen y SHA-256. Su existencia habilita el siguiente QA de despliegue en cliente limpio, pero no demuestra aún primer arranque o funcionalidad allí. No hay artefactos Linux nativos desde este host Windows ni política de firma de release aprobada.

La Validación Humana aprobó el siguiente contrato de cierre técnico: importación continuable con paquete original/provenance preservados y sin promoción automática de Evidence; `HISTORICALLY_VALIDATED` bloqueado para esta Alpha; Desktop Windows portable e instalable, Desktop Linux x64 `.deb` y AppImage, y CLI separado en ambas plataformas. Estas decisiones están aprobadas, pero su registro no constituye por sí solo implementación ni QA de los artefactos nuevos.

Después se ejecutará la revalidación humana integral del recorrido vigente:

Workspace → contexto autorizado → Execution → Artifact/Evidence → asistencia/comprensión → Knowledge → Replay/Retest → continuidad → exportación → timestamp/verificación → cierre → reapertura/continuación.

`HISTORICALLY_VALIDATED` continúa bloqueado por su gate humano y la importación continuable mantiene sus decisiones pendientes. Ningún PASS técnico sustituye la Validación Humana y ningún merge a `main`, release o publicación ocurre sin aprobación humana explícita.

### Horizonte posterior a Sprint 03

Se conservan como **HORIZON / RESEARCH**, no como alcance autorizado de implementación:

1. estudiar la extensión de los fundamentos de trazabilidad, provenance, preservación e integridad hacia digital forensics, incident response, investigación y otros trabajos profesionales reproducibles. Las garantías criptográficas actuales no equivalen por sí solas a cadena de custodia, suficiencia forense o admisibilidad legal. Antes de claims o capacidades especializadas se requiere investigación formal de adquisición, provenance, identidad del operador, fuentes de tiempo, almacenamiento, transferencias, cadena de custodia, estándares y aplicabilidad;

2. mantener la Alpha bajo PolyForm Noncommercial 1.0.0 y estudiar posteriormente la evolución comercial según capacidades, soporte, validación y garantías sostenibles. Esta línea no autoriza cambios de licencia, paywalls ni degradación artificial de la Alpha.

Estas líneas no modifican el cierre previsto de Sprint 03.
