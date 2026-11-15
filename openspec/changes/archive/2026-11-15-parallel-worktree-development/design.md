# Design

## Context

See proposal.md for the motivation. These facts shape the approach.

- The development shell in `flake.nix` sets every `ADUNATA_` and `PG` variable with fixed values for port 8080 and the database `adunata`.
- `client/vite.config.ts` uses the Vite default port 5173 and proxies `/api` to the fixed address `http://127.0.0.1:8080`. `client/playwright.config.ts` already reads `ADUNATA_E2E_URL`.
- The server rejects a plain HTTP public URL unless its host is `localhost`, as the [configuration reference](../../../../docs/operating/configuration.md) states.
- `empty_database` in `server/src/database.rs` creates `adunata_test_<pid>_<n>` and never drops it. `fresh_database` builds on it. 17 tests call `fresh_database` and 2 tests in `database.rs` call `empty_database`.
- git-worktree-init [1] calls a Python plugin with `apply(ctx)` on create and refresh and with `teardown(ctx)` on `git wt remove` and on every sweep. A sweep runs on `git wt prune`, create and refresh. In mode `prune` the worktree is gone and `ctx.run` starts in the main worktree. Teardown may run more than once. Slot 0 is reserved by default. Paths matching `*/.claude/worktrees/*` are skipped by default.
- The hook runs in whatever shell called `git worktree add`. That shell often lacks `createdb`, `npm` or `direnv`. It may also carry the variables of another worktree's direnv.

[1]: https://github.com/V1ncNet/git-worktree-init

## Goals / Non-Goals

**Goals:**

- Server, Vite and their databases of several worktrees run at the same time on one machine.
- A new worktree compiles without a manual client build.
- The main checkout keeps its ports and database when no plugin and no `.envrc.local` are present. Only Vite changes, because `strictPort` makes it fail on a taken port instead of moving on.

**Non-Goals:**

- Separate PostgreSQL servers or Mailpit instances per worktree.
- Parallel worktrees for developers without Nix. The compose setup in the README stays a single-checkout setup.
- Automated tests for the plugin. A manual acceptance run covers it, see tasks.md.

## Decisions

### direnv loads the development shell

The tracked `.envrc` calls `use flake` and then `source_env_if_exists .envrc.local`. The order matters, because the local file overrides the defaults of the development shell. With nix-direnv installed, `use flake` caches the shell in `.direnv/`, which `.gitignore` excludes. Without nix-direnv the shell evaluates on every entry, which is slower but correct.

Alternative considered: keep `nix develop`. It replaces the login shell and cannot pick up per-worktree values without a wrapper script.

### Variables in `.envrc.local`

For slot `n` and `port_base` `b`, the plugin writes:

|Variable|Value|
|-|-|
|`ADUNATA_LISTEN_ADDRESS`|`127.0.0.1:<b+10n>`|
|`ADUNATA_PUBLIC_URL`|`http://localhost:<b+10n>`|
|`ADUNATA_E2E_URL`|`http://localhost:<b+10n>`|
|`ADUNATA_VITE_PORT`|`<b+10n+1>`|
|`ADUNATA_DATABASE_URL`|the URL of the development shell with the database `adunata_wt<n>`|

The public URL uses `localhost`, because the server rejects plain HTTP for any other host. `PGDATABASE` stays `adunata`, since the tests use it only to connect before they create their own databases. `slot_of` reads the slot back from the database name in `ADUNATA_DATABASE_URL`, so the slot has no second home in the file.

Ten ports per slot leave room for later services of a worktree without a new layout.

### Vite reads its port and proxy target from the environment

`vite.config.ts` sets `server.port` from `ADUNATA_VITE_PORT` with fallback 5173 and sets `strictPort`, so a taken port fails loudly instead of moving to the next one. The proxy target is `http://` plus `ADUNATA_LISTEN_ADDRESS`, with fallback `127.0.0.1:8080`. The listen address is the single home of the server port. The variable avoids the `VITE_` prefix, so Vite never exposes it to the client bundle. The server reads only the `ADUNATA_` variables it knows, so the new name does not disturb it.

