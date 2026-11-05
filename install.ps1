[CmdletBinding()]
param(
    [string]$Version = $env:CHOOSE_BROWSER_VERSION,
    [string]$Repository = $env:CHOOSE_BROWSER_REPOSITORY,
    [string]$InstallDir = $env:CHOOSE_BROWSER_INSTALL_DIR,
    [string]$BinaryPath,
    [switch]$SkipRegister
)

$ErrorActionPreference = 'Stop'

function Install-ChooseBrowser {
    [CmdletBinding()]
    param(
        [string]$Version = $env:CHOOSE_BROWSER_VERSION,
        [string]$Repository = $env:CHOOSE_BROWSER_REPOSITORY,
        [string]$InstallDir = $env:CHOOSE_BROWSER_INSTALL_DIR,
        [string]$BinaryPath,
        [switch]$SkipRegister
    )

    if ($env:OS -ne 'Windows_NT') {
        throw 'This installer requires Windows.'
    }
    if (-not $Version) { $Version = 'latest' }
    if (-not $Repository) { $Repository = 'tarcisiomiranda/choose-browser' }
    if (-not $InstallDir) { $InstallDir = Join-Path $env:LOCALAPPDATA 'Programs\ChooseBrowser' }
    if ($Repository -notmatch '^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$') {
        throw 'Repository must be an owner/name pair.'
    }
    if ($Version -ne 'latest' -and $Version -notmatch '^v[0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.+-]+)?$') {
        throw 'Version must be latest or a version tag such as v0.1.0.'
    }

    $temporaryDirectory = $null
    try {
        if (-not $BinaryPath) {
            $architecture = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
            if ($architecture -ne 'X64') {
                throw "Unsupported Windows architecture: $architecture (x64 required)."
            }
            $asset = 'choose-browser-windows-amd64.exe'
            $baseUrl = if ($Version -eq 'latest') {
                "https://github.com/$Repository/releases/latest/download"
            } else {
                "https://github.com/$Repository/releases/download/$Version"
            }
            $temporaryDirectory = Join-Path ([IO.Path]::GetTempPath()) ('choose-browser-' + [Guid]::NewGuid().ToString('N'))
            New-Item -ItemType Directory -Path $temporaryDirectory | Out-Null
            $BinaryPath = Join-Path $temporaryDirectory $asset
            $checksumPath = Join-Path $temporaryDirectory 'checksums.txt'
            [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
            Write-Host "Downloading choose-browser for Windows x64 ($Version)..."
            Invoke-WebRequest -UseBasicParsing -Uri "$baseUrl/$asset" -OutFile $BinaryPath
            Invoke-WebRequest -UseBasicParsing -Uri "$baseUrl/checksums.txt" -OutFile $checksumPath
            $expected = @(
                Get-Content -LiteralPath $checksumPath | ForEach-Object {
                    if ($_ -match ('^([a-fA-F0-9]{64})\s+\*?' + [regex]::Escape($asset) + '$')) {
                        $Matches[1]
                    }
                }
            )
            if ($expected.Count -ne 1) {
                throw "Expected exactly one SHA-256 checksum for $asset."
            }
            if ((Get-FileHash -LiteralPath $BinaryPath -Algorithm SHA256).Hash -ne $expected[0]) {
                throw "Checksum mismatch for $asset."
            }
            Write-Host 'SHA-256 checksum verified.'
        }

        $source = (Resolve-Path -LiteralPath $BinaryPath).ProviderPath
        $InstallDir = [IO.Path]::GetFullPath($InstallDir)
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        $targetPath = Join-Path $InstallDir 'choose-browser.exe'
        if ($source -ne $targetPath) {
            Copy-Item -LiteralPath $source -Destination $targetPath -Force
        }
        & $targetPath --list | Out-Null
        if ($LASTEXITCODE -ne 0) {
            throw 'The installed binary did not start correctly.'
        }
        if (-not $SkipRegister -and $env:CHOOSE_BROWSER_SKIP_REGISTER -notmatch '^(1|true|yes|on)$') {
            & $targetPath --install | Out-Host
            if ($LASTEXITCODE -ne 0) { throw 'Browser registration failed.' }
        }
        Write-Host "Choose Browser installed: $targetPath"
        Write-Host ('Run: & "' + $targetPath + '" --list')
    } finally {
        if ($temporaryDirectory) {
            $resolvedTemporary = [IO.Path]::GetFullPath($temporaryDirectory)
            $temporaryRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
            if (-not $resolvedTemporary.StartsWith($temporaryRoot, [StringComparison]::OrdinalIgnoreCase)) {
                throw 'Refusing to remove a directory outside the temporary directory.'
            }
            Remove-Item -LiteralPath $resolvedTemporary -Recurse -Force
        }
    }
}

if ($MyInvocation.InvocationName -ne '.') {
    Install-ChooseBrowser @PSBoundParameters
}
