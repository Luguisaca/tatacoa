# Guía de QA humano — TATACOA Alpha

## Propósito y alcance

Esta guía permite validar TATACOA como usuario final, sin asumir conocimiento interno del código. Cubre el recorrido `ENGAGEMENT → SCOPE → ENVIRONMENT → TARGET → SESSION → EXECUTION`, export Plain y Encrypted v1, verificación offline, Security Profiles y pruebas negativas controladas.

Esta guía conserva el procedimiento reproducible utilizado para el QA técnico/humano previo de la Alpha. Los resultados obtenidos siguen siendo evidencia válida de los casos ejecutados, pero la validación integral como producto queda pendiente hasta disponer del flujo real de usuario de Sprint 03 / Usable Alpha. Futuras ejecuciones sirven para regresión y no amplían por sí solas las garantías existentes. Registre el entorno, los resultados y cualquier diferencia observada. Use datos ficticios y herramientas autorizadas; nunca incorpore secretos o información real de clientes.

## Preparación

Desde la raíz de una copia limpia del repositorio y sobre la rama aprobada para QA:

```powershell
git status --short --branch
cargo build --locked --workspace
cargo test --locked --workspace
```

Resultado esperado:

- la rama corresponde al candidato que se desea evaluar;
- el working tree está limpio antes de comenzar;
- build y tests terminan con código `0`;
- existen `target\debug\tatacoa.exe` y `target\debug\tatacoa-verify.exe` en Windows.

Defina rutas de prueba fuera de datos reales:

```powershell
$Tatacoa = Resolve-Path '.\target\debug\tatacoa.exe'
$Verifier = Resolve-Path '.\target\debug\tatacoa-verify.exe'
$QaRoot = Join-Path $env:TEMP 'tatacoa-alpha-user-qa'
$Workspace = Join-Path $QaRoot 'workspace-lab'
$PlainBundle = Join-Path $QaRoot 'bundle-plain'
$EncryptedBundle = Join-Path $QaRoot 'bundle-encrypted.tatacoa'
New-Item -ItemType Directory -Path $QaRoot -ErrorAction Stop | Out-Null
```

Si `$QaRoot` ya existe, elija otro nombre. No reutilice ni borre una ruta cuyo contenido no haya inspeccionado.

## Caso 1 — recorrido completo LAB_LEARNING

### Crear el contexto

```powershell
$Engagement = & $Tatacoa engagement-create `
  --workspace $Workspace `
  --name 'QA Alpha Lab' `
  --security-profile lab-learning

$Scope = & $Tatacoa scope-create `
  --workspace $Workspace `
  --engagement $Engagement `
  --name 'QA local autorizada' `
  --authorization-boundary 'Solo este host y datos ficticios'

$Environment = & $Tatacoa environment-create `
  --workspace $Workspace `
  --engagement $Engagement `
  --scope $Scope `
  --name 'Windows local'

$Target = & $Tatacoa target-create `
  --workspace $Workspace `
  --engagement $Engagement `
  --scope $Scope `
  --environment $Environment `
  --label 'Rust compiler' `
  --locator 'localhost'

$Session = & $Tatacoa session-create `
  --workspace $Workspace `
  --engagement $Engagement `
  --scope $Scope `
  --environment $Environment `
  --target $Target `
  --name 'QA manual'
```

Cada comando debe finalizar con código `0` y devolver un ID. Conserve esos IDs en el registro de QA.

### Ejecutar y capturar

```powershell
$RunOutput = & $Tatacoa run `
  --workspace $Workspace `
  --engagement $Engagement `
  --session $Session `
  rustc --version

$RunOutput
$ExecutionLine = $RunOutput | Where-Object { $_ -like 'execution=*' }
$Execution = $ExecutionLine.Substring('execution='.Length)
```

Resultado esperado: aparecen `execution=<ID>` y `capture_status=Complete`. La ejecución no usa un shell implícito.

### Exportar y verificar Plain

LAB_LEARNING usa Plain por defecto:

```powershell
& $Tatacoa export `
  --workspace $Workspace `
  --engagement $Engagement `
  --execution $Execution `
  --bundle $PlainBundle

& $Tatacoa verify $PlainBundle
& $Verifier $PlainBundle
```

Resultado esperado: ambos verificadores terminan con código `0`, cada artifact aparece como `VALID` y el resultado final es `VERIFICATION: VALID`.

### Exportar y verificar Encrypted

```powershell
& $Tatacoa export `
  --workspace $Workspace `
  --engagement $Engagement `
  --execution $Execution `
  --bundle $EncryptedBundle `
  --encrypted

& $Tatacoa verify $EncryptedBundle
& $Verifier $EncryptedBundle
```

Durante export, introduzca una password ficticia de al menos 12 caracteres y máximo 1024 bytes UTF-8, y confírmela. Durante cada verificación introdúzcala una vez. La entrada no debe mostrarse en pantalla.

Resultado esperado: el bundle es un archivo, no un directorio; ambos verificadores terminan con código `0`; los artifacts aparecen como `VALID`. La password no debe aparecer en stdout, stderr, nombres de archivo ni mensajes de error.

