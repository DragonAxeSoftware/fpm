# UI Assets Bundle

A collection of common UI assets for use across projects.

## Contents

- `assets/icons/` - SVG icons
- `assets/styles/` - Base CSS stylesheets

## Usage

Add this bundle to your project's `bundle.toml`:

```toml
[bundles.ui-assets]
git = "https://github.com/DragonAxeSoftware/fpm-example-1.git"
```

Then run:

```bash
fpm install
```

## Version Tracking

The `version` field in `bundle.toml` is used by fpm integration tests to verify push functionality. It gets incremented automatically during test runs.
