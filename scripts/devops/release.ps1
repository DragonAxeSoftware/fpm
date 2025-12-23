<#
.SYNOPSIS
    Creates a new release by incrementing version and pushing a git tag.

.DESCRIPTION
    This script:
    1. Reads the current version from Cargo.toml
    2. Increments the specified version component (major, minor, or patch)
    3. Updates Cargo.toml with the new version
    4. Commits the version bump
    5. Creates and pushes a git tag (e.g., v0.2.0)
    6. The tag push triggers the GitHub Actions release workflow

.PARAMETER Bump
    Which version component to increment: major, minor, or patch (default: patch)

.PARAMETER DryRun
    If specified, shows what would happen without making changes

.EXAMPLE
    .\release.ps1                  # Bump patch: 0.1.0 -> 0.1.1
    .\release.ps1 -Bump minor      # Bump minor: 0.1.0 -> 0.2.0
    .\release.ps1 -Bump major      # Bump major: 0.1.0 -> 1.0.0
    .\release.ps1 -DryRun          # Preview without changes
#>

param(
    [ValidateSet("major", "minor", "patch")]
    [string]$Bump = "patch",
    
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
    $major = [int]$Matches[1]
    $minor = [int]$Matches[2]
    $patch = [int]$Matches[3]
    $currentVersion = "$major.$minor.$patch"
} else {
    Write-Error "Could not parse version from Cargo.toml"
    exit 1
}

# Calculate new version
switch ($Bump) {
    "major" {
        $major++
        $minor = 0
        $patch = 0
    }
    "minor" {
        $minor++
        $patch = 0
    }
    "patch" {
        $patch++
    }
}
$newVersion = "$major.$minor.$patch"

Write-Host ""
Write-Host "Current version: " -NoNewline
Write-Host $currentVersion -ForegroundColor Yellow
Write-Host "New version:     " -NoNewline
Write-Host $newVersion -ForegroundColor Green
Write-Host ""

if ($DryRun) {
    Write-Host "[DRY RUN] Would perform the following:" -ForegroundColor Cyan
    Write-Host "  1. Update Cargo.toml version to $newVersion"
    Write-Host "  2. Commit: 'chore: bump version to $newVersion'"
    Write-Host "  3. Create tag: v$newVersion"
    Write-Host "  4. Push commit and tag to origin"
    Write-Host "  5. GitHub Actions release workflow would build binaries"
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

# Update Cargo.toml
Write-Host "Updating Cargo.toml..." -ForegroundColor Cyan
$newContent = $content -replace 'version\s*=\s*"\d+\.\d+\.\d+"', "version = `"$newVersion`""
Set-Content $cargoToml $newContent -NoNewline

# Commit the change
Write-Host "Committing version bump..." -ForegroundColor Cyan
git add $cargoToml
git commit -m "chore: bump version to $newVersion"

# Create tag
Write-Host "Creating tag v$newVersion..." -ForegroundColor Cyan
git tag "v$newVersion"

# Push commit and tag
Write-Host "Pushing to origin..." -ForegroundColor Cyan
git push
git push origin "v$newVersion"

Write-Host ""
Write-Host "Release v$newVersion initiated!" -ForegroundColor Green
Write-Host ""
Write-Host "The GitHub Actions release workflow is now building binaries."
Write-Host "Check progress at: https://github.com/DragonAxeSoftware/fpm/actions"
Write-Host "Releases will appear at: https://github.com/DragonAxeSoftware/fpm/releases"
