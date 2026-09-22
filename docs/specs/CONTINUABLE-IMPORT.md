# Importación continuable — contrato Alpha

## Autoridad y alcance

D-035, D-040 y D-050 gobiernan esta operación. `tatacoa-core` valida y materializa; App API, CLI y Desktop solo exponen la operación. Un bundle recibido es entrada no confiable. El booleano de revalidación representa una confirmación humana explícita del alcance actual; importar nunca autoriza ejecutar una receta o herramienta.

## Plain v2 implementado en SP3-20

La primera ruta admite un bundle directorio Plain `tatacoa.alpha.v2` con un solo manifest de Execution y contexto completo. Antes de crear un trabajo visible, Core exige inventario exacto, rutas y tipos regulares, ausencia de symlinks/reparse points, estructura/relaciones válidas y coincidencia de tamaño y SHA-256 de cada artifact. Copia a staging, vuelve a verificar la copia retenida y materializa los registros históricos con los IDs, horas observadas, contexto y provenance recibidos. La copia original queda bajo `engagements/<id>/received/plain` y no es el archivo de trabajo mutable.

El trabajo importado queda `PAUSED` con una sesión histórica elegida. Reanudar exige revalidación explícita del Scope/entorno/autorización actuales; cada nueva Execution conserva su gate de revisión. Colisión del ID de Engagement, fallo de verificación, falta de autorización o materialización fallida rechazan la operación; el staging no aparece como trabajo.

Esta Alpha solo materializa artifacts cuyo estado recibido es `CAPTURED`; un bundle que reclama `CANDIDATE`, `REVIEWED` o `VALIDATED` no se convierte automáticamente en Evidence local. `HIGH_SENSITIVITY` y `CUSTOM` rechazan importación Plain. Esas restricciones son fail-closed y no reescriben el paquete recibido.

## Límites aún pendientes

Encrypted v1 requiere una ruta de materialización que autentique y verifique antes de persistir plaintext en el workspace; la mera verificación sin exposición de contenido no equivale a proyecto continuable. Esta ruta está pendiente del incremento siguiente. `tatacoa.alpha.v1` carece de contexto tipado completo y se mantiene verificable, pero no importable como trabajo continuable. Un sidecar RFC 3161 externo al bundle no se importa automáticamente; su verificación offline permanece separada. Un hash y un manifest no prueban autoría ni autorización histórica.
