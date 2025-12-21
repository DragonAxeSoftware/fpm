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

```bash
cargo install fpm
```

Or build from source:

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

## License

MIT
