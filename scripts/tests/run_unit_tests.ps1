<#
.SYNOPSIS
    Runs all unit tests in the library.

.DESCRIPTION
    Executes cargo test with the unit_tests filter to run all unit tests
    while excluding integration and manual tests.

.EXAMPLE
    .\scripts\devops\run_unit_tests.ps1
#>

cargo test unit_tests
