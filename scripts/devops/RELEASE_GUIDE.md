# Release Guide for fpm

This guide explains how to create releases for the fpm project.

## Release Types

### Stable Release
Stable releases are production-ready versions that all users will see and can install.

**When to use:**
- Features are fully tested and documented
- No known critical bugs
- Ready for general availability

**Version format:** `MAJOR.MINOR.PATCH` (e.g., `0.2.0`, `1.0.0`)

### Pre-Release (Beta/RC/Alpha)
Pre-releases allow you to test new features with early adopters without affecting stable users.

**When to use:**
- Testing new features before stable release
- Getting feedback from early adopters
- Validating changes in production-like environments
- Moving fast without breaking stable installations

**Version format:** `MAJOR.MINOR.PATCH-IDENTIFIER.NUMBER`
- Beta: `0.2.0-beta.1`, `0.2.0-beta.2`
- Release Candidate: `1.0.0-rc.1`, `1.0.0-rc.2`
- Alpha: `0.3.0-alpha.1`

**Benefits:**
- Users on stable versions won't auto-update to pre-releases
- Pre-releases are clearly marked on GitHub
- You can iterate quickly without risk
- Easy to test major changes before committing to a stable release

## How to Create a Release

### 1. Update Version in Cargo.toml

Edit `Cargo.toml` and update the version:

**For stable release:**
```toml
[package]
version = "0.2.0"
```

**For pre-release:**
```toml
[package]
version = "0.2.0-beta.1"
```

### 2. Commit the Version Change

```bash
git add Cargo.toml
git commit -m "Bump version to 0.2.0-beta.1"
git push
```

### 3. Run the Release Script

**For stable release:**
```powershell
.\scripts\devops\release.ps1
```

**For pre-release:**
```powershell
.\scripts\devops\release.ps1 -PreRelease
```

**To preview without creating a release:**
```powershell
.\scripts\devops\release.ps1 -DryRun
.\scripts\devops\release.ps1 -PreRelease -DryRun
```

### 4. Verify the Release

1. Check GitHub Actions: https://github.com/DragonAxeSoftware/fpm/actions
2. Once complete, verify the release: https://github.com/DragonAxeSoftware/fpm/releases
3. Pre-releases will be marked with a "Pre-release" badge

## Example Workflow

### Testing a New Feature

1. Develop the feature on a branch
2. Merge to main when ready for testing
3. Update version to `0.2.0-beta.1` in Cargo.toml
4. Run `.\scripts\devops\release.ps1 -PreRelease`
5. Test the pre-release build
6. If issues found, fix and create `0.2.0-beta.2`
7. When satisfied, update to `0.2.0` and create stable release

### Quick Iteration

Pre-releases let you:
- Ship features to early adopters quickly
- Get real-world feedback
- Fix issues without affecting stable users
- Build confidence before stable release

## Semantic Versioning

fpm follows [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)
- **Pre-release**: Additional identifier for testing versions

Examples:
- `0.1.0` → `0.2.0`: New features added
- `0.2.0` → `0.2.1`: Bug fixes
- `0.2.0` → `1.0.0`: Breaking changes
- `0.2.0-beta.1`: Testing version before 0.2.0 stable
