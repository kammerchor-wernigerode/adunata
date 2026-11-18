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
- A person may have a change reviewed before its tasks with `/review-change [change] [model] [effort]`. Agents never start it on their own.
- Small fixes need no change proposal.
- An architecturally significant decision gets a record in `docs/decisions/`. Significant means that an operator or a running instance notices it. Tooling for development, tests and CI never is, even where a record mentions it. [ADR 0001](docs/decisions/0001-record-architecture-decisions.md) defines what counts. The development environment is not architecture. The README documents it.
- Plan each change from its own problem and the current code. Archived changes are no template.
- Archive a merged OpenSpec change directly on `main`, never through a pull request. Pull `main` first. The commit subject ends with `[skip ci]`, because the archive changes no code.
- When an agent repeats a mistake, add a rule here.

## Pull requests

CI runs the checks up to `integration` in the job `verify` on every push to a pull request. Pull requests that are ready for review and pushes to `main` also run the jobs `e2e` and `nixos`, which take much longer. The [testing concept](docs/arc42/08-cross-cutting-concepts.md#testing) describes the checks.

- A change is ready for human review when nothing local can still find a problem in it. Every local check is cheaper than a CI round, and every push after review starts one. So the agent that finishes a change runs `openspec-verify-change` (for OpenSpec changes), a code review of the whole branch with its own review command and the checks up to integration, and fixes what they find. Once all three pass, push the branch and open the pull request ready for review. The first push then carries the reviewed state, and `e2e` and `nixos` run once.
- Open a draft pull request only for an intermediate state that someone should see.
- Before each push, run the checks up to integration locally with `nix build .#checks.<system>.integration`, where `<system>` is the local Nix system such as `aarch64-linux`. Without Nix, run the commands that the [README](README.md#checks-before-a-push) lists. Never push a state that fails them.
- Push once per round of feedback. Collect every remark of a review, fix them together and then push.
- CodeRabbit reviews a pull request once it is ready for review. Its remarks fit well into the same round as the human ones, which saves a push and a CI run. A remark that does not hold up against the code is worth a short reply rather than a change.
- An approval covers the commit the reviewer saw. A push after it needs a new review, so mention in the pull request what changed and why.
- Dependabot opens its pull requests ready for review without the local checks above. Such a pull request may fail its checks, for example after an update of `flake.lock`. A maintainer then pushes the fix to its branch.

## Code style

- Every file follows `.editorconfig`.
- Every dependency of server and client, including fonts, icons and tools for build and test, has a license approved by the OSI. A package that only tools for build and test use may have another license if [`client/license-exceptions.json`](client/license-exceptions.json) names it with a reason. Adunata meets the conditions of each license, such as shipping a notice or license text with the artifacts that contain the dependency. The commit that adds a dependency names its license.

### Rust

Rust code stays plain, so that it is easy to read and to change.

- Write no macros of your own. Derive macros from dependencies are fine.
- Use a concrete type where it works. Introduce a trait or a generic parameter only when at least two real implementations need it.
- Prefer owned data. Clone a value rather than add a lifetime parameter to a struct or an enum.
- Every new dependency needs a reason. The commit that adds it says what the dependency does and why the project cannot do without it.
- Formatting and lints run in CI. Any warning fails the build.
- Keep a string literal on one line. A backslash continuation keeps its leading spaces, which breaks the indentation rule of `.editorconfig`.
- Each module of the server is one domain or technical concern. Other modules use only what it marks `pub`. Never widen visibility, for example with `pub(crate)`, to reach into another module. The compiler then rejects every use past a module's interface.

### TypeScript and React

- Each route has its own folder under `client/src/routes/`. Code that only one route uses sits next to it in a folder or file with the prefix `-`. Shared components, hooks and helpers live in `client/src/components/`, `client/src/hooks/`, `client/src/lib/` or a module of their own such as `client/src/l10n/`, never in the root of `client/src/routes/`.
- Use what TanStack Router offers before writing a component for it. Layout routes wrap pages, `beforeLoad` guards them and loaders redirect with `redirect`.
- Build every control from a shadcn component. If one is missing, add it with the command line tool of shadcn instead of styling a native element.
- A file holds one component. Props and other object shapes are a `type`, never an `interface`, unless a library demands one.
- `pnpm run build` and `pnpm run lint` finish without a warning. Fix a warning instead of raising a limit or silencing a rule.

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
