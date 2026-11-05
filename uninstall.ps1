[CmdletBinding()]
param([string]$InstallDir = $env:CHOOSE_BROWSER_INSTALL_DIR)

$ErrorActionPreference = 'Stop'
if (-not $InstallDir) { $InstallDir = Join-Path $env:LOCALAPPDATA 'Programs\ChooseBrowser' }
$InstallDir = [IO.Path]::GetFullPath($InstallDir)
$binary = Join-Path $InstallDir 'choose-browser.exe'
if (Test-Path -LiteralPath $binary) {
    & $binary --uninstall | Out-Host
    if ($LASTEXITCODE -ne 0) { throw 'Could not remove browser registration.' }
    Remove-Item -LiteralPath $binary -Force
} else {
    foreach ($key in @(
        'HKCU:\Software\Clients\StartMenuInternet\ChooseBrowser',
        'HKCU:\Software\Classes\ChooseBrowser.Url'
    )) {
        if (Test-Path -LiteralPath $key) {
            Remove-Item -LiteralPath $key -Recurse -Force
        }
    }
    $registeredApps = 'HKCU:\Software\RegisteredApplications'
    if (Get-ItemProperty -LiteralPath $registeredApps -Name 'Choose Browser' -ErrorAction SilentlyContinue) {
        Remove-ItemProperty -LiteralPath $registeredApps -Name 'Choose Browser'
    }
}
if ((Test-Path -LiteralPath $InstallDir) -and @(Get-ChildItem -LiteralPath $InstallDir -Force).Count -eq 0) {
    Remove-Item -LiteralPath $InstallDir
}
Write-Host 'Choose Browser removed. Your configuration has been kept.'
Write-Host 'Select your preferred browser in Settings > Apps > Default apps.'
