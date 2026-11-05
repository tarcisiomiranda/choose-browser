[CmdletBinding()]
param([string]$BinaryPath)

$ErrorActionPreference = 'Stop'
if (-not $BinaryPath) { $BinaryPath = Join-Path $PSScriptRoot '..\target\debug\choose-browser.exe' }
$fixtureBinary = (Resolve-Path -LiteralPath $BinaryPath).ProviderPath
. (Join-Path $PSScriptRoot '..\install.ps1')
$fixtureChecksum = (Get-FileHash -LiteralPath $fixtureBinary -Algorithm SHA256).Hash.ToLowerInvariant()
$testRoot = Join-Path ([IO.Path]::GetTempPath()) ('choose-browser-installer-test-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $testRoot | Out-Null
$script:checksumMode = 'valid'
$script:downloads = @()

function Invoke-WebRequest {
    param([switch]$UseBasicParsing, [string]$Uri, [string]$OutFile)
    $script:downloads += $Uri
    if ($Uri.EndsWith('/checksums.txt')) {
        $checksum = if ($script:checksumMode -eq 'invalid') { '0' * 64 } else { $fixtureChecksum }
        $lines = switch ($script:checksumMode) {
            'missing' { "$checksum  choose-browser-linux-amd64" }
            'duplicate' { @("$checksum  choose-browser-windows-amd64.exe", "$checksum  choose-browser-windows-amd64.exe") }
            default { "$checksum  choose-browser-windows-amd64.exe" }
        }
        Set-Content -LiteralPath $OutFile -Value $lines -Encoding Ascii
    } else {
        Copy-Item -LiteralPath $fixtureBinary -Destination $OutFile
    }
}

try {
    $installDir = Join-Path $testRoot 'path with spaces'
    Install-ChooseBrowser -InstallDir $installDir -Version v0.1.0 -Repository owner/choose-browser -SkipRegister
    $target = Join-Path $installDir 'choose-browser.exe'
    if ((Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant() -ne $fixtureChecksum) {
        throw 'The installed binary differs from the verified download.'
    }
    if ($script:downloads[0] -ne 'https://github.com/owner/choose-browser/releases/download/v0.1.0/choose-browser-windows-amd64.exe') {
        throw 'Pinned release URL was incorrect.'
    }
    Write-Host 'PASS: verified download, custom repository, pinned release, and a path with spaces.'

    foreach ($mode in @('invalid', 'missing', 'duplicate')) {
        $script:checksumMode = $mode
        $rejected = $false
        try {
            Install-ChooseBrowser -InstallDir (Join-Path $testRoot $mode) -SkipRegister
        } catch {
            if ($_.Exception.Message -notmatch '(?i)checksum') { throw }
            $rejected = $true
        }
        if (-not $rejected -or (Test-Path -LiteralPath (Join-Path $testRoot "$mode\choose-browser.exe"))) {
            throw "Unsafe download was installed: $mode"
        }
        Write-Host "PASS: rejected $mode checksum before installation."
    }

    $script:checksumMode = 'valid'
    $script:downloads = @()
    Install-ChooseBrowser -InstallDir $installDir -SkipRegister
    if ($script:downloads[0] -notmatch '/releases/latest/download/') {
        throw 'Latest release URL was incorrect.'
    }
    Install-ChooseBrowser -BinaryPath $fixtureBinary -InstallDir (Join-Path $testRoot 'local') -SkipRegister
    Write-Host 'PASS: latest release, reinstall, and local binary installation.'
} finally {
    $resolvedRoot = [IO.Path]::GetFullPath($testRoot)
    if (-not $resolvedRoot.StartsWith([IO.Path]::GetTempPath(), [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Test cleanup path escaped the temporary directory.'
    }
    Remove-Item -LiteralPath $resolvedRoot -Recurse -Force
}
