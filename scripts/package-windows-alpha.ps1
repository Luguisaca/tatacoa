param(
    [Parameter(Mandatory = $true)]
    [string] $OutputDirectory
)

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$head = (& git -C $root rev-parse --short=7 HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or -not $head) { throw 'No se pudo determinar HEAD.' }
if (& git -C $root status --porcelain) { throw 'El árbol debe estar limpio antes de empaquetar.' }
$release = Join-Path $root 'target/release'
$binaries = @{
    desktop = @('tatacoa-desktop.exe')
    cli = @('tatacoa.exe', 'tatacoa-verify.exe')
}
$output = [IO.Path]::GetFullPath($OutputDirectory)
if (Test-Path -LiteralPath $output) { throw "Destino ya existe: $output" }
New-Item -ItemType Directory -Path $output | Out-Null

foreach ($flavor in @('desktop', 'cli')) {
    $name = "tatacoa-0.1.0-alpha.1-$head-windows-x64-$flavor"
    $stage = Join-Path $output $name
    New-Item -ItemType Directory -Path $stage | Out-Null
    foreach ($binary in $binaries[$flavor]) {
        $source = Join-Path $release $binary
        if (-not (Test-Path -LiteralPath $source -PathType Leaf)) { throw "Falta build release: $source" }
        Copy-Item -LiteralPath $source -Destination $stage
    }
    Copy-Item -LiteralPath (Join-Path $root 'LICENSE') -Destination $stage
    Copy-Item -LiteralPath (Join-Path $root 'NOTICE') -Destination $stage
    @(
        "TATACOA 0.1.0-alpha.1 — candidato local de HUMAN QA"
        "Commit: $head"
        "Target: Windows x64 (MSVC)"
        "Modalidad: $flavor"
        'No es release publicada ni HUMAN QA PASS.'
        'Desktop requiere Microsoft Edge WebView2 Runtime en el cliente.'
        'Extraiga el ZIP en una carpeta nueva. No requiere Rust, Node ni CLI preinstalado para Desktop.'
        'El workspace local no está cifrado. Use únicamente datos de QA autorizados.'
    ) | Set-Content -LiteralPath (Join-Path $stage 'QA-README.txt') -Encoding utf8
    $zip = Join-Path $output "$name.zip"
    Compress-Archive -LiteralPath $stage -DestinationPath $zip -CompressionLevel Optimal
    $resolvedStage = (Resolve-Path -LiteralPath $stage).Path
    if (-not $resolvedStage.StartsWith($output + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'El directorio temporal queda fuera del destino previsto.'
    }
    Remove-Item -LiteralPath $stage -Recurse
    $hash = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
    "$hash  $name.zip" | Add-Content -LiteralPath (Join-Path $output 'SHA256SUMS.txt') -Encoding ascii
    Write-Output "$name.zip $hash"
}
