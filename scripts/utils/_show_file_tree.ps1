function Show-Tree {
    param(
        [string]$Path,
        [int]$Depth = 2,
        [int]$Level = 0,
        [string[]]$Exclude = @()
    )

    if ($Level -ge $Depth) { return }

    Get-ChildItem -Path $Path | ForEach-Object {
        if ($_.PSIsContainer -and $Exclude -contains $_.Name) {
            return
        }
        
        Write-Output (" " * ($Level * 2) + "|-- " + $_.Name)
        if ($_.PSIsContainer) {
            Show-Tree -Path $_.FullName -Depth $Depth -Level ($Level + 1) -Exclude $Exclude
        }
    }
}

Show-Tree -Path "." -Depth 20 -Exclude @("target", ".github", ".vscode")
