param(
    [Parameter(Mandatory = $true)] [string] $TauriCli,
    [Parameter(Mandatory = $true)] [string] $OutputDirectory
)

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$cli = (Resolve-Path -LiteralPath $TauriCli).Path
if (& git -C $root status --porcelain) { throw 'El árbol debe estar limpio antes de empaquetar.' }
$head = (& git -C $root rev-parse --short=7 HEAD).Trim()
if ($LASTEXITCODE -ne 0) { throw 'No se pudo determinar HEAD.' }
$out = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $out) { throw "Destino ya existe: $out" }

Push-Location (Join-Path $root 'crates/tatacoa-desktop')
try {
    & $cli build --bundles nsis --ci
    if ($LASTEXITCODE -ne 0) { throw "Tauri build falló: $LASTEXITCODE" }
} finally {
    Pop-Location
}

$source = Join-Path $root 'target/release/bundle/nsis/TATACOA_0.1.0-alpha.1_x64-setup.exe'
if (-not (Test-Path -LiteralPath $source -PathType Leaf)) { throw "Falta instalador: $source" }
New-Item -ItemType Directory -Path $out | Out-Null
$name = "tatacoa-0.1.0-alpha.1-$head-windows-x64-desktop-setup.exe"
$dest = Join-Path $out $name
Copy-Item -LiteralPath $source -Destination $dest
$hash = (Get-FileHash -LiteralPath $dest -Algorithm SHA256).Hash.ToLowerInvariant()
"$hash  $name" | Set-Content -LiteralPath (Join-Path $out 'SHA256SUMS.txt') -Encoding ascii
@(
    'TATACOA Usable Alpha — candidato local de HUMAN QA'
    "Commit: $head"
    'Instalador NSIS Windows x64 sin firma de release.'
    'Desktop no requiere CLI preinstalado.'
    'WebView2 Runtime debe estar disponible en el cliente.'
    'Instalación, primer arranque, cierre/reapertura y desinstalación requieren HUMAN QA en cliente limpio.'
) | Set-Content -LiteralPath (Join-Path $out 'QA-README.txt') -Encoding utf8
Write-Output "$name $hash"
