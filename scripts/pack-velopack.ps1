<#
.SYNOPSIS
  Empacota o kapture com Velopack: Setup.exe moderno (one-click, por usuário, sem admin),
  pacote .nupkg (completo e delta), Portable.zip e os índices releases/assets.

.DESCRIPTION
  Pré-requisito: target/release/kapture.exe já compilado (npx tauri build --no-bundle).
  Idempotente: roda quantas vezes quiser; a pasta de entrada e a de saída são recriadas.
  A versão do `vpk` (CLI) é SEMPRE igual à do crate `velopack` no Cargo.lock.

.EXAMPLE
  ./scripts/pack-velopack.ps1 -Version 0.0.1-pr.12
#>
param(
  # Versão SemVer do pacote (ex.: 1.2.3 ou 0.0.1-pr.12)
  [Parameter(Mandatory = $true)][string]$Version,
  # Executável já compilado
  [string]$Exe = 'target/release/kapture.exe',
  # Pasta de saída dos artefatos do Velopack
  [string]$OutDir = 'Releases',
  # MSI para comparar tamanhos (opcional; usa o primeiro encontrado se existir)
  [string]$Msi = ''
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

function Fail($msg) { Write-Error $msg; exit 1 }

# --- 1. Versão do crate velopack no Cargo.lock --------------------------------------------
$lock = Get-Content (Join-Path $root 'Cargo.lock') -Raw
$m = [regex]::Match($lock, 'name = "velopack"\s+version = "([^"]+)"')
if (-not $m.Success) { Fail 'Crate velopack não encontrado no Cargo.lock. Ele está em src-tauri/Cargo.toml?' }
$vpkVersion = $m.Groups[1].Value
Write-Host "Crate velopack no Cargo.lock: $vpkVersion"

# --- 2. Instala o vpk na MESMA versão (pula se já estiver certa) ---------------------------
$instalada = ''
$lista = (dotnet tool list -g 2>$null) | Where-Object { $_ -match '^\s*vpk\s' }
if ($lista) { $instalada = ($lista -split '\s+' | Where-Object { $_ })[1] }
if ($instalada -eq $vpkVersion) {
  Write-Host "vpk $vpkVersion já instalado: nada a fazer."
} elseif ($instalada) {
  dotnet tool update -g vpk --version $vpkVersion
  if ($LASTEXITCODE -ne 0) { Fail "Falha ao atualizar o vpk para $vpkVersion." }
} else {
  dotnet tool install -g vpk --version $vpkVersion
  if ($LASTEXITCODE -ne 0) { Fail "Falha ao instalar o vpk $vpkVersion (dotnet SDK instalado?)." }
}
# O global tool vive em ~/.dotnet/tools; garante que esteja no PATH deste processo
$env:PATH = "$env:PATH$([IO.Path]::PathSeparator)$(Join-Path $HOME '.dotnet/tools')"

# --- 3. Pasta de entrada limpa só com o executável -----------------------------------------
if (-not (Test-Path $Exe)) { Fail "Executável não encontrado: $Exe (rode: npx tauri build --no-bundle)." }
$packDir = Join-Path $root 'target/velopack-input'
if (Test-Path $packDir) { Remove-Item $packDir -Recurse -Force }
New-Item -ItemType Directory -Force $packDir | Out-Null
Copy-Item $Exe (Join-Path $packDir 'kapture.exe')

if (Test-Path $OutDir) { Remove-Item $OutDir -Recurse -Force }

# --- 4. vpk pack ---------------------------------------------------------------------------
# Setup one-click por usuário (%LOCALAPPDATA%\kapture), sem admin. Atalhos: área de trabalho e
# menu iniciar. Sem assinatura de código (o SmartScreen avisa).
vpk pack `
  --packId kapture `
  --packVersion $Version `
  --packDir $packDir `
  --mainExe kapture.exe `
  --runtime win-x64 `
  --icon src-tauri/icons/icon.ico `
  --packTitle kapture `
  --packAuthors klawdyo `
  --splashImage packaging/velopack/splash.png `
  --shortcuts 'Desktop,StartMenuRoot' `
  --outputDir $OutDir
if ($LASTEXITCODE -ne 0) { Fail "vpk pack falhou (código $LASTEXITCODE)." }

# --- 5. Confere os arquivos esperados ------------------------------------------------------
$setup    = Get-ChildItem $OutDir -Filter '*-win-Setup.exe'  -ErrorAction SilentlyContinue | Select-Object -First 1
$full     = Get-ChildItem $OutDir -Filter '*-full.nupkg'     -ErrorAction SilentlyContinue | Select-Object -First 1
$portable = Get-ChildItem $OutDir -Filter '*-win-Portable.zip' -ErrorAction SilentlyContinue | Select-Object -First 1
$faltando = @()
if (-not $setup)    { $faltando += '*-win-Setup.exe' }
if (-not $full)     { $faltando += '*-full.nupkg' }
if (-not $portable) { $faltando += '*-win-Portable.zip' }
foreach ($f in 'releases.win.json', 'assets.win.json') {
  if (-not (Test-Path (Join-Path $OutDir $f))) { $faltando += $f }
}
if ($faltando) { Fail ("Arquivos esperados ausentes em ${OutDir}: " + ($faltando -join ', ')) }

# --- 6. Tabela de tamanhos (console e resumo do job) ---------------------------------------
$linhas = @(
  @{ n = 'kapture.exe (sem instalador)'; f = Get-Item $Exe },
  @{ n = 'Setup.exe (Velopack)';         f = $setup },
  @{ n = '.nupkg completo';              f = $full },
  @{ n = 'Portable.zip';                 f = $portable }
)
$delta = Get-ChildItem $OutDir -Filter '*-delta.nupkg' -ErrorAction SilentlyContinue | Select-Object -First 1
if ($delta) { $linhas += @{ n = '.nupkg delta'; f = $delta } }
if (-not $Msi) {
  $achado = Get-ChildItem 'target/release/bundle/msi' -Filter '*.msi' -ErrorAction SilentlyContinue | Select-Object -First 1
  if ($achado) { $Msi = $achado.FullName }
}
if ($Msi -and (Test-Path $Msi)) { $linhas += @{ n = 'MSI (WiX, para comparar)'; f = Get-Item $Msi } }

$md = @('### Tamanhos dos instaladores', '', '| Arquivo | Nome | Tamanho |', '|---|---|---:|')
foreach ($l in $linhas) {
  $md += ('| {0} | `{1}` | {2:N2} MB |' -f $l.n, $l.f.Name, ($l.f.Length / 1MB))
}
if ($Msi -and (Test-Path $Msi)) {
  $razao = $setup.Length / (Get-Item $Msi).Length
  $md += ''
  $md += ('Setup.exe equivale a {0:P0} do tamanho do MSI.' -f $razao)
}
$md | ForEach-Object { Write-Host $_ }
if ($env:GITHUB_STEP_SUMMARY) { $md | Add-Content -Path $env:GITHUB_STEP_SUMMARY -Encoding utf8 }
