# Remove empty directories after module renaming
# Run from the src directory

param(
    [string]$Path = "."
)

Get-ChildItem -Path $Path -Recurse -Directory | Where-Object { 
    (Get-ChildItem -Path $_.FullName -File -Recurse).Count -eq 0 -and 
    (Get-ChildItem -Path $_.FullName -Directory -Recurse).Count -eq 0 
} | Sort-Object -Property FullName -Descending | ForEach-Object { 
    Remove-Item -Path $_.FullName -Force
    Write-Host "Removed empty directory: $($_.FullName)"
}