## Caso 2 — Security Profiles

Repita la creación de contexto y una ejecución en workspaces separados. No copie IDs entre engagements.

| Perfil | Acción | Resultado esperado |
|---|---|---|
| `lab-learning` | export sin selector | Plain permitido por defecto |
| `lab-learning` | export con `--encrypted` | Encrypted permitido |
| `professional` | export sin selector | solicita password y crea Encrypted |
| `professional` | export con `--acknowledge-plain-export` | Plain permitido |
| `professional` | export Plain sin acknowledgement | no debe producir Plain; aplica default Encrypted |
| `high-sensitivity` | export sin selector | solicita password y crea Encrypted |
| `high-sensitivity` | export con `--acknowledge-plain-export` | rechazo, código distinto de `0`, sin bundle final |
| `custom` | cualquier modo | rechazo hasta aprobar política Custom, sin bundle final |

Además, confirme que Clap rechaza seleccionar simultáneamente ambos modos:

```powershell
& $Tatacoa export `
  --workspace $Workspace `
  --engagement $Engagement `
  --execution $Execution `
  --bundle (Join-Path $QaRoot 'invalid-selection') `
  --encrypted `
  --acknowledge-plain-export
```

Resultado esperado: error de argumentos, código distinto de `0` y ningún bundle creado.

## Caso 3 — pruebas negativas Encrypted

Conserve `$EncryptedBundle` intacto como control. Cada mutación se hace sobre una copia diferente.

### Password incorrecta

```powershell
& $Verifier $EncryptedBundle
$LASTEXITCODE
```

Introduzca una password incorrecta. Resultado esperado: `VERIFICATION ERROR`, código `2`, ningún artifact aceptado y ningún archivo plaintext generado.

### Modificación de un byte

```powershell
$TamperedBundle = Join-Path $QaRoot 'bundle-tampered.tatacoa'
Copy-Item -LiteralPath $EncryptedBundle -Destination $TamperedBundle -ErrorAction Stop
$Bytes = [System.IO.File]::ReadAllBytes($TamperedBundle)
$Bytes[$Bytes.Length - 1] = $Bytes[$Bytes.Length - 1] -bxor 1
[System.IO.File]::WriteAllBytes($TamperedBundle, $Bytes)
& $Verifier $TamperedBundle
$LASTEXITCODE
```

Resultado esperado: código `2`, fallo de autenticación y ningún plaintext aceptado.

### Truncamiento

```powershell
$TruncatedBundle = Join-Path $QaRoot 'bundle-truncated.tatacoa'
$Bytes = [System.IO.File]::ReadAllBytes($EncryptedBundle)
$Truncated = [byte[]]::new($Bytes.Length - 1)
[Array]::Copy($Bytes, $Truncated, $Truncated.Length)
[System.IO.File]::WriteAllBytes($TruncatedBundle, $Truncated)
& $Verifier $TruncatedBundle
$LASTEXITCODE
```

Resultado esperado: código `2` y rechazo cerrado.

### Datos añadidos

```powershell
$AppendedBundle = Join-Path $QaRoot 'bundle-appended.tatacoa'
$Bytes = [System.IO.File]::ReadAllBytes($EncryptedBundle)
$Appended = [byte[]]::new($Bytes.Length + 1)
[Array]::Copy($Bytes, $Appended, $Bytes.Length)
$Appended[$Appended.Length - 1] = 0
[System.IO.File]::WriteAllBytes($AppendedBundle, $Appended)
& $Verifier $AppendedBundle
$LASTEXITCODE
```

Resultado esperado: código `2`; el lector exige EOF exacto.

### Control posterior

```powershell
& $Verifier $EncryptedBundle
$LASTEXITCODE
```

Resultado esperado: el original continúa válido y devuelve código `0`.

## Caso 4 — pruebas negativas Plain

Copie `$PlainBundle` antes de alterarlo. Modifique un archivo bajo `objects` y ejecute ambos verificadores. Resultado esperado: artifact `INVALID`, SHA-256 mismatch y código distinto de `0`. Agregue además un objeto no declarado bajo `objects`; debe ser rechazado. No modifique el bundle de control.

## Caso 5 — aislamiento entre engagements

Cree dos engagements y sus contextos completos en el mismo workspace. Confirme:

- los IDs y directorios son independientes;
- una ejecución del engagement A no puede exportarse indicando el ID del engagement B;
- IDs de scope, environment, target o session de A no pueden utilizarse dentro de B;
- el fallo no crea manifest ni bundle final.

## Caso 6 — límites operativos

Ejecute una captura con límite pequeño:

```powershell
& $Tatacoa run `
  --workspace $Workspace `
  --engagement $Engagement `
  --session $Session `
  --max-stream-bytes 8 `
  rustc --version
