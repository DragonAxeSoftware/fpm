<#
.SYNOPSIS
    Runs all integration tests sequentially.

.DESCRIPTION
    Lists all local integration tests from the codebase, prioritizes transaction tests,
    and runs them one by one. Local integration tests are found in local_integration_tests modules.

.PARAMETER Filter
    Optional filter to run only tests matching a pattern.

.PARAMETER List
    Only list the tests without running them.

.EXAMPLE
    .\scripts\tests\run_local_integration_tests.ps1
    .\scripts\tests\run_local_integration_tests.ps1 -Filter "transactions"
    .\scripts\tests\run_local_integration_tests.ps1 -List
#>

param(
    [string]$Filter = "",
    [switch]$List
)

# Get all local integration test names
$output = cargo test local_integration_tests --lib -- --list 2>&1 | Out-String
$lines = $output -split "`r?`n"

# Parse test names (lines ending with ": test")
$tests = $lines | Where-Object { $_ -match ": test$" } | ForEach-Object { 
    ($_ -replace ": test$", "").Trim() 
}

# Apply filter if provided
if ($Filter) {
    $tests = $tests | Where-Object { $_ -match $Filter }
}

# Separate transaction tests (priority) from others
$transactionTests = $tests | Where-Object { $_ -match "transactions" }
$otherTests = $tests | Where-Object { $_ -notmatch "transactions" }

# Combine: transactions first, then others
$orderedTests = @($transactionTests) + @($otherTests)

if ($List) {
    Write-Host "`n=== Integration Tests ($(($orderedTests).Count) total) ===" -ForegroundColor Cyan
    Write-Host "`nTransaction tests (priority):" -ForegroundColor Yellow
    $transactionTests | ForEach-Object { Write-Host "  $_" }
    Write-Host "`nOther tests:" -ForegroundColor Yellow
    $otherTests | ForEach-Object { Write-Host "  $_" }
    exit 0
}

Write-Host "`n=== Running $(($orderedTests).Count) Local Integration Tests ===" -ForegroundColor Cyan
Write-Host "Transaction tests will run first.`n" -ForegroundColor Yellow

$passed = 0
$failed = 0
$failedTests = @()

foreach ($test in $orderedTests) {
    Write-Host "Running: $test" -ForegroundColor Cyan
    cargo test $test --lib -- --exact --test-threads=1 2>&1 | Out-String | Write-Host
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "FAILED - Stopping sequence" -ForegroundColor Red
        Write-Host "`n=== Summary ===" -ForegroundColor Cyan
        Write-Host "Passed: $passed" -ForegroundColor Green
        Write-Host "Failed: 1" -ForegroundColor Red
        Write-Host "Failed test: $test" -ForegroundColor Red
        exit 1
    }
    
    $passed++
    Write-Host "PASSED" -ForegroundColor Green
    Write-Host ""
}

Write-Host "`n=== Summary ===" -ForegroundColor Cyan
Write-Host "Passed: $passed" -ForegroundColor Green
Write-Host "All tests passed!" -ForegroundColor Green

exit 0
