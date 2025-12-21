# Refactor Module Names from mod.rs to Idiomatic Naming

Rename single-file modules from `<mod_name>/mod.rs` to `<mod_name>.rs` to follow Rust's idiomatic naming convention.

## Scripts

PowerShell scripts for this refactoring are located at: `scripts/utils/mod_renaming/`

See the [README](../../scripts/utils/mod_renaming/README.md) for detailed usage instructions.

## Workflow

1. **Find** single-file modules using `find_single_file_modules.ps1`
2. **Rename** modules in batches using `rename_modules.ps1` (process deepest nested first)
3. **Clean up** empty directories using `remove_empty_dirs.ps1`
4. **Verify** compilation with `cargo check` and `cargo build`

## Important Notes

- **No import changes needed**: Rust handles both `mod.rs` and `<name>.rs` transparently
- **Keep directories with multiple files**: Only rename when the directory contains ONLY `mod.rs`
- **Order matters**: Rename deepest nested paths first to avoid parent directory issues

## Commit Message

Use conventional commit format:

```
refactor: rename single-file modules from mod.rs to idiomatic naming

Convert N single-file modules from <mod_name>/mod.rs to <mod_name>.rs
to follow Rust's idiomatic naming convention. This improves code clarity
and maintainability by using the simpler file naming pattern for modules
that don't contain sub-modules.
```
