<#
.SYNOPSIS
  Empacota o Cap em .msix com MakeAppx.exe (Windows SDK).

.EXAMPLE
  ./packaging/msix/build-msix.ps1 -Version 0.1.0 -Exe target/release/cap.exe -Out dist/Cap.msix

.NOTES
  O pacote gerado NÃO é assinado. Isso basta para enviar à Microsoft Store
  (ela assina no processo de certificação), mas não instala localmente sem
  assinatura. Para testes locais, assine com signtool e um certificado confiável.
#>
param(
  [Parameter(Mandatory)] [string] $Version,
  [Parameter(Mandatory)] [string] $Exe,
  [string] $Out = "dist/Cap.msix"
)
$ErrorActionPreference = "Stop"

$here = Split-Path -Parent $MyInvocation.MyCommand.Path
if (-not (Test-Path $Exe)) { throw "Executável não encontrado: $Exe" }

# MSIX exige versão com 4 partes (A.B.C.D)
$parts = ($Version -split '[-+]')[0].Split('.')
while ($parts.Count -lt 4) { $parts += '0' }
$msixVersion = ($parts[0..3]) -join '.'

$stage = Join-Path ([System.IO.Path]::GetTempPath()) ("cap-msix-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $stage | Out-Null
New-Item -ItemType Directory -Path (Join-Path $stage "Assets") | Out-Null

Copy-Item $Exe (Join-Path $stage "cap.exe")
Copy-Item (Join-Path $here "Assets/*") (Join-Path $stage "Assets")
(Get-Content (Join-Path $here "AppxManifest.xml") -Raw).Replace("@VERSION@", $msixVersion) |
  Set-Content (Join-Path $stage "AppxManifest.xml") -Encoding utf8

# Localiza o MakeAppx.exe mais recente do Windows SDK
$makeappx = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\makeappx.exe" -ErrorAction SilentlyContinue |
  Sort-Object FullName -Descending | Select-Object -First 1
if (-not $makeappx) { throw "MakeAppx.exe não encontrado. Instale o Windows SDK." }

$outDir = Split-Path -Parent $Out
if ($outDir) { New-Item -ItemType Directory -Force $outDir | Out-Null }
& $makeappx.FullName pack /d $stage /p $Out /o
if ($LASTEXITCODE -ne 0) { throw "MakeAppx falhou ($LASTEXITCODE)" }

Remove-Item $stage -Recurse -Force
Write-Warning "Pacote NÃO assinado: serve para envio à Microsoft Store, não para instalação local."
"MSIX gerado: $Out"
