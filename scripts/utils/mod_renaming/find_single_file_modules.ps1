# Find all directories containing only mod.rs (no sibling files)
# These are candidates for renaming to idiomatic Rust naming convention
# Run from the src directory

param(
    [string]$Path = "."
)

Get-ChildItem -Path $Path -Recurse -Directory | Where-Object { 
    $files = Get-ChildItem -Path $_.FullName -File
    $files.Count -eq 1 -and $files[0].Name -eq 'mod.rs' 
} | Select-Object -ExpandProperty FullName | ForEach-Object { 
    $_.Replace((Get-Location).Path + '\', '') 
} | Sort-Object -Property { $_.Split('\').Count } -Descending
