# Política de seguridad

T·A·T·A·C·O·A es software de seguridad en desarrollo. La existencia de esta política no implica certificación, garantía forense ni cumplimiento de un estándar específico.

## Reporte de vulnerabilidades

Mientras el repositorio sea privado, las vulnerabilidades deben comunicarse directamente a los mantenedores mediante un canal privado autorizado del proyecto.

No incluir secretos, datos personales innecesarios, información de clientes ni evidencia sensible en Issues, Pull Requests o discusiones.

Antes de hacer público el repositorio se deberá definir y probar un canal formal de divulgación responsable.

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
