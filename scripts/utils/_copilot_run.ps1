<#
.SYNOPSIS
    Wraps a command with colored markers to distinguish Copilot-automated commands.

.DESCRIPTION
    Runs a command with cyan "COPILOT RUNNING" header and green "COPILOT DONE" footer.
    Captures stderr to prevent PowerShell from displaying it as red error blocks.

.PARAMETER Command
    The command to execute.

.EXAMPLE
    .\scripts\utils\_copilot_run.ps1 "cargo test --lib"
    .\scripts\utils\_copilot_run.ps1 "cargo build --release"
#>

param(
    [Parameter(Mandatory=$true, Position=0)]
    [string]$Command
)

Write-Host "`n======== COPILOT RUNNING ========" -ForegroundColor Cyan
Invoke-Expression "$Command 2>&1" | Out-String | Write-Host
Write-Host "======== COPILOT DONE ========" -ForegroundColor Green
