# OpenSpec in Adunata

This repository uses the workflows `propose`, `explore`, `apply`, `update`, `sync`, `verify` and `archive` of the profile `custom`. The skills for them live in `.agents/skills/` and `.claude/skills/`. The reviewer `reviewer-openspec` of the [software factory](../AGENTS.md#software-factory) follows the skill of `verify`.

## Profile

The profile is global to the machine. `openspec update` deletes every skill that the active profile does not include. Before anyone runs it, make sure the profile is `custom` with the seven workflows above. Check it with `openspec config list`. Set it with:

```sh
openspec config set profile custom
openspec config set workflows '["propose","explore","apply","update","sync","verify","archive"]'
```

The profile `core` has no workflow `verify`. `openspec update` with `core` deletes the skill `openspec-verify-change`, which the factory needs.

Afterwards commit the regenerated skills in `.agents/skills/` and `.claude/skills/` together.
