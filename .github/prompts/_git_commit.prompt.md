---
mode: agent
---

Show the git status and the diffs using these commands:

```bash
git status
git diff --cached
```

Then make a short, commit message summarizing the changes made. Confirm that the commit message follows conventional commit standards which could be found at: https://www.conventionalcommits.org/en/v1.0.0/#summary. If no files are staged, stage them all using:

```bash
git add .
```

Confirm that the git message is ok before actually committing.

If approved, commit by running the command:

```bash
git commit -m "<commit message>"
```
