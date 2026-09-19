# Política de seguridad

T·A·T·A·C·O·A es software de seguridad en desarrollo. La existencia de esta política no implica certificación, garantía forense ni cumplimiento formal de un estándar específico.

## Problema que aborda

Las pruebas de seguridad generan información crítica de forma fragmentada: salidas de herramientas, comandos, notas, capturas, archivos, objetivos y resultados pueden quedar distribuidos entre terminales y carpetas. En trabajo paralelo aumenta además el riesgo de mezclar evidencia entre entornos o targets. La información puede contener secretos o datos sensibles y, al momento de revalidar un hallazgo, reconstruir exactamente qué se ejecutó y qué produjo un resultado puede requerir trabajo manual considerable.

Esta necesidad no es únicamente operativa. NIST SP 800-115 recomienda conservar información sobre las actividades del evaluador y un registro paso a paso que permita disponer de un audit trail; para registros manuales identifica datos como fecha y hora, sistema de evaluación, objetivo, herramienta, comando y comentarios. OWASP WSTG recomienda que los hallazgos contengan información suficiente para comprenderlos, reproducirlos y remediarlos, incluyendo evidencia técnica y protegiendo la información sensible.

TATACOA busca reducir esta brecha preservando contexto, ejecución, artefactos, integridad y procedencia desde el momento de la prueba, sin sustituir la interpretación profesional ni convertir automáticamente la salida de una herramienta en un hallazgo validado.

## Aprendizaje y práctica

El mismo problema aparece al aprender seguridad: ejecutar una técnica no garantiza comprender por qué funcionó, qué demuestra realmente, cuáles son sus limitaciones ni cómo reproducirla posteriormente.

Por ello TATACOA debe servir tanto para evaluaciones profesionales autorizadas como para laboratorios propios, plataformas de práctica y formación. El flujo de aprendizaje debe relacionar la ejecución real con conocimiento verificable: qué se está probando, por qué se realiza, qué observar, qué demuestra y qué no demuestra el resultado, errores comunes, validación, contexto defensivo y referencias.

La reproducción también es parte del aprendizaje: una prueba validada puede convertirse, de forma deliberada y trazable, en una receta de replay, script o miniherramienta reutilizable para practicar, verificar una corrección o realizar un retest. La reutilización no elimina los requisitos de alcance y autorización.

El principio es: **aprender haciendo y poder demostrar lo aprendido**.

## Referencias y trazabilidad de ingeniería

TATACOA debe diseñarse, implementarse, documentarse y validarse tomando como referencia los estándares, marcos y lineamientos aplicables a cada componente del proyecto. Esto incluye, según corresponda, publicaciones NIST, guías OWASP, estándares ISO/IEC relacionados con seguridad y evidencia digital y lineamientos colombianos vigentes de seguridad y privacidad de la información.

La aplicabilidad de cada referencia debe evaluarse durante el desarrollo. Cuando una referencia origine o respalde un requisito del producto, el proyecto debe mantener trazabilidad entre:

`FUENTE → REQUISITO TATACOA → DECISIÓN DE DISEÑO → IMPLEMENTACIÓN → PRUEBA → EVIDENCIA DE QA`

Entre las referencias ya identificadas para investigación y aplicación se encuentran NIST SP 800-115 para pruebas y evaluaciones técnicas, NIST SSDF para desarrollo seguro, OWASP WSTG y guías aplicables, la familia ISO/IEC 27000 pertinente a seguridad y evidencia digital, y los lineamientos colombianos que resulten aplicables. La lista deberá evolucionar de forma controlada conforme avance la arquitectura.

La alineación técnica con una referencia no equivale por sí sola a certificación, conformidad integral ni aprobación de una entidad externa. TATACOA solo debe garantizar comportamientos y controles concretos cuando hayan sido implementados y demostrados mediante pruebas y QA reproducible.

## Reporte de vulnerabilidades

No publiques en un Issue, Pull Request o discusión detalles explotables de vulnerabilidades, credenciales, secretos, datos personales innecesarios, información de clientes ni evidencia sensible.

El canal preferido para vulnerabilidades es **Private Vulnerability Reporting de GitHub** cuando esté habilitado para el repositorio. Si ese mecanismo no está disponible para la persona reportante, inicia el contacto de forma privada mediante `contacto@luguisaca.com`.

Los bugs ordinarios que no impliquen una vulnerabilidad de seguridad pueden reportarse mediante GitHub Issues cuando estén habilitados o a `bugs@luguisaca.com`.

Incluye, cuando sea seguro compartirlo, el commit o versión afectada, condiciones de reproducción, impacto observado y una prueba mínima. Evita enviar secretos o datos de terceros que no sean necesarios para investigar el reporte.

## Qué reportar

Son especialmente relevantes fallos que puedan afectar:

- integridad o sustitución de evidencia;
- aislamiento entre engagements;
- traversal, symlinks o acceso fuera del workspace;
- ejecución de comandos o inyección;
- manejo de secretos;
- parsers de entradas hostiles;
- manifests, procedencia o verificación;
- redacción y derivación de artefactos;
- exportación o cifrado;
- reproducción/replay;
- dependencias y cadena de suministro.

## Tratamiento

Un reporte no se considera resuelto únicamente porque deje de reproducirse. La corrección debe incluir, cuando sea viable, una prueba de regresión y revisión de la causa raíz.

## Divulgación

No se promete actualmente un SLA público. La política de versiones soportadas, tiempos de respuesta, advisories y divulgación coordinada se definirá antes de una versión pública estable.

## Claims

No describir TATACOA como “FIPS validated”, “ISO certified”, “forensically certified”, “tamper-proof”, “unhackable”, “government approved” ni equivalente sin una validación formal que sustente exactamente esa afirmación.