```

Resultado esperado: el comando termina sin deadlock y reporta captura truncada cuando la salida supera el límite. El manifest debe conservar explícitamente ese estado; no debe presentarlo como captura completa.

## Códigos de salida

| Programa | Condición | Código esperado |
|---|---|---:|
| `tatacoa` | operación o verificación válida | `0` |
| `tatacoa` | fallo operativo, de política o verificación | `1` |
| `tatacoa` | argumentos CLI inválidos | `2` |
| `tatacoa-verify` | bundle válido | `0` |
| `tatacoa-verify` | bundle Plain procesado con artifacts inválidos | `1` |
| `tatacoa-verify` | error de lectura, formato o autenticación | `2` |

## Registro mínimo de QA

Copie esta tabla al reporte de la ejecución:

| Campo | Valor |
|---|---|
| Fecha/hora y zona | |
| Evaluador humano | |
| Commit exacto (`git rev-parse HEAD`) | |
| Rama | |
| Sistema operativo/versión | |
| Arquitectura | |
| `rustc --version` | |
| `cargo --version` | |
| Perfil probado | |
| Caso de prueba | |
| Resultado esperado | |
| Resultado observado | |
| PASS / FAIL / BLOCKED | |
| Código de salida | |
| Evidencia o captura | |
| Observaciones, sin secretos | |

Registre un renglón por caso y conserve los comandos exactos ejecutados. No pegue passwords, claves, datos sensibles ni bundles reales en el reporte.

## QA Linux e interoperabilidad

En Linux use `./target/debug/tatacoa` y `./target/debug/tatacoa-verify`; sustituya las variables de PowerShell por variables equivalentes del shell. Para la prueba de interoperabilidad:

1. genere un Plain y un Encrypted en Windows;
2. transfiera copias binarias sin modificarlas y verifíquelas en Linux;
3. genere ambos formatos en Linux y verifíquelos en Windows;
4. registre hashes SHA-256 del archivo Encrypted antes y después del transporte;
5. no automatice ni escriba la password en argumentos, scripts, variables persistentes o historial;
6. repita en WSL2, Kali y Parrot, identificando cada entorno por separado.

Un PASS en WSL2 no sustituye Kali o Parrot. Un fallo debe conservarse como hallazgo hasta determinar si corresponde al producto, al entorno o al procedimiento.

## Caso 7 — RFC 3161 explícito (SP3-09)

Use únicamente una TSA de laboratorio/autorizada. TATACOA no configura una TSA predeterminada y exportar no debe producir tráfico de red.

```powershell
$Sidecar = "$PlainBundle.tsr"
& $Tatacoa timestamp-request --bundle $PlainBundle --sidecar $Sidecar --mode plain --tsa 'https://TSA-AUTORIZADA/timestamp' --timeout-seconds 30
& $Tatacoa timestamp-verify --bundle $PlainBundle --sidecar $Sidecar --mode plain
& $Verifier $PlainBundle --timestamp-sidecar $Sidecar --timestamp-mode plain
```

Para evaluar `TRUSTED`, agregue tanto a solicitud/verificación como al verifier los parámetros explícitos `--tsa-trust-anchor-der C:\ruta\anchor.der --tsa-policy 1.2.3...`; repita `--tsa-intermediate-der` cuando la cadena lo necesite. No use certificados del trust store HTTPS por inferencia. Desktop ofrece los mismos campos como una ruta/OID por línea.

Resultados esperados:

- se crea un único `.tsr` DER solo después de que la respuesta coincida con objeto y nonce y cumpla el contrato `SIGNATURE_VALID`;
- una TSA compatible muestra `SIGNATURE_VALID`, con los checks CMS, ESSCertIDv2, certificado firmante, atributos y firma en `PASS`;
- sin política de trust explícita, EKU/path/trust permanecen `NOT_EVALUATED`; con anchor/policy correctos pueden llegar a `TRUSTED`, pero validación histórica permanece `INDETERMINATE`;
- la verificación offline no contacta la TSA;
- alterar el bundle, usar modo incorrecto, truncar/corromper el sidecar o exceder 4 MiB falla sin aceptar el timestamp;
- repetir la solicitud contra un sidecar existente no lo sobrescribe;
- una TSA ausente o fallida no modifica ni invalida el bundle exportado.

Para Encrypted use `--mode encrypted`; el imprint corresponde a los bytes exactos del archivo final y no requiere descifrarlo. No interprete `SIGNATURE_VALID` como TSA confiable ni como validación histórica.

## Cierre y manejo de resultados

- Preserve los bundles originales de control mientras dure la investigación.
- Trate workspace y bundles como información sensible aunque los datos sean ficticios.
- Elimine las copias de QA únicamente después de revisar la ruta exacta y cuando ya no sean necesarias.
- No interprete `VERIFICATION: VALID` como firma de autor, timestamp confiable, autorización de la prueba o Validación Humana de Evidence.
- Entregue el reporte y los hallazgos para revisión antes de declarar Alpha aprobada.
- El gate integral de Encrypted v1 debe repetirse/confirmarse dentro del flujo de usuario de la Usable Alpha cuando Sprint 03 lo haga posible; no basta con que el componente funcione de forma aislada.
