<#
.SYNOPSIS
    Runs integration tests for gitf2.

.DESCRIPTION
    Builds the gitf2 binary and then executes integration tests that require 
    external dependencies:
    - Git installed and in PATH
    - Network access to test repositories

    These tests are marked with #[ignore] and must be explicitly requested.

.EXAMPLE
    .\scripts\tests\run_integration_tests.ps1
#>

$ErrorActionPreference = "Stop"

Push-Location $PSScriptRoot\..\..

try {
    # Check if git is available
    $gitVersion = & git --version 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Git is not installed or not in PATH." -ForegroundColor Red
        Write-Host "Please install git or ensure it's correctly configured in your PATH environment variable." -ForegroundColor Yellow
        exit 1
    }

    Write-Host "Git found: $gitVersion" -ForegroundColor Green

    # Build the binary first
    Write-Host ""
    Write-Host "Building gitf2..." -ForegroundColor Cyan
    cargo build
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Build failed." -ForegroundColor Red
        exit 1
    }

    # Check if the binary exists
    $binaryPath = Join-Path $PWD "target\debug\gitf2.exe"
    if (-not (Test-Path $binaryPath)) {
        Write-Host "ERROR: gitf2 binary not found at $binaryPath" -ForegroundColor Red
        exit 1
    }

    Write-Host "gitf2 binary ready: $binaryPath" -ForegroundColor Green
    Write-Host ""
    Write-Host "Running integration tests..." -ForegroundColor Cyan

    cargo test --lib integration_tests -- --ignored
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "Integration tests completed successfully." -ForegroundColor Green
    } else {
        Write-Host ""
        Write-Host "Integration tests failed." -ForegroundColor Red
        exit $LASTEXITCODE
    }
} finally {
    Pop-Location
}
