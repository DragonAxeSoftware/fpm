<#
.SYNOPSIS
    Recreates an existing release tag to re-trigger the release workflow.

.DESCRIPTION
    This script is useful when a release workflow fails and you need to retry it
    with the same version number. It will:
    
    1. Delete the tag from the remote (GitHub)
    2. Delete the tag locally
    3. Recreate the tag pointing to the current HEAD
    4. Push the new tag to trigger the release workflow again
    
    Note: This will NOT delete the GitHub Release if one was partially created.
    You may need to manually delete it from the GitHub Releases page first.

.PARAMETER Version
    The version tag to recreate (e.g., "v0.1.0"). If not specified, reads from Cargo.toml.

.PARAMETER DryRun
    If specified, shows what would happen without making changes.

.EXAMPLE
    .\recreate-release.ps1                    # Use version from Cargo.toml
    
.EXAMPLE
    .\recreate-release.ps1 -Version v0.1.0    # Use explicit version

.EXAMPLE
    .\recreate-release.ps1 -DryRun            # Preview without changes
#>

param(
    [string]$Version,
    
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"

# Get the repo root (where Cargo.toml is)
$repoRoot = git rev-parse --show-toplevel 2>$null
if (-not $repoRoot) {
    Write-Error "Not in a git repository"
    exit 1
}

# If version not provided, read from Cargo.toml
if (-not $Version) {
    $cargoToml = Join-Path $repoRoot "Cargo.toml"
    if (-not (Test-Path $cargoToml)) {
        Write-Error "Cargo.toml not found at $cargoToml"
        exit 1
    }
    
    $content = Get-Content $cargoToml -Raw
    if ($content -match 'version\s*=\s*"(\d+)\.(\d+)\.(\d+)"') {
        $Version = "v$($Matches[1]).$($Matches[2]).$($Matches[3])"
    } else {
        Write-Error "Could not parse version from Cargo.toml"
        exit 1
    }
}

# Validate version format
if ($Version -notmatch '^v\d+\.\d+\.\d+$') {
    Write-Error "Version must be in format 'vX.Y.Z' (e.g., v0.1.0)"
    exit 1
}

Write-Host ""
Write-Host "Recreating release tag: " -NoNewline
Write-Host $Version -ForegroundColor Yellow
Write-Host ""

if ($DryRun) {
    Write-Host "[DRY RUN] Would perform the following:" -ForegroundColor Cyan
    Write-Host "  1. Delete remote tag: git push origin --delete $Version"
    Write-Host "  2. Delete local tag:  git tag -d $Version"
    Write-Host "  3. Create new tag:    git tag $Version"
    Write-Host "  4. Push new tag:      git push origin $Version"
    Write-Host ""
    Write-Host "Note: If a GitHub Release exists, delete it manually first at:"
    Write-Host "  https://github.com/DragonAxeSoftware/fpm/releases"
    exit 0
}

# Step 1: Delete the tag from remote
Write-Host "Step 1: Deleting remote tag..." -ForegroundColor Cyan
try {
    git push origin --delete $Version 2>&1 | Out-Null
    Write-Host "  Remote tag deleted" -ForegroundColor Green
} catch {
    Write-Host "  Remote tag not found (may not exist)" -ForegroundColor Yellow
}

# Step 2: Delete the local tag
Write-Host "Step 2: Deleting local tag..." -ForegroundColor Cyan
try {
    git tag -d $Version 2>&1 | Out-Null
    Write-Host "  Local tag deleted" -ForegroundColor Green
} catch {
    Write-Host "  Local tag not found (may not exist)" -ForegroundColor Yellow
}

# Step 3: Create new tag at HEAD
Write-Host "Step 3: Creating new tag at HEAD..." -ForegroundColor Cyan
git tag $Version
Write-Host "  Tag created" -ForegroundColor Green

# Step 4: Push the new tag
Write-Host "Step 4: Pushing tag to origin..." -ForegroundColor Cyan
git push origin $Version
Write-Host "  Tag pushed" -ForegroundColor Green

Write-Host ""
Write-Host "Release $Version re-triggered!" -ForegroundColor Green
Write-Host ""
Write-Host "The GitHub Actions release workflow is now running."
Write-Host "Check progress at: https://github.com/DragonAxeSoftware/fpm/actions"
Write-Host ""
Write-Host "Note: If the old GitHub Release still exists, delete it manually at:"
Write-Host "  https://github.com/DragonAxeSoftware/fpm/releases"
