[CmdletBinding()]
param(
    [string]$InstallDir = $env:CHOOSE_BROWSER_INSTALL_DIR,
    [switch]$SkipRegister
)

$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Push-Location $projectRoot
try {
    mise run build
    if ($LASTEXITCODE -ne 0) { throw 'Release build failed.' }
    & (Join-Path $projectRoot 'install.ps1') -BinaryPath (Join-Path $projectRoot 'target\release\choose-browser.exe') -InstallDir $InstallDir -SkipRegister:$SkipRegister
} finally {
    Pop-Location
}