### Plugin location and tool environment

The plugin lives at `dev/worktree_plugin.py`. A developer opts in with `git config worktree-init.plugin dev/worktree_plugin.py` in the main checkout. It is the first Python file in the repository, so `.editorconfig` gains a `[*.py]` section with an indent of 4, as PEP 8 expects.

Every external command runs as `nix develop <main> --command ...`. That gives `createdb`, `psql`, `npm` and the `PG` variables of the development shell regardless of the calling shell. The main worktree is used in every mode, because in mode `prune` the target is gone and an old branch in the target may carry an older flake. `direnv allow` runs outside Nix, since direnv belongs to the login shell.

`SANITIZE_ENV` lists every variable the plugin writes, the `PG` variables and `DIRENV_DIR`, `DIRENV_FILE`, `DIRENV_DIFF` and `DIRENV_WATCHES`. Without that, a worktree created from inside another worktree would inherit that worktree's database URL.

Alternative considered: `direnv exec <target>`. It needs `direnv allow` first and fails on branches without `.envrc`.

### `apply` is idempotent

`apply` runs on create and on refresh, so the steps that create something check first. It refuses to run in the main worktree, which `git wt refresh` without an argument would otherwise give a slot.

1. Write `.envrc.local`, keep the previous file as `.envrc.local.bak` and call `ensure_ignored` for `.envrc.local`, `.envrc.local.bak` and `.direnv`, which covers branches older than this change.
2. Create `adunata_wt<n>` only if `pg_database` lacks it.
3. Clone only when step 2 created the database. On a failed clone, drop the database before re-raising, so a refresh does not keep a half copy.
4. Run `npm ci --prefer-offline` and `npm run build` in `client/` every time, so a refresh picks up a changed lockfile.
5. Run `direnv allow` only if `.envrc` exists and `direnv` is on the `PATH`. Otherwise `next_steps` names the missing step. Without `.envrc` it says to rebase the branch.

### Clone with `pg_dump` into `psql`

The option `clone_db` defaults to off. When on, the plugin runs `pg_dump --no-owner adunata | psql -v ON_ERROR_STOP=1 adunata_wt<n>` under `bash -o pipefail`.

Alternative considered: `createdb --template=adunata`. It is atomic but fails while any session holds a connection to `adunata`, which is the case whenever the main checkout's server runs. The dump works with a running server. Its copy is not atomic, so step 3 above drops a half copy.

### Teardown

`teardown` runs `dropdb --if-exists --force adunata_wt<n>` in both modes. It relies only on `ctx.slot`, so it works when the files are gone. `--force` ends sessions of a server still running in the removed worktree.

Claude Code removes its worktrees with plain `git worktree remove`. Their database goes away at the next sweep. Claude Code worktrees take part only after a developer clears the default `ignorePaths`, which the README describes.

### Test databases drop themselves

`empty_database` and `fresh_database` return a test database value that owns the database name and implements `Drop`. Its `drop` starts a thread with a current-thread Tokio runtime and runs `DROP DATABASE ... WITH (FORCE)` there, then joins the thread. `FORCE` closes the pool's connections that are still open. A panicking test unwinds and drops the value too. Call sites bind the value for the length of the test and take the pool or the connect options from it. Binding it to `_` would drop the database at once, so the type exposes no tuple and the accessors borrow.

Alternatives considered:

- Drop all `adunata_test_*` databases at the start of a run. Several worktrees run `cargo test` against the same server at once, so this deletes databases of a concurrent run.
- `#[sqlx::test]`. It cleans up old test databases on first use, which has the same problem across processes. It also needs `DATABASE_URL` at compile time and replaces the project's own test setup.

A run killed with `SIGKILL` or Ctrl-C still leaks its databases. The README names a cleanup command for that case.

### sccache as compiler wrapper

The development shell adds `pkgs.sccache` and sets `RUSTC_WRAPPER` to it. Dependencies from `~/.cargo/registry` have the same paths in every worktree, so their compiler output hits the cache. Crates of the workspace compile incrementally, which sccache does not cache, so they still build per worktree. The Nix builds do not use the wrapper, because it is set only in the development shell.

