# Rename single-file modules from <mod_name>/mod.rs to <mod_name>.rs
# Run from the src directory
# Pass an array of directory paths to rename

param(
    [Parameter(Mandatory=$true)]
    [string[]]$Dirs
)

foreach ($dir in $Dirs) {
    $modPath = Join-Path $dir "mod.rs"
    $newPath = $dir + ".rs"
    if (Test-Path $modPath) {
        Move-Item -Path $modPath -Destination $newPath -Force
        Write-Host "Moved: $modPath -> $newPath"
    } else {
        Write-Host "Skipped (not found): $modPath"
    }
}
