---
applyTo: "**"
---

# Summary

See Readme.md at the root of the repository for a summary of the project.

## Stack

-Rust programming language

## Architecture/guidelines

-Scripts are placed in /scripts for automation of common tasks. Script can serve to setup development environment, run tests, or deploy the application. Private scripts run by the user are typically prefixed with an underscore.
-More details about the architecture can be found by traversing and listing the files and folders in the repository, with an emphasis on the src/ directory. Some folders and modules might contain a readme file with more details.

- use the script at scripts/\_show_file_tree.ps1 to get a detailed view of the file structure. The module names are often self-descriptive.
- Factory like functions are found at src\service_factories. These are used to create complex services that depend on multiple other services.
- There should be no "factory calls" from within the "service factory functions". We should inject the dependencies and not instantiate them.
- Wallet factory is found at src\wallet\factory. This is used to create wallet instances.
- Typically, when there is a "gateway" service or module in a sub-module, that is the entry point for the whole module.
- Module types are in the <module>/types.mod module. We put business logic outside of the types module. When there is more complex/non-standard business specific logic, we put the struct definition in the types module but the business logic in a domain-specific module.
- We should not inject network based configs statically at construction as these depend on each network and should be fetched dynamically, typically from the network provider.
- When we use caching, we typically favor the use of the decorator pattern, which wrap the non-caching service with the caching service. Sometimes, we can orchestrate the caching/request at the manager/orchestrator level.
- We use the decorator pattern to add cross-cutting concerns (like caching, logging) to providers. A decorator wraps a provider, implements the same trait, adds behavior, and delegates to the wrapped provider. This keeps the client code unaware of the decoration.

## Code style

-modules are placed in <...>/my_module/mod.rs
-private members of structs are prefixed with an underscore.
-we use dependency injection via traits to allow mocking of external dependencies, such as file system or database access. We want to avoid instantiating external dependencies directly in a struct, unless for simple data structs and tests.
-Avoid using unwrap() or expect() in the code. Use proper error handling via Result and the anyhow crate. Exceptions can be made in tests and quick proof of concepts.
-Lets say we have a public function named my_function with side-effects. One way to isolate the logic (making it pure and without side effects) is to create a private function \_my_function that contains the pure logic, while my_function handles the side effects (such as logging, file system access, etc). This way, we can test \_my_function in isolation without worrying about the side effects.
-Put the "use" statements(imports) at the top of the file or module, if the file has many sub modules, like when we have test modules.
-No need to systematically add documentation comments such as /// when the code is already describing the intent.

## Tests

Test are separated in these modules: unit_tests (run within the app without any other external dependencies), integration_tests (run with external dependencies, such as a database or a blockchain node), manual_tests (run manually by a developer to test specific functionality). Avoid running all the tests as this takes time. Instead, run only the relevant tests for the code you are working on.

## Scripts

Some script use python. Use the scripts at scripts/\_setup_dev_env\* to setup a python virtual environment with the required dependencies. Use pyenv to manage multiple python versions if needed.

- `scripts/utils/_copilot_run.ps1` - Wraps commands with colored markers (cyan start, green done) to distinguish Copilot-automated terminal commands. Captures stderr to prevent PowerShell red error blocks. Usage: `.\scripts\utils\_copilot_run.ps1 "cargo test --lib"`

## Devops

Devops script are found in the scripts/devops folder. These can be used to setup the development environment, run tests, or deploy the application.

## Documentation

There are some Gherkin spec files accross the code, which are used as documentation of the expected behavior of the code. These are found in _specs folders close to the implementation. i suggest you search for these relevant files, when needed. They have this extension: ".feature".

If needed, especially when writing or reviewing new Gherkin specs, read instructions at .github\instructions\gherkin.md 

## Ai plan suggestions

-Traverse the repository to understand the structure of the codebase. There is a powershell script for that at {workspace}/scripts/utils/\_show_file_tree.ps1

## Related repositories.

Note: use the mcp servers such as the github one to connect to the related repositories.

If any of the following repositories are private, please ask for access. If not found, for the correct links.

https://github.com/DragonAxeSoftware/qrs-core-rust

## More specific instructions

Instruction files can be found. Don't read them unless needed.

- Profiling: .github/instructions/profiling.md
