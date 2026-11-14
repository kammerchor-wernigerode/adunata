# Adunata

This file routes. It names where things live and the rules that apply everywhere. Content belongs in the files it points to.

## Where things live

|Path|Content|
|-|-|
|`openspec/specs/`|Current behavior as requirements|
|`openspec/changes/`|Proposed and in-progress changes|
|`openspec/config.yaml`|Rules for OpenSpec artifacts|
|`openspec/README.md`|OpenSpec skills in use and the profile they need, read before running `openspec update`|
|`docs/decisions/`|Architecture decision records, as described in [ADR 0001](docs/decisions/0001-record-architecture-decisions.md)|
|`docs/arc42/`|Architecture documentation, see the [entry page](docs/arc42/README.md)|
|`docs/operating/`|Documentation for operators who install and run an instance|

## Workflow

- New features start with `/openspec-propose <name>`. Open ideas start with `/openspec-explore`.
- Small fixes need no change proposal.
- An architecturally significant decision gets a record in `docs/decisions/`.
- When an agent repeats a mistake, add a rule here.

## Pull requests

CI runs fast checks on every push to a pull request. Pull requests that are ready for review also run the end-to-end test, the memory measurement and the NixOS test, which take much longer.

- Open a pull request as a draft. Pushing to a draft needs no approval.
- Before each push, run the fast checks locally with `nix build .#checks.<system>.fast`, where `<system>` is the local Nix system such as `aarch64-linux`. Never push a state that fails them.
- Push once per round of feedback. Collect every remark of a review, fix them together and then push.
- Mark the pull request ready for review once the work is complete.
- After an approval, push only after asking.

## Code style

- Every file follows `.editorconfig`.

### Rust

Rust code stays plain, so that it is easy to read and to change.

- Write no macros of your own. Derive macros from dependencies are fine.
- Use a concrete type where it works. Introduce a trait or a generic parameter only when at least two real implementations need it.
- Prefer owned data. Clone a value rather than add a lifetime parameter to a struct or an enum.
- Every new dependency needs a reason. The commit that adds it says what the dependency does and why the project cannot do without it.
- Formatting and lints run in CI. Any warning fails the build.
- Keep a string literal on one line. A backslash continuation keeps its leading spaces, which breaks the indentation rule of `.editorconfig`.
- Each module of the server is one domain or technical concern. Other modules use only what it marks `pub`. Never widen visibility, for example with `pub(crate)`, to reach into another module. The compiler then rejects every use past a module's interface.

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
