# OpenSpec in Adunata

This repository uses the workflows `propose`, `explore`, `apply`, `update`, `sync` and `archive` of the profile `core`. The skills for them live in `.agents/skills/` and `.claude/skills/`. The [software factory](../AGENTS.md#software-factory) verifies a change instead of the workflow `verify`.

## Profile

The profile is global to the machine. `openspec update` deletes every skill that the active profile does not include. Before anyone runs it, make sure the profile is `core`. Check it with `openspec config list`. Set it with:

```sh
openspec config profile core
```

Afterwards commit the regenerated skills in `.agents/skills/` and `.claude/skills/` together.
