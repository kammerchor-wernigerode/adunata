# Tasks

Each group is one commit that passes `nix build .#checks.<system>.fast`.

## 1. Settings files for debug builds of the server

- [x] 1.1 Add the tracked `server/.env` with the values from design.md. Add `/.env`, `/.env.bak`, `.env.local` and `.env.local.bak` to `.gitignore`. Verify that `git check-ignore .env .env.bak server/.env.local server/.env.local.bak client/.env.local client/.env.local.bak` lists all six and that `git check-ignore server/.env` reports nothing.
- [x] 1.2 In `server/src/configuration.rs`, add a public lookup that returns the environment value, else the value from `.env.local`, else the value from `.env`, both in a directory the caller passes. `Configuration::from_environment` passes the working directory when `cfg!(debug_assertions)` holds and no directory otherwise. A pure parser over a string takes `KEY=value` lines, skips blank lines and `#` comments and reports a malformed line with file and line number as a configuration problem. An empty value counts as unset. Cover the parser, the precedence, missing files and the absent directory with unit tests that use their own temporary directory. Verify that `cargo test` passes and that `cargo run -- serve` starts in `server/` with every `ADUNATA_` variable unset while PostgreSQL and Mailpit run.
- [x] 1.3 Let every test in `server/tests/startup.rs` run the binary in a fresh empty directory that it creates below `std::env::temp_dir()`. Verify that `cargo test --test startup` passes in `server/`, where `.env` sets every required setting.
- [x] 1.4 Extend the section Configuration in [section 08](../../../docs/arc42/08-cross-cutting-concepts.md#configuration) and `docs/operating/configuration.md` with the rule that a release build reads only the environment and a development build also reads `.env.local` and `.env` in its working directory. Verify that the relative links resolve.

## 2. Tests find services through the settings

- [x] 2.1 Let `empty_database` in `server/src/database.rs` connect through `ADUNATA_DATABASE_URL` from the lookup instead of the `PG` variables. Let the mail tests in `server/src/mail.rs` take `ADUNATA_SMTP_PORT` and `MAILPIT_UI_PORT` from the lookup and drop `ADUNATA_TEST_MAILPIT_SMTP_PORT` and `ADUNATA_TEST_MAILPIT_HTTP_PORT`. Verify that `cargo test` passes in a shell without any `PG` or `ADUNATA_` variable.
- [x] 2.2 Let `preCheck` in `flake.nix` export `ADUNATA_DATABASE_URL=postgres:///postgres?host=$PGHOST&user=adunata`, `ADUNATA_SMTP_PORT` and `MAILPIT_UI_PORT` for its Mailpit. Do the same for every other check that sets the removed variables. Verify that `nix build .#checks.<system>.fast` passes.
- [x] 2.3 Update the README sentence on `cargo test` to say that tests find PostgreSQL and Mailpit through `ADUNATA_DATABASE_URL`, `ADUNATA_SMTP_PORT` and `MAILPIT_UI_PORT`. Verify that the README renders the variable names as written.

## 3. Client reads its settings file

- [x] 3.1 Let `client/vite.config.ts` read `ADUNATA_VITE_PORT`, `ADUNATA_LISTEN_ADDRESS` and `ADUNATA_PUBLIC_URL` through `loadEnv` from `client/` with an empty prefix. Verify that `npm run dev` serves on 5173 without `client/.env.local` and on the port of `client/.env.local` when it sets `ADUNATA_VITE_PORT`. Verify that `npm run lint` passes.
- [ ] 3.2 Let `client/playwright.config.ts` load `client/.env.local` with `process.loadEnvFile` if the file exists. Verify that `npx playwright test --list` passes with and without the file. Let the plugin's `client/.env.local` carry `ADUNATA_E2E_MAILPIT_URL` as design.md shows.

## 4. Services from Compose for every maintainer

- [ ] 4.1 Let `compose.yaml` take its host ports from `POSTGRES_PORT`, `MAILPIT_SMTP_PORT` and `MAILPIT_UI_PORT` with defaults 5432, 1025 and 8025. Keep `name: adunata`. Verify that `docker compose config` without a root `.env` reports the project `adunata` and the default ports, and with a root `.env` reports its project name and ports.
- [ ] 4.2 Rewrite the README section "Development". Docker or Podman with Compose is a requirement with and without Nix. `docker compose up --detach --wait` starts the services. `docker compose exec postgres psql -U adunata` opens the database. The `pg_ctl` block, the export block and the note on `direnv exec .` go away. A paragraph explains the settings files of each component, that the environment wins over them and that no shell exports them. Verify that each documented command runs as written.

## 5. Plugin without service dependencies

- [ ] 5.1 Rewrite `dev/worktree_plugin.py` as design.md describes. `OPTIONS` keeps only `port_base`. `slot_of` reads the slot from the first line of the root `.env`. `apply` updates its own keys in the root `.env`, `server/.env.local` and `client/.env.local` and keeps other lines. It removes an old `.envrc.local` only when the worktree has a tracked `server/.env`, and otherwise `next_steps` asks to rebase onto main. It builds the client with or without Nix and runs `direnv allow` if direnv is installed. `teardown` runs `docker compose -p adunata-<b> down --volumes` with `b` from the slot and `port_base` and only warns on failure. `SANITIZE_ENV` lists every variable that the three files and `server/.env` set. Verify with `python3 -m py_compile dev/worktree_plugin.py` and that a refresh keeps a line a developer added to `server/.env.local`.
- [ ] 5.2 Drop PostgreSQL, Mailpit and every `ADUNATA_`, `PG` and `MP_` variable from the development shell in `flake.nix`. Reduce `.envrc` to `use flake`. Remove `.envrc.local` and `.envrc.local.bak` from `.gitignore`. Verify that neither `direnv exec . env` nor `nix develop --command env` shows any of the removed variables.
- [ ] 5.3 Set the status of [ADR 0009](../../../docs/decisions/0009-optional-worktree-tooling.md) to `Deprecated` with a link to the README section on parallel worktrees and update its row in [section 09](../../../docs/arc42/09-architecture-decisions.md). Verify that the relative links resolve.
- [ ] 5.4 Rewrite the README section "Parallel worktrees". It covers the port layout of a slot, the project name `adunata-<b>` and what a change of `port_base` leaves behind, the three files the plugin writes and that it keeps lines it does not own, starting the services per worktree, the teardown that warns while Docker is down and the removal of `clone_db`. It shows the one-liner that copies the main database into a worktree through `docker compose exec` with `pg_dump` and `psql` and the command that drops leftover `adunata_wt<n>` databases from the main server. It says to rebase old worktrees before `git wt refresh` and warns that skipped worktrees under `.claude/worktrees/` share the Compose project of the main checkout. It tells Nix users to dump and stop a `pg_ctl` server before they update. Verify that each documented command runs as written.

## 6. Acceptance

- [ ] 6.1 Stop Docker. Create a worktree with `git wt add` and remove it with `git wt remove`. Verify that both succeed and that removal prints the warning with the command to run later.
- [ ] 6.2 Start Docker. Create two worktrees, run `docker compose up --detach --wait` in each root and `cargo run -- serve` in each `server/` and in the main checkout. Verify that all three serve at the same time on their own ports and that a mail sent in one worktree shows up only in its own Mailpit. Start a long `cargo test` in one worktree from a shell that entered the main checkout first. Verify that its `adunata_test_*` databases appear only in that worktree's PostgreSQL.
- [ ] 6.3 Remove one worktree with `git wt remove`. Verify that `docker compose ls --all` no longer lists its project and that `docker volume ls` no longer lists its volume.
