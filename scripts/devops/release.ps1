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

.PARAMETER PreRelease
    If specified, marks the release as a pre-release (e.g., beta, alpha, rc)
    This is useful for testing releases without affecting stable users.

.EXAMPLE
    .\release.ps1                      # Create stable release from Cargo.toml version
    .\release.ps1 -DryRun              # Preview without changes
    .\release.ps1 -PreRelease          # Create pre-release (useful for beta versions)
#>

param(
    [switch]$DryRun,
    [switch]$PreRelease
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

# Read current version (supports semantic versioning with pre-release identifiers)
$content = Get-Content $cargoToml -Raw
if ($content -match 'version\s*=\s*"([\d\.]+-[\w\.]+|\d+\.\d+\.\d+)"') {
    $version = $Matches[1]
} else {
    Write-Error "Could not parse version from Cargo.toml"
    exit 1
}

# Check if version has pre-release identifier
$hasPreReleaseId = $version -match '-'

Write-Host ""
Write-Host "Version from Cargo.toml: " -NoNewline
Write-Host $version -ForegroundColor Green
if ($hasPreReleaseId) {
    Write-Host "Pre-release identifier detected in version" -ForegroundColor Yellow
}
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
    if ($PreRelease -or $hasPreReleaseId) {
        Write-Host "  3. GitHub Actions would create a PRE-RELEASE" -ForegroundColor Yellow
    } else {
        Write-Host "  3. GitHub Actions would create a stable release"
    }
    Write-Host "  4. Build binaries for all platforms"
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
if ($PreRelease -or $hasPreReleaseId) {
    Write-Host "Pre-release v$version initiated!" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "This will be marked as a PRE-RELEASE on GitHub." -ForegroundColor Yellow
    Write-Host "Pre-releases are useful for testing without affecting stable users."
} else {
    Write-Host "Release v$version initiated!" -ForegroundColor Green
}
Write-Host ""
Write-Host "The GitHub Actions release workflow is now building binaries."
Write-Host "Check progress at: https://github.com/DragonAxeSoftware/fpm/actions"
Write-Host "Releases will appear at: https://github.com/DragonAxeSoftware/fpm/releases"
