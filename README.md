# fpm - File Package Manager

A file package manager that resembles Git and NPM, but for files in general. Manage file bundles using git repositories as the backend storage.

## Overview

fpm allows you to:
- Define file bundles using `bundle.toml` manifest files
- Fetch bundles from any git repository (SSH or HTTPS)
- Publish local bundles to remote git repositories
- Support nested/recursive bundle dependencies
- Track bundle synchronization status

## Installation

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/DragonAxeSoftware/fpm/main/install.ps1 | iex
```

This downloads the latest release, installs to `~/.fpm/bin`, and adds it to your PATH.

### From Source

```bash
cargo install --path .
```

Or build manually:

```bash
cargo build --release
```

## Usage

### Create a Bundle Manifest

Create a `bundle.toml` file in your project:

```toml
fpm_version = "0.1.0"
identifier = "fpm-bundle"
description = "My project's design assets"

[bundles.design-from-martha]
version = "1.0.0"
git = "https://github.com/martha/designs.git"
path = "assets"

[bundles.shared-components]
version = "2.0.0"
git = "git@github.com:company/shared-components.git"
```

### Commands

#### Install Bundles

```bash
fpm install
```

Fetches all bundles defined in `bundle.toml` and places them in `.fpm/` subdirectories.

#### Check Status

```bash
fpm status
```

Shows the synchronization status of all bundles:
- **synced**: Bundle matches its remote source
- **unsynced**: Bundle has local changes or hasn't been downloaded
- **source**: This is a source bundle (has artifacts to publish)

#### Publish Bundles

```bash
fpm publish
```

Pushes local source bundle changes to the configured git remotes. Use this when you're the **author** of a bundle and want to publish updates.

#### Push Bundle Changes

```bash
fpm push
fpm push -b ui-assets              # Push specific bundle
fpm push -m "Update styles"        # Custom commit message
```

Pushes local changes in **installed** bundles back to their source repositories. Use this when you're a **consumer** who made changes to installed bundles and wants to contribute back.

The command starts from the current manifest location and recursively pushes all nested bundles (deepest first, then parent bundles). This ensures dependent bundles are updated before their parents.

### Options

```bash
fpm --help
fpm -m path/to/bundle.toml install
```

## Bundle Structure

When bundles are installed, they're placed in `.fpm` directories:

```
workspace/
 src/
    design/
        bundle.toml          # Your manifest
        .fpm/
            design-from-martha/
               bundle.toml  # Bundle's manifest (nested deps)
               ...files...
            shared-components/
                ...files...
 .gitignore                   # Add .fpm/ to ignore
```

## Source Bundles

To create a publishable bundle, add the `root` property:

```toml
fpm_version = "0.1.0"
identifier = "fpm-bundle"
description = "My reusable components"
root = "components"

[bundles]
# Dependencies go here
```

## Example Repositories

The following example bundles are used for integration testing:

- [fpm-example-1](https://github.com/DragonAxeSoftware/fpm-example-1) - UI assets (leaf bundle)
- [fpm-example-2](https://github.com/DragonAxeSoftware/fpm-example-2) - UI components (depends on example-3)
- [fpm-example-3](https://github.com/DragonAxeSoftware/fpm-example-3) - Base styles (leaf bundle)

### Version Tracking for Testing

Each example bundle has a `version` field in its `bundle.toml` manifest. Integration tests bump this version and create a `test_counter.txt` file to verify the `push` command works correctly:

```toml
fpm_version = "0.1.0"
identifier = "fpm-bundle"
version = "0.0.X"  # Incremented by integration tests
```

The `test_counter.txt` file contains an incrementing number to verify that new files can be pushed alongside existing file modifications.

## Development

### Building

```bash
cargo build --release
```

### Running Tests

```bash
# Unit tests
cargo test --lib unit_tests

# Local integration tests (requires binary built first)
cargo build && cargo test --lib local_integration_tests

# Format and lint checks
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

### CI/CD

This project uses GitHub Actions for continuous integration and releases:

- **CI Workflow** (`ci.yml`): Runs format checks, clippy, and tests. Triggered manually via the Actions tab.
- **Release Workflow** (`release.yml`): Builds binaries for all platforms when a version tag is pushed.

### Creating a Release

1. Update the version in `Cargo.toml`
2. Commit the change
3. Run the release script to create and push the tag:

```powershell
# Create release from version in Cargo.toml
.\scripts\devops\release.ps1

# Preview without making changes
.\scripts\devops\release.ps1 -DryRun
```

The script will:
1. Read the version from `Cargo.toml`
2. Create and push a git tag (e.g., `v0.2.0`)
3. Trigger the GitHub Actions release workflow

Binaries are built for:
- Linux x64 and ARM64
- macOS x64 and ARM64 (Apple Silicon)
- Windows x64

Releases appear at: https://github.com/DragonAxeSoftware/fpm/releases

## License

MIT
