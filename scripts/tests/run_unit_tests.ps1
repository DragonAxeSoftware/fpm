<#
.SYNOPSIS
    Runs all unit tests in the library.

.DESCRIPTION
    Executes cargo test with --lib flag to run only library tests,
    excluding integration tests and manual tests.
    Integration tests are excluded via the --skip filter.

.EXAMPLE
    .\scripts\tests\run_unit_tests.ps1
#>

cargo test --lib -- --skip integration_tests