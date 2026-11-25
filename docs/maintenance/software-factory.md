# Software factory

The [software factory](../../AGENTS.md#software-factory) needs these tools in addition to those of the [README](../../README.md#development), with and without Nix:

- Python 3.10 or newer, for the scripts in `factory/bin/`
- the [GitHub CLI](https://cli.github.com) `gh`, signed in with an account that may push to the repository and manage its labels
- the [OpenSpec CLI](https://github.com/Fission-AI/OpenSpec) in version 2, the version that generated the skills in `.agents/skills/`. The development shell has no package for it yet. Install it with `npm install --global --prefix ~/.local @fission-ai/openspec@2` and put `~/.local/bin` on `PATH`. The prefix keeps npm out of the Nix store, which the development shell makes read-only
- a coding agent such as Claude Code or Codex that runs the skill `factory`
