# gitf2 - Git for Files

A file package manager that resembles Git and NPM, but for files in general. Manage file bundles using git repositories as the backend storage.

## Overview

gitf2 allows you to:
- Define file bundles using `bundle.toml` manifest files
- Fetch bundles from any git repository (SSH or HTTPS)
- Publish local bundles to remote git repositories
- Support nested/recursive bundle dependencies
- Track bundle synchronization status

## Installation

```bash
cargo install gitf2
```

Or build from source:

```bash
cargo build --release
```

## Usage

### Create a Bundle Manifest

Create a `bundle.toml` file in your project:

```toml
gitf2_version = "0.1.0"
identifier = "gitf2-bundle"
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
gitf2 install
```

Fetches all bundles defined in `bundle.toml` and places them in `.gitf2/` subdirectories.

#### Check Status

```bash
gitf2 status
```

Shows the synchronization status of all bundles:
- **synced**: Bundle matches its remote source
- **unsynced**: Bundle has local changes or hasn't been downloaded
- **source**: This is a source bundle (has artifacts to publish)

#### Publish Bundles

```bash
gitf2 publish
```

Pushes local source bundle changes to the configured git remotes. Use this when you're the **author** of a bundle and want to publish updates.

#### Push Bundle Changes

```bash
gitf2 push
gitf2 push -b ui-assets              # Push specific bundle
gitf2 push -m "Update styles"        # Custom commit message
```

Pushes local changes in **installed** bundles back to their source repositories. Use this when you're a **consumer** who made changes to installed bundles and wants to contribute back.

The command starts from the current manifest location and recursively pushes all nested bundles (deepest first, then parent bundles). This ensures dependent bundles are updated before their parents.

### Options

```bash
gitf2 --help
gitf2 -m path/to/bundle.toml install
```

## Bundle Structure

When bundles are installed, they're placed in `.gitf2` directories:

```
workspace/
├── src/
│   └── design/
│       ├── bundle.toml          # Your manifest
│       └── .gitf2/
│           ├── design-from-martha/
│           │   ├── bundle.toml  # Bundle's manifest (nested deps)
│           │   └── ...files...
│           └── shared-components/
│               └── ...files...
└── .gitignore                   # Add .gitf2/ to ignore
```

## Source Bundles

To create a publishable bundle, add the `root` property:

```toml
gitf2_version = "0.1.0"
identifier = "gitf2-bundle"
description = "My reusable components"
root = "components"

[bundles]
# Dependencies go here
```

## Example Repositories

The following example bundles are used for integration testing:

- [gitf2-example-1](https://github.com/DragonAxeSoftware/gitf2-example-1) - UI assets (leaf bundle)
- [gitf2-example-2](https://github.com/DragonAxeSoftware/gitf2-example-2) - UI components (depends on example-3)
- [gitf2-example-3](https://github.com/DragonAxeSoftware/gitf2-example-3) - Base styles (leaf bundle)

### Counter and Versioning

Each example bundle contains a `counter.txt` file used for testing the `push` command:

```
version=0.0.X
count=Y
```

- **version**: Patch version incremented on each test push
- **count**: Total number of test pushes

This file is intentionally simple and safe to modify during integration tests without disrupting the bundle's actual content. When running `gitf2 push` tests, only `counter.txt` is updated, leaving other files untouched.

## License

MIT
