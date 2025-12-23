<#
.SYNOPSIS
    Installs fpm (File Package Manager) on Windows.

.DESCRIPTION
    Downloads the latest fpm release from GitHub and installs it to ~/.fpm/bin.
    Adds the install directory to the user's PATH if not already present.

.PARAMETER Version
    Specific version to install (e.g., "0.1.0"). Defaults to latest.

.EXAMPLE
    # Install latest version (run this one-liner in PowerShell):
    irm https://raw.githubusercontent.com/DragonAxeSoftware/fpm/main/install.ps1 | iex

.EXAMPLE
    # Install specific version:
    $env:FPM_VERSION = "0.1.0"; irm https://raw.githubusercontent.com/DragonAxeSoftware/fpm/main/install.ps1 | iex
#>

$ErrorActionPreference = "Stop"

$repo = "DragonAxeSoftware/fpm"
$installDir = Join-Path $env:USERPROFILE ".fpm\bin"
$version = $env:FPM_VERSION

Write-Host ""
Write-Host "Installing fpm..." -ForegroundColor Cyan
Write-Host ""

# Get latest version if not specified
if (-not $version) {
    Write-Host "Fetching latest version..."
    $releaseInfo = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest"
    $version = $releaseInfo.tag_name -replace '^v', ''
    Write-Host "Latest version: v$version"
} else {
    Write-Host "Installing version: v$version"
}

# Determine download URL
$zipName = "fpm-x86_64-pc-windows-msvc.zip"
$downloadUrl = "https://github.com/$repo/releases/download/v$version/$zipName"

# Create install directory
if (-not (Test-Path $installDir)) {
    Write-Host "Creating install directory: $installDir"
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

# Download
$tempZip = Join-Path $env:TEMP "fpm-$version.zip"
Write-Host "Downloading from: $downloadUrl"
try {
    Invoke-WebRequest -Uri $downloadUrl -OutFile $tempZip -UseBasicParsing
} catch {
    Write-Error "Failed to download fpm v$version. Check that the version exists at: https://github.com/$repo/releases"
    exit 1
}

# Extract
Write-Host "Extracting to: $installDir"
Expand-Archive -Path $tempZip -DestinationPath $installDir -Force

# Cleanup
Remove-Item $tempZip -Force

# Verify binary exists
$exePath = Join-Path $installDir "fpm.exe"
if (-not (Test-Path $exePath)) {
    Write-Error "Installation failed: fpm.exe not found after extraction"
    exit 1
}

# Add to PATH if not already there
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    Write-Host "Adding to PATH: $installDir"
    $newPath = "$installDir;$userPath"
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    $env:Path = "$installDir;$env:Path"
    Write-Host "  PATH updated (restart terminal for full effect)" -ForegroundColor Yellow
} else {
    Write-Host "Already in PATH: $installDir"
}

# Verify installation
Write-Host ""
Write-Host "Verifying installation..."
$versionOutput = & $exePath --version 2>&1
Write-Host "  $versionOutput" -ForegroundColor Green

Write-Host ""
Write-Host "fpm installed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "Usage:"
Write-Host "  fpm --help"
Write-Host "  fpm install"
Write-Host "  fpm status"
Write-Host ""
Write-Host "Note: Restart your terminal if 'fpm' command is not found."
