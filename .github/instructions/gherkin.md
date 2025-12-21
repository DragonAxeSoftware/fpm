# Instructions and Guidance regarding Gherkin Specifications

Gherkin artifacts and specs should be placed alongside the code, typically in a `_specs` folder within the relevant module or feature directory. I suggest you check other module spec files to see how this is typically done and add some context. For that, I think you can search for files with the `.feature` extension in the repository.

Some traits are linked to specs. For the concrete implementation, make sure not to duplicate the specs, but extends or refine them if needed.

When writing Gherkin specifications, follow best practices to ensure clarity and effectiveness.

Avoid putting implementation details in the Gherkin specs, unless we are defining specs for a concrete or specific implementation. Avoid putting blockchain specific details in the Gherkin specs, unless we are defining specs for a concrete or specific implementation tied to a particiular blockchain. Example, we should not see things like "UTXO", "scriptpubkey" and such when dealing with high-level multichain abstractions. You can use blockchain/implementation specific terms in the examples.

Above the concrete types and/or traits, comment the path to the relevant Gherkin spec files, so that it is easy to find them.

## References

General documentation starting point about Gherkin can be found at:
https://cucumber.io/docs/

Language doc can be found at:
https://cucumber.io/docs/gherkin/reference

Good practices:
https://automationpanda.com/2017/01/30/bdd-101-writing-good-gherkin/