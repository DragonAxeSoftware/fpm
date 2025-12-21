# Instructions and Guidance for DevOps Tasks

This document covers CI/CD pipelines, local testing with Act, Docker usage, and release workflows.

## GitHub Actions Workflows

Workflows are located in `.github/workflows/`:

- **ci.yml** - CI workflow for testing and building
  - Triggered manually via `workflow_dispatch`
  - Runs `cargo fmt --check`, `cargo clippy`, unit tests, and local integration tests
  - Matrix build across Ubuntu, macOS, and Windows

- **release.yml** - Release workflow for creating GitHub Releases
  - Triggered by pushing version tags (e.g., `v0.1.0`, `v1.2.3`)
  - Builds binaries for multiple targets:
    - `x86_64-unknown-linux-gnu` (Linux x64)
    - `aarch64-unknown-linux-gnu` (Linux ARM64)
    - `x86_64-apple-darwin` (macOS x64)
    - `aarch64-apple-darwin` (macOS ARM64)
    - `x86_64-pc-windows-msvc` (Windows x64)
  - Uploads `.tar.gz` for Unix, `.zip` for Windows
  - Generates SHA256 checksums

## Creating a Release

1. Ensure all changes are committed and pushed
2. Create and push a version tag:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
3. The release workflow will automatically build and upload binaries to GitHub Releases

## Local Pipeline Testing with Act

[Act](https://github.com/nektos/act) runs GitHub Actions locally using Docker containers.

### Prerequisites

- Docker must be running (via WSL on Windows)
- Act installed in WSL

### Running Docker via WSL (Windows)

On Windows, Docker Desktop may not always be accessible from PowerShell. Use Docker through WSL instead:

```powershell
# Check if Docker is accessible via WSL
wsl docker ps

# Run docker commands through WSL
wsl docker images
```

### Installing Act in WSL

```bash
# In WSL terminal
curl -s https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash
```

This installs `act` to `./bin/act` in the current directory.

### Running Act

```powershell
# List workflows that would run on push
wsl bash -c "cd /mnt/u/dev/projects/dragonaxe/repos/gitf2 && ./bin/act -l push"

# Dry-run the CI workflow
wsl bash -c "cd /mnt/u/dev/projects/dragonaxe/repos/gitf2 && ./bin/act push -W .github/workflows/ci.yml -n"

# Dry-run the release workflow with simulated tag
wsl bash -c "cd /mnt/u/dev/projects/dragonaxe/repos/gitf2 && ./bin/act push -W .github/workflows/release.yml -e .github/workflows/test-event.json -n"
```

### Act Event Files

To simulate tag pushes for testing release workflows, create an event file:

```json
{
  "ref": "refs/tags/v0.1.0"
}
```

### Act Limitations

- Only supports Linux containers (Windows/macOS jobs are skipped)
- Some GitHub Actions features may not work locally
- Use `-n` flag for dry-run to validate workflow syntax without execution

## Code Formatting (CI Requirement)

The CI pipeline checks code formatting with `rustfmt`. Before pushing:

```powershell
# Check formatting (what CI runs)
cargo fmt --all -- --check

# Auto-fix formatting issues
cargo fmt --all
```

## Linting with Clippy (CI Requirement)

The CI pipeline runs Clippy with warnings treated as errors. Before pushing:

```powershell
# Run clippy (what CI runs - treats warnings as errors)
cargo clippy --all-targets --all-features -- -D warnings

# Run clippy without failing on warnings (for development)
cargo clippy --all-targets --all-features
```

### Common Clippy Fixes

- **Unused variables**: Prefix with underscore (`_unused_var`)
- **Unnecessary clones**: Remove `.clone()` when not needed
- **Redundant closures**: Use function reference instead (`|x| foo(x)` → `foo`)
- **Missing documentation**: Add `///` doc comments or `#[allow(missing_docs)]`

### Suppressing Clippy Warnings

When a warning is intentional, suppress it locally:

```rust
#[allow(clippy::too_many_arguments)]
fn complex_function(...) { }

// Or for a single expression
#[allow(clippy::unwrap_used)]
let value = some_option.unwrap(); // Only in tests
```

## Pre-Push Checklist

Before pushing, run these commands to ensure CI will pass:

```powershell
# 1. Format code
cargo fmt --all

# 2. Check clippy
cargo clippy --all-targets --all-features -- -D warnings

# 3. Run unit tests
cargo test --lib unit_tests

# 4. Run local integration tests
cargo test --lib local_integration_tests
```

## References

### GitHub Actions for Rust

- [Build and upload Rust binary to GitHub Releases](https://github.com/marketplace/actions/build-and-upload-rust-binary-to-github-releases)
- [create-gh-release-action](https://github.com/taiki-e/create-gh-release-action)
- [upload-rust-binary-action](https://github.com/taiki-e/upload-rust-binary-action)
- [rust-toolchain action](https://github.com/dtolnay/rust-toolchain)

### Act (Local GitHub Actions Runner)

- [Act GitHub Repository](https://github.com/nektos/act)
- [Act Documentation](https://nektosact.com/usage/index.html)
- [Act Installation](https://nektosact.com/installation/index.html)

### Docker

- [Docker Desktop WSL 2 backend](https://docs.docker.com/desktop/wsl/)
- [Running Docker in WSL](https://docs.docker.com/engine/install/ubuntu/)
