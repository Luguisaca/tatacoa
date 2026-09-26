# TATACOA — Identidad y dirección del producto

## Estado

**DOCUMENTO RECTOR — dirección de producto aprobada.**

Este documento describe el propósito estable de TATACOA. Los sprints implementan partes de esta dirección; no redefinen el producto desde cero.

## Identidad

T·A·T·A·C·O·A significa **Test · Analyze · Trace · Assure · Contextualize · Observe · Apply**.

En español: **Testear · Analizar · Trazar · Asegurar · Contextualizar · Observar · Aplicar**.

La expansión expresa acciones conectadas del producto, no fases rígidas ni necesariamente lineales:

- **Test / Testear:** realizar una acción técnica controlada dentro de un contexto autorizado.
- **Analyze / Analizar:** examinar e interpretar lo obtenido sin convertir automáticamente un resultado en conclusión o evidencia validada.
- **Trace / Trazar:** mantener procedencia y relaciones entre contexto, acciones, ejecuciones, artifacts, derivados y evidencia.
- **Assure / Asegurar:** proteger y permitir comprobar las propiedades que TATACOA puede sustentar, especialmente integridad, procedencia y estados de verificación; no implica garantizar verdad, autoría, suficiencia forense o validez legal.
- **Contextualize / Contextualizar:** relacionar cada acción y resultado con el trabajo, alcance, entorno, objetivo, sesión y propósito que le dan significado.
- **Observe / Observar:** examinar estados, resultados, comportamientos y evolución durante el trabajo y sus revisiones.
- **Apply / Aplicar:** reutilizar lo aprendido para continuar, validar, documentar, reproducir o realizar nuevas pruebas y retests autorizados.

Expresión humana del ciclo:

> **Haz el trabajo. Compréndelo. Conserva su trazabilidad. Protege su integridad. Mantén su contexto. Observa lo que ocurre. Aplica lo aprendido.**

> **Do the work. Understand it. Preserve its trace. Protect its integrity. Keep its context. Observe what happens. Apply what you learn.**

TATACOA es un producto de **LUGUISACA — luguisaca.com** para profesionales que realizan evaluaciones de seguridad autorizadas y necesitan preservar no solo resultados, sino también contexto, evidencia, procedencia, decisiones y conocimiento reproducible.

## Origen

TATACOA nació de una necesidad recurrente en pentesting: ejecutar herramientas no basta. Una auditoría o laboratorio produce contexto, comandos, stdout/stderr, archivos, notas, decisiones, interrupciones y retests que deben poder relacionarse y retomarse.

El Discovery descartó competir como scanner, gestor genérico de findings o simple grabador de terminal. El foco es un espacio de trabajo verificable para procedencia, integridad, transformación, reproducción y comprensión de evidencia de pruebas de seguridad.

## Misión

Ayudar a profesionales de seguridad a ejecutar, preservar, comprender, retomar y demostrar su trabajo autorizado con contexto, trazabilidad e integridad, reduciendo la pérdida de evidencia y conocimiento sin convertir la automatización o la IA en autoridad de validación.

## Visión

Construir un espacio de trabajo local-first para pentesting y evaluación de seguridad que acompañe el ciclo técnico completo —desde el contexto autorizado hasta evidencia, aprendizaje, replay y retest— y permita que el trabajo siga siendo verificable, portable y útil más allá de una sesión, una interfaz o un equipo.

TATACOA debe poder evolucionar por capacidades sin obligar a rediseñar sus fundamentos en cada sprint.

## Problema

Los flujos manuales pueden provocar:

- evidencia dispersa entre terminales, carpetas, capturas y notas;
- pérdida del contexto exacto de ejecución;
- mezcla accidental de targets, entornos o engagements;
- dificultad para distinguir RAW de evidencia derivada o redactada;
- revalidación costosa por ausencia de recetas reproducibles;
- pérdida del punto de trabajo al pausar, cerrar o sufrir fallos;
- dificultad para reconstruir qué se hizo al retomar un proyecto o realizar un retest;
- exposición accidental de secretos o datos sensibles;
- aprendizaje superficial sin registrar qué demuestra una técnica y sus límites;
- dependencia de una aplicación para “creer” evidencia que debería poder verificarse externamente.

## Usuarios

TATACOA se diseña para:

1. profesionales que realizan evaluaciones de seguridad autorizadas;
2. personas que estudian y practican en laboratorios propios o plataformas autorizadas;
3. revisores que necesitan comprobar integridad, procedencia y contexto sin confiar ciegamente en la aplicación.

## Principios de producto

- **Local-first y offline-capable:** el trabajo local no depende de SaaS, cuenta cloud ni conexión permanente.
- **Interfaces independientes:** CLI y Desktop son experiencias de primera clase sobre una autoridad de dominio común.
- **Continuidad:** el trabajo debe poder pausarse, persistirse de forma segura, retomarse y resumirse.
- **Portabilidad:** un trabajo o bundle compatible debe poder trasladarse y abrirse sin perder su contexto/procedencia verificable.
- **Alcance autorizado:** ninguna automatización elimina la responsabilidad de trabajar dentro del scope permitido.
- **Validación Humana:** herramientas e IA asisten; no promueven por sí solas resultados a evidencia validada.
- **Preservar originales:** RAW nunca se sustituye silenciosamente por derivados.
- **Procedencia explícita:** transformaciones, derivaciones y retests mantienen relaciones trazables.
- **Verificación portable/offline:** lo verificable no debe exigir confiar ciegamente en la aplicación.
- **Reproducibilidad:** conservar suficiente contexto para repetir y comparar pruebas cuando sea legítimo.
- **Minimización de datos y secure-by-design.**
- **Estándares como ingeniería, no marketing:** una referencia no implica certificación.
- **Aprender haciendo:** ejecución, evidencia y conocimiento deben poder relacionarse.

## Motores conceptuales

### Scope / Context
Evita que una ejecución exista sin contexto operativo identificable y autorizado.

### Evidence
Captura, identifica, preserva, deriva, exporta y verifica artefactos.

### Knowledge / Learning
Relaciona ejecución con objetivos, observaciones, límites, validación, contexto defensivo y fuentes.

### Replay / Retest
Conecta contexto, evidencia y conocimiento para repetir una prueba de forma deliberada y comparar resultados sin convertir automáticamente una técnica en explotación genérica.

### Continuity
Permite que un trabajo sobreviva a pausas, cierres y cambios de sesión; facilita recordar qué se hizo, qué quedó pendiente y desde dónde continuar.

## Criterio de éxito

TATACOA tiene éxito cuando una persona puede realizar y retomar una evaluación autorizada conservando contexto, evidencia y conocimiento; y cuando un receptor autorizado puede verificar lo verificable sin tener que “creerle” ciegamente a TATACOA.

La evolución técnica de esta dirección se mantiene en [ROADMAP.md](ROADMAP.md). La identidad visual aprobada y sus límites se mantienen en [VISUAL-IDENTITY.md](VISUAL-IDENTITY.md).
