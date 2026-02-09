#Requires -Version 5.1
<#
.SYNOPSIS
    c9watch installer for Windows 11

.DESCRIPTION
    Downloads and installs the latest c9watch release for Windows.
    Supports both x64 and ARM64 architectures.

.EXAMPLE
    # Run directly from PowerShell:
    irm https://raw.githubusercontent.com/minchenlee/c9watch/main/install.ps1 | iex

    # Or download and run:
    .\install.ps1
#>

$ErrorActionPreference = 'Stop'

$REPO = "minchenlee/c9watch"
$APP_NAME = "c9watch"

function Write-Info { param([string]$Message) Write-Host "=> $Message" -ForegroundColor Cyan }
function Write-Err  { param([string]$Message) Write-Host "Error: $Message" -ForegroundColor Red; exit 1 }

# --- Check OS ---
if ($env:OS -ne 'Windows_NT') {
    Write-Err "This installer is for Windows only. Detected OS: $($env:OS)"
}

$osVersion = [System.Environment]::OSVersion.Version
if ($osVersion.Build -lt 22000) {
    Write-Host "Warning: c9watch is optimized for Windows 11 (build 22000+). Current build: $($osVersion.Build)" -ForegroundColor Yellow
}

# --- Detect architecture ---
$arch = $env:PROCESSOR_ARCHITECTURE
switch ($arch) {
    'AMD64'   { $archLabel = 'x64' }
    'ARM64'   { $archLabel = 'arm64' }
    default   { Write-Err "Unsupported architecture: $arch" }
}

Write-Info "Detected Windows ($archLabel)"

# --- Find latest release ---
Write-Info "Fetching latest release..."

try {
    $releaseInfo = Invoke-RestMethod -Uri "https://api.github.com/repos/$REPO/releases/latest" -UseBasicParsing
    $latestTag = $releaseInfo.tag_name
} catch {
    Write-Err "Could not determine the latest release. Check https://github.com/$REPO/releases"
}

if (-not $latestTag) {
    Write-Err "Could not determine the latest release tag."
}

Write-Info "Latest version: $latestTag"

# --- Look for NSIS installer first, then MSI ---
$nsisPattern = "${APP_NAME}_${latestTag}_${archLabel}-setup.exe"
$msiPattern  = "${APP_NAME}_${latestTag}_${archLabel}_en-US.msi"

$downloadAsset = $null
$installerType = $null

foreach ($asset in $releaseInfo.assets) {
    if ($asset.name -eq $nsisPattern) {
        $downloadAsset = $asset
        $installerType = 'nsis'
        break
    }
    if ($asset.name -eq $msiPattern) {
        $downloadAsset = $asset
        $installerType = 'msi'
    }
}

if (-not $downloadAsset) {
    Write-Err "No installer found for architecture $archLabel in release $latestTag.`nLooked for: $nsisPattern or $msiPattern"
}

$downloadUrl = $downloadAsset.browser_download_url
$fileName = $downloadAsset.name

Write-Info "Downloading $fileName..."

# --- Download ---
$tempDir = Join-Path $env:TEMP "c9watch-install-$(Get-Random)"
New-Item -ItemType Directory -Path $tempDir -Force | Out-Null
$installerPath = Join-Path $tempDir $fileName

try {
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $downloadUrl -OutFile $installerPath -UseBasicParsing
    $ProgressPreference = 'Continue'
} catch {
    Write-Err "Failed to download installer: $_"
}

Write-Info "Downloaded to $installerPath"

# --- Install ---
Write-Info "Installing $APP_NAME..."

try {
    if ($installerType -eq 'nsis') {
        # NSIS installer - run silently
        $process = Start-Process -FilePath $installerPath -ArgumentList '/S' -Wait -PassThru
        if ($process.ExitCode -ne 0) {
            Write-Err "NSIS installer exited with code $($process.ExitCode)"
        }
    } else {
        # MSI installer - run silently
        $process = Start-Process -FilePath 'msiexec.exe' -ArgumentList "/i `"$installerPath`" /qn" -Wait -PassThru
        if ($process.ExitCode -ne 0) {
            Write-Err "MSI installer exited with code $($process.ExitCode)"
        }
    }
} catch {
    Write-Err "Installation failed: $_"
}

# --- Cleanup ---
Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue

# --- Done ---
Write-Host ""
Write-Info "$APP_NAME has been installed successfully!"
Write-Info "You can launch it from the Start Menu or by searching for '$APP_NAME'."
Write-Host ""
