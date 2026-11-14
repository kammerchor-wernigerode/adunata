# OpenSpec in Adunata

This repository uses the workflows `propose`, `explore`, `apply`, `update`, `sync`, `verify` and `archive`. The skills for them live in `.agents/skills/` and `.claude/skills/`.

## Profile

The profile is global to the machine. The default profile `core` has no `verify`. `openspec update` deletes every skill that the active profile does not include, so running it with `core` silently removes `openspec-verify-change`.

Before anyone runs `openspec update`, make sure the profile contains `verify`. Check it with `openspec config list`. Enable it with:

```sh
openspec config set profile custom
openspec config set workflows '["propose","explore","apply","update","sync","verify","archive"]'
```

Afterwards commit the regenerated skills in `.agents/skills/` and `.claude/skills/` together.
