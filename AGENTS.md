# Adunata

This file routes. It names where things live and the rules that apply everywhere. Content belongs in the files it points to.

## Where things live

|Path|Content|
|-|-|
|`openspec/specs/`|Current behavior as requirements|
|`openspec/changes/`|Proposed and in-progress changes|
|`openspec/config.yaml`|Rules for OpenSpec artifacts|
|`docs/decisions/`|Architecture decision records, as described in [ADR 0001](docs/decisions/0001-record-architecture-decisions.md)|

## Workflow

- New features start with `/openspec-propose <name>`. Open ideas start with `/openspec-explore`.
- Small fixes need no change proposal.
- An architecturally significant decision gets a record in `docs/decisions/`.
- When an agent repeats a mistake, add a rule here.

## Code style

- Every file follows `.editorconfig`.

## Language

- Code, comments, commit messages, specs, agent files, documentation and domain terms are English.

## Documentation

- Documentation describes the current state and, where planned, the target state. It never describes the past.
- Documentation is not a changelog. Git records changes. Specs and plans are exempt, because they capture a target state at a point in time. Architecture decision records are exempt too, because they keep superseded decisions next to the ones that replace them.
- Each fact has one home. Documentation links to it instead of copying it. Links are relative and must resolve.
- Documentation, issues and pull requests name no maintainer or contributor.
- Architecture documentation lives in `docs/arc42/` and follows the skill `arc42`.
- Markdown has no hard line breaks. A paragraph is one line. Tables use minimal padding (`|-|-|`).

## Local setup

Tracked files hold nothing specific to one machine or person. Absolute paths, user-scope skills and personal preferences go in a gitignored local file such as `CLAUDE.local.md`. The file `.worktreeinclude` copies it into new worktrees.
