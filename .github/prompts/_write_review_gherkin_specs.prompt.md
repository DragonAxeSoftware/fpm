---
mode: agent
---

Read the files in the current modules and write gherkin specifications files in English. Apply the instructions and guidance found at .github/instructions/gherkin.md and .github/instructions/common.instructions.md. Place the files close to where the implementation is. For top level specs, place them at the module root, in the \_specs folder. For specific implementations, place them in a \_specs folder close to the implementation. Name the files with a .feature extension.

Dont hesitate to traverse and read dependencies and other modules to understand the context and write better specifications.

If there are already some specifications, review, update, correct and/or enrich them with the current state of the code. To know which commit is linked to the current code, check the last commit that modified the file.

When writing the gherkin specifications, make sure to follow best practices for writing clear and effective gherkin scenarios.

As a genral workflow to refactor code based on new specs, when rules are added or modified in the specifications, i suggest you first check for the git changes/additions in the .feature files. Then you gonna know what to do.

It is possible some of the specs are written by human, introducing typos and formatting errors. Correct them. In doubt, stop and ask me for clarifications.
