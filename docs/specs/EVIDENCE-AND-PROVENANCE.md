# Evidence Manifest y Provenance

## Estado

**Especificación conceptual aprobada; schema exacto pendiente de implementación/QA.**

## Principios

- JSON UTF-8 versionado y portable;
- serialización determinista donde afecte verificación;
- distinguir `unknown`, `not_present` y `not_applicable`;
- ID lógico separado del digest;
- paths relativos;
- invocation separa executable, argv y display representation;
- shell es explícito;
- SHA-256 es el digest baseline del Alpha.

## Artifact

Debe poder expresar como mínimo:

- logical ID;
- engagement/execution de origen;
- tipo/rol;
- path relativo;
- tamaño;
- digest y algoritmo;
- estado de captura;
- timestamps observacionales claramente etiquetados;
- herramienta/contexto cuando aplique;
- clasificación RAW/derived;
- metadata versionada.

## Provenance DAG

Los Artifact son nodos y las derivaciones son edges. Tipos iniciales:

`CAPTURE | IMPORT | COPY | CROP | REDACTION | ANNOTATION | CONVERSION | EXTRACTION | MERGE | OTHER`

Reglas:

- cada derivado obtiene nuevo ID y digest;
- el original permanece disponible según política;
- ciclos son inválidos;
- una transformación no hereda silenciosamente la identidad del original;
- la procedencia debe poder recorrerse desde derivado hasta sus fuentes.

## Evidencia

Un Artifact capturado no es automáticamente Evidence. El flujo de validación conserva esa separación.

## Verificación

El verifier debe poder comprobar offline al menos:

- versión/estructura soportada;
- paths seguros;
- presencia esperada;
- tamaño/digest;
- referencias existentes;
- DAG sin ciclos;
- inconsistencias de manifest;
- artifacts alterados o faltantes.

La verificación de hash demuestra integridad respecto al valor registrado; no demuestra autoría, trusted time ni cadena de custodia legal.

## Evolución

Las firmas digitales permanecen en el horizonte Post-V1.

RFC 3161 / trusted timestamping fue promovido a Sprint 03 como **diseño + primera implementación funcional**. Su incorporación debe mantener compatibilidad con bundles anteriores y conservar la separación entre integridad, autoría, trusted time y cadena de custodia.

Antes de codificar el sellado debe superarse un gate de Validación Humana basado en fuentes oficiales para definir:

- qué objeto(s) exactos se timestamp-ean;
- en qué momento exacto del ciclo se solicita el timestamp.

Después de ese gate, el contrato debe especificar representación persistida, verificación, política/TSA y comportamiento ante operación offline, fallo o indisponibilidad de TSA. La ausencia de trusted timestamp debe ser explícita y nunca convertir evidencia existente en algo distinto de lo que sus propiedades realmente demuestran.
