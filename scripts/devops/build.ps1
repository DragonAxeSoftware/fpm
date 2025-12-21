#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Builds the gitf2 project.

.DESCRIPTION
    This is the standard way to build the gitf2 project.
    Run this script instead of calling cargo directly to ensure consistent builds.

.EXAMPLE
    .\scripts\devops\build.ps1
    
.EXAMPLE
    .\scripts\devops\build.ps1 -Release
#>

param(
    [switch]$Release
)

$ErrorActionPreference = "Stop"

Push-Location $PSScriptRoot\..\..

try {
    Write-Host "Building gitf2..." -ForegroundColor Cyan
    
    if ($Release) {
        cargo build --release
    } else {
        cargo build
    }
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Build completed successfully." -ForegroundColor Green
        
        $targetDir = if ($Release) { "target\release" } else { "target\debug" }
        $binaryPath = Join-Path $PWD (Join-Path $targetDir "gitf2.exe")
        
        Write-Host ""
        Write-Host "Artifact location:" -ForegroundColor Cyan
        Write-Host "  $binaryPath" -ForegroundColor Yellow
    } else {
        Write-Host "Build failed." -ForegroundColor Red
        exit $LASTEXITCODE
    }
} finally {
    Pop-Location
}
