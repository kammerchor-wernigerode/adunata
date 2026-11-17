# Proposal

## Why

The plugin for git-worktree-init creates and drops a database on the shared PostgreSQL server. So `git wt add` and `git wt remove` fail while PostgreSQL is down, and nobody can create a worktree without first starting a service. Creating a worktree must not depend on any running development service.

The settings for development also live only in the Nix development shell. A maintainer without Nix copies an export block from the README, and a worktree changes them through a file that only direnv reads.

## What Changes

- Each worktree runs its own PostgreSQL and Mailpit from `compose.yaml` as a separate Compose project. The main checkout keeps the project `adunata` with ports 5432, 1025 and 8025.
- Compose, the server and the client each read dotenv files from their own directory. A `.env.local` overrides `.env`, as in Vite.
  - Compose takes its project name and host ports from a gitignored `.env` in the repository root. `compose.yaml` holds the defaults.
  - The server takes its settings from the tracked `server/.env` and the gitignored `server/.env.local`.
  - Vite and Playwright take their ports and URLs from the gitignored `client/.env.local`. Their configs hold the defaults.
- A debug build of the server reads both of its files from its working directory. Each file is a source below the environment and the configuration file. A release build reads neither file and behaves as before. Tests read the database and Mailpit through the same sources.
- The plugin no longer touches any service. For a new worktree it writes the root `.env`, `server/.env.local` and `client/.env.local` with the ports of the slot. It replaces only its own keys and keeps other lines. The Compose project of a worktree is named after the port base of its slot, such as `adunata-18010`.
- The plugin builds the client with `npm` when Nix is not installed.
- Teardown runs `docker compose down --volumes` for the project of the worktree. When Docker is not running it warns and leaves the containers, but it never fails the removal.
- **BREAKING** for development: the option `clone_db` goes away. The README shows a one-line copy between two Compose projects instead.
- **BREAKING** for development: the Nix development shell keeps only the Rust and Node tools. It drops PostgreSQL, Mailpit and every `ADNT_`, `PG` and `MP_` variable. A Nix user starts the services with Compose like everyone else.
- `.envrc` keeps only `use flake` and no longer sources `.envrc.local`. No shell exports a setting, so a process started in the main checkout cannot carry its database URL into a worktree.
- ADR 0009 becomes deprecated, because its shared database server goes away. The change adds no decision record, because it changes the development environment and not the architecture.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `instance-configuration`: The requirement "Configuration from the environment" gains the rule that a development build also reads two settings files in its working directory below the configuration file. A release build never reads them.

## Impact

- `server/src/configuration.rs` adds `.env.local` and `.env` as two sources after the configuration file in debug builds. `server/src/database.rs` and `server/src/mail.rs` use it in tests. `server/tests/startup.rs` runs the binary outside the checkout. No new dependency.
- `client/vite.config.ts` and `client/playwright.config.ts` load `client/.env.local`.
- `compose.yaml` takes its host ports from variables with defaults. `dev/worktree_plugin.py` is rewritten around the three files.
- `flake.nix` removes the variables, PostgreSQL and Mailpit from the development shell. The Nix checks set `ADNT_DATABASE_URL` instead of the `PG` variables.
- New tracked file `server/.env`. `.gitignore` gains `/.env` and `.env.local` and keeps `server/.env` tracked even where a global gitignore excludes `.env`.
- `README.md`, ADR 0009, `docs/arc42/` and `docs/operating/configuration.md` change.
- Every maintainer needs Docker or Podman with Compose, also with Nix.
