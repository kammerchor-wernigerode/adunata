# Proposal

## Why

Two worktrees of Adunata cannot run side by side today. Both bind the server to port 8080 and Vite to port 5173, and both write to the same development database `adunata`. A new worktree also lacks `client/node_modules` and `client/dist`, so the server does not compile until someone builds the client by hand. Entering the development shell takes `nix develop` and drops the login shell with its prompt, aliases and completion.

## What Changes

- A tracked `.envrc` loads the flake's development shell through direnv into the login shell. It also sources an untracked `.envrc.local` if one exists. Without that file the defaults of the development shell apply, so the main checkout keeps ports 8080 and 5173 and the database `adunata`.
- A tracked plugin for git-worktree-init prepares every new linked worktree. A developer opts in once with `git config worktree-init.plugin` in the main checkout. Without the tool installed the plugin does nothing.
  - The tool hands each worktree a slot `n`. Slot 0 is reserved for the main checkout.
  - The plugin writes `.envrc.local` with the listen address, the public URL, the URL for end-to-end tests, the Vite port and the database URL of the slot.
  - The server listens on `port_base + 10n` and Vite on `port_base + 10n + 1`. The option `port_base` defaults to 18000. A developer changes it per repository, so other projects running on the same machine do not collide.
  - The plugin creates the database `adunata_wt<n>` on the shared PostgreSQL server. It is empty by default and migrations build the schema on start. The option `clone_db` copies the main checkout's database instead.
  - The plugin installs and builds the client, then runs `direnv allow` in the worktree.
  - Its teardown drops `adunata_wt<n>` when the worktree goes away, through `git wt remove` or the next `git wt prune`.
- The Vite configuration reads its port and the address of its `/api` proxy from the environment and falls back to today's values.
- The development shell sets sccache as the Rust compiler wrapper, so worktrees share compiled dependencies instead of each building them from scratch.
- Tests drop the databases they create. Today every `cargo test` run leaves `adunata_test_*` databases behind on the shared server.
- The README describes direnv, the plugin and the local Git configuration it needs, including how to include worktrees of Claude Code.

Mail stays shared. Every worktree delivers to the same Mailpit, and tests already tell their mails apart by a unique token.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. The change affects only the development environment and the test setup, not the behavior of Adunata. The change therefore sets `skip_specs: true`.

## Impact

- New tracked files: `.envrc` and the plugin for git-worktree-init.
- `.gitignore` and `.worktreeinclude` gain `.envrc.local` and the direnv cache.
- `client/vite.config.ts` reads its port and its proxy target from the environment.
- `flake.nix` adds sccache to the development shell. sccache is a new development dependency. It caches compiler output across worktrees, which Cargo cannot do without locking a shared target directory.
- The test helpers in `server/src/database.rs` drop each test database after use.
- `README.md` gains a section on parallel worktrees.
- Outside the repository the plugin needs git-worktree-init with teardown support, which a developer installs on their own machine.
