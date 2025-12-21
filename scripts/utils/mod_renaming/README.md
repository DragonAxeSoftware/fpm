# Module Renaming Scripts

Scripts for renaming single-file modules from `<mod_name>/mod.rs` to `<mod_name>.rs` to follow Rust's idiomatic naming convention.

## Scripts

### find_single_file_modules.ps1

Find all directories containing only `mod.rs` (no sibling files). These are candidates for renaming.

```powershell
cd src
..\scripts\utils\mod_renaming\find_single_file_modules.ps1
```

### rename_modules.ps1

Rename modules by moving `mod.rs` files to `<mod_name>.rs`. Process deepest nested paths first to avoid conflicts.

```powershell
cd src
$dirs = @('path\to\module1', 'path\to\module2')
..\scripts\utils\mod_renaming\rename_modules.ps1 -Dirs $dirs
```

### remove_empty_dirs.ps1

Remove empty directories after module renaming.

```powershell
cd src
..\scripts\utils\mod_renaming\remove_empty_dirs.ps1
```

## Full Workflow

```powershell
cd src

# 1. Find candidates
$candidates = ..\scripts\utils\mod_renaming\find_single_file_modules.ps1
$candidates

# 2. Rename in batches (deepest first - script output is already sorted)
..\scripts\utils\mod_renaming\rename_modules.ps1 -Dirs $candidates

# 3. Remove empty directories
..\scripts\utils\mod_renaming\remove_empty_dirs.ps1

# 4. Verify compilation
cd ..
cargo check
cargo build
```

## Notes

- **No import changes needed**: Rust handles both `mod.rs` and `<name>.rs` transparently
- **Process deepest first**: The find script sorts by depth (deepest first) to avoid parent directory issues
- **Verify after renaming**: Always run `cargo check` after renaming to ensure compilation still works
