<#
.SYNOPSIS
    Creates a new release by reading version from Cargo.toml and pushing a git tag.

.DESCRIPTION
    This script:
    1. Reads the current version from Cargo.toml
    2. Creates and pushes a git tag (e.g., v0.2.0)
    3. The tag push triggers the GitHub Actions release workflow

    Update the version in Cargo.toml before running this script.

.PARAMETER DryRun
    If specified, shows what would happen without making changes

.EXAMPLE
    .\release.ps1                  # Create release from Cargo.toml version
    .\release.ps1 -DryRun          # Preview without changes
#>

param(
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"

# Get the repo root (where Cargo.toml is)
$repoRoot = git rev-parse --show-toplevel 2>$null
if (-not $repoRoot) {
    Write-Error "Not in a git repository"
    exit 1
}

$cargoToml = Join-Path $repoRoot "Cargo.toml"
if (-not (Test-Path $cargoToml)) {
    Write-Error "Cargo.toml not found at $cargoToml"
    exit 1
}

# Read current version
$content = Get-Content $cargoToml -Raw
if ($content -match 'version\s*=\s*"(\d+)\.(\d+)\.(\d+)"') {
    $version = "$($Matches[1]).$($Matches[2]).$($Matches[3])"
} else {
    Write-Error "Could not parse version from Cargo.toml"
    exit 1
}

Write-Host ""
Write-Host "Version from Cargo.toml: " -NoNewline
Write-Host $version -ForegroundColor Green
Write-Host ""

# Check if tag already exists
$existingTag = git tag -l "v$version" 2>$null
if ($existingTag) {
    Write-Error "Tag v$version already exists. Update the version in Cargo.toml first."
    exit 1
}

if ($DryRun) {
    Write-Host "[DRY RUN] Would perform the following:" -ForegroundColor Cyan
    Write-Host "  1. Create tag: v$version"
    Write-Host "  2. Push tag to origin"
    Write-Host "  3. GitHub Actions release workflow would build binaries"
    exit 0
}

# Check for uncommitted changes
$status = git status --porcelain
if ($status) {
    Write-Error "Working directory has uncommitted changes. Please commit or stash them first."
    exit 1
}

# Check we're on main branch
$branch = git branch --show-current
if ($branch -ne "main") {
    Write-Warning "Not on main branch (currently on '$branch'). Continue? (y/N)"
    $response = Read-Host
    if ($response -ne "y" -and $response -ne "Y") {
        Write-Host "Aborted."
        exit 0
    }
}

# Create tag
Write-Host "Creating tag v$version..." -ForegroundColor Cyan
git tag "v$version"

# Push tag
Write-Host "Pushing tag to origin..." -ForegroundColor Cyan
git push origin "v$version"

Write-Host ""
Write-Host "Release v$version initiated!" -ForegroundColor Green
Write-Host ""
Write-Host "The GitHub Actions release workflow is now building binaries."
Write-Host "Check progress at: https://github.com/DragonAxeSoftware/fpm/actions"
Write-Host "Releases will appear at: https://github.com/DragonAxeSoftware/fpm/releases"