Alternative considered: one shared `CARGO_TARGET_DIR`. Cargo locks the directory for each build, so worktrees would wait for each other.

### Decision record

Draft for `docs/decisions/0009-optional-worktree-tooling.md`. It supersedes no record and extends [ADR 0005](../../../../docs/decisions/0005-nix-flake-as-build-definition.md).

> **ADR 0009: Optional tooling for parallel worktrees**
>
> **Context.** Coding agents and people work on several branches at once, each in its own Git worktree. Every worktree binds the same ports and writes to the same development database. Git runs no hook when a worktree goes away, so resources a worktree sets up outlive it. git-worktree-init [1] prepares new worktrees through a plugin and tears its resources down on removal or on the next prune. It is a separate project and not part of the flake. Contributors without it must still be able to develop.
>
> **Decision.** We will load the development shell through direnv and a tracked `.envrc` that sources an untracked `.envrc.local`. We will ship a plugin for git-worktree-init that gives each worktree its own ports and database. Using the tool stays optional. Without it, development works as before with the defaults of the development shell.
>
> **Status.** Accepted
>
> **Consequences.** Worktrees run side by side on one PostgreSQL server and one Mailpit. A developer installs git-worktree-init and direnv outside the flake and opts in through local Git configuration. The plugin follows the plugin interface of the tool, so a breaking change there breaks the plugin but not the build. Databases of worktrees removed without the tool remain until the next `git wt prune`. The repository gains its first Python file.

### Affected arc42 sections

|Section|Change|
|-|-|
|[08 Cross-cutting concepts](../../../../docs/arc42/08-cross-cutting-concepts.md#testing)|The server slice row says that each test drops its database afterwards.|
|[09 Architecture decisions](../../../../docs/arc42/09-architecture-decisions.md)|A row for ADR 0009.|

Section 07 stays as it is, because it describes how Adunata runs, not how it is developed. Section 11 stays as it is, because the tooling is optional and development works without it.

## Risks / Trade-offs

- The database name depends only on the slot. A second clone of Adunata on the same PostgreSQL server hands out the same slots and would share and drop the databases of the first. → Only one clone per PostgreSQL server is supported, which the README states.
- A clone interrupted by Ctrl-C or a kill leaves a partial database, which the next refresh keeps. → Drop it with `dropdb` and run `git wt refresh` again.
- A branch older than this change has no `.envrc` and an old `vite.config.ts`. Its Vite binds 5173 and its server uses the shell defaults. → The plugin still writes `.envrc.local` and the database. `strictPort` makes a port clash visible. Rebasing onto main fixes it.
- `port_base + 10n + 1` can exceed 65535 for a large `port_base` and slot. → The server and Vite fail to bind with a clear error. The plugin does not check it.
- A shell without the direnv hook ignores `.envrc.local`. A developer who runs `nix develop` in a worktree, or an agent whose shell never shows a prompt, gets the defaults and writes to the database `adunata` on port 8080. → The README names `direnv exec . <command>` for such shells. An acceptance task checks the shell of Claude Code.
- `nix develop` adds a few seconds to each plugin command on a cold evaluation cache. → It runs only on create, refresh and teardown.
- The plugin has no automated tests. → The manual acceptance run in tasks.md covers create, refresh, remove and prune.
- Test databases leaked before this change stay on the server. → The README names a one-off cleanup command.

## Migration Plan

1. Merge the change. The main checkout needs no step, because without `.envrc.local` the defaults apply.
2. A developer who wants parallel worktrees installs direnv with nix-direnv and git-worktree-init. Then they run `direnv allow` in the main checkout and set `worktree-init.plugin`.
3. Existing worktrees get their slot through `git wt refresh <worktree>`.
4. Leaked test databases go away once, while no tests run, with the cleanup command from the README.

Rollback is `git config --unset worktree-init.plugin` and `git wt prune` after the worktrees are gone.
