# Example Bundle Repositories

This folder contains example bundle content that should be hosted on real git repositories for integration testing.

## Required Repositories

The integration tests expect the following public git repository to exist:

| Local Folder | Remote Repository |
|--------------|-------------------|
| `example-1/` | https://github.com/<your-org>/fpm-example-1 |

## Setup Instructions

### 1. Create the Remote Repository

1. Create a new public repository on GitHub (or your preferred git host)
2. Name it `fpm-example-1` (or update the URL in `src/integration_tests/mod.rs`)

### 2. Push the Example Content

```bash
cd example-bundle-repos/example-1

# Initialize git repository
git init -b main

# Add all files
git add .
git commit -m "Initial commit: example bundle for fpm integration tests"

# Add remote and push
git remote add origin https://github.com/<your-org>/fpm-example-1.git
git push -u origin main
```

### 3. Verify the Setup

Run the integration tests to verify everything is configured correctly:

```powershell
.\scripts\tests\run_integration_tests.ps1
```

## Bundle Structure

Each example bundle should contain:

- `bundle.toml` - The fpm manifest file
- `README.md` - Bundle documentation
- `assets/` - Example asset files (icons, styles, etc.)

## Adding New Example Bundles

1. Create a new folder under `example-bundle-repos/` (e.g., `example-2/`)
2. Add the required bundle files including `bundle.toml`
3. Create the corresponding remote repository
4. Add a new constant in `src/integration_tests/mod.rs` for the repository URL
5. Create integration tests that use the new bundle
