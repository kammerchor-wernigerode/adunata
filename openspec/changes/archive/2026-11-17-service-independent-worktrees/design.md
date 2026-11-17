# Design

## Context

See proposal.md for the motivation.

Today the development settings come from three places. The Nix development shell exports every `ADNT_`, `PG` and `MP_` variable. The README repeats them as an export block for maintainers without Nix. The plugin overrides some of them in `.envrc.local`, which only direnv reads. The server has defaults only for settings that are safe in production, so a debug build without the shell refuses to start.

The plugin derives the slot of a worktree from the database name in `.envrc.local`. It runs `psql`, `createdb` and `dropdb` against the shared server, and it builds the client inside `nix develop`.

Vite and Playwright already fall back to ports 5173 and 8080 when their variables are unset.

The Nix checks build the server from `server/` alone and copy no dotenv file into the sandbox. They start PostgreSQL on a Unix socket and Mailpit on ports 11025 and 18025 and export `PG` variables plus `ADUNATA_TEST_MAILPIT_*`.

## Goals / Non-Goals

**Goals:**

- `git wt add`, `git wt refresh` and `git wt remove` succeed with every development service stopped and with Docker stopped.
- The main checkout needs no setting in the environment, with or without Nix, with or without direnv.
- Compose, the server and the client each read their own settings files from their own directory. No consumer depends on the shell to export a setting.

**Non-Goals:**

- Copying a database into a new worktree. The README shows how to do it by hand.
- Dev Containers. The flake already describes the toolchain, and a container would describe it a second time.
- Interpolation, quoting or multi-line values in the settings files.
- Any change to the configuration file for operators. Development keeps using the settings files.

## Decisions

### Settings files per component

Not architecturally significant, because only developers see these files and a release build ignores them.

Each component reads dotenv files from its own directory. A `.env.local` overrides `.env`, as in Vite and other tools that use dotenv. No file is a template. A tracked file is loaded as it is, and a gitignored file exists only where something overrides. In a worktree the plugin owns its own keys in these files and leaves every other line alone, so a developer can add personal overrides there too.

|Component|Directory|Tracked|Gitignored, written by the plugin|
|-|-|-|-|
|Compose|repository root|none, defaults live in `compose.yaml`|`.env`|
|Server and its tests|`server/`|`.env`|`.env.local`|
|Vite and Playwright|`client/`|none, defaults live in the configs|`.env.local`|

The tracked `server/.env` holds the settings of the main checkout:

```
ADNT_HTTP_PUBLIC_URL=http://localhost:8080
ADNT_DATABASE_URL=postgres://adunata:adunata@localhost:5432/adunata
ADNT_SMTP_HOST=localhost
ADNT_SMTP_PORT=1025
ADNT_SMTP_ENCRYPTION=none
ADNT_MAIL_SENDER=adunata@localhost
ADNT_LOCALIZATION_TIME_ZONE=Europe/Berlin
MAILPIT_UI_PORT=8025
```

`MAILPIT_UI_PORT` has no `ADNT_` prefix, because only tests read it and the prefix belongs to settings of the server. Compose uses the same name.

Values stay unquoted and free of `${…}`. Compose, Node and the server parse dotenv slightly differently, but they agree on plain `KEY=value` lines. The mail sender is a bare address for that reason.

A worktree has the same ports in three files, because Compose binds them, the server connects to them and Vite forwards to them. The plugin writes all three files from one slot, so they cannot drift apart.

Alternatives considered:

- One `.env` and one `.env.local` in the repository root for every component. Compose reads only `.env`, so a worktree needed a generated `compose.override.yaml` with `!override` for its ports. Compose also saw every server setting, and Vite read files outside its own directory.
- Compiled defaults for debug builds. They hide the values from the README and from anyone who reads the checkout.
- A YAML or TOML file in the style of Spring profiles. Compose reads only dotenv, so the plugin would write two formats.

### Server reads its files in debug builds

This decision touches the architecture, because section 08 names the sources of the configuration. It needs no record, because it changes nothing for a release build and is easy to reverse.

The server takes each setting from the first source that sets it: the environment, then the configuration file named with `--config`, then `server/.env.local`, then `server/.env`. A file the developer names explicitly wins over files the debug build loads on its own. Each settings file is a source of its own in the list that `Configuration::load` already takes, so the rule that one source must not set both forms of a secret applies to each file separately. Parsing and validation stay as they are.

The server reads the files only when `cfg!(debug_assertions)` holds and only in its working directory. `cargo run` and `cargo test` run in `server/`, so they find them. A server started from the repository root finds no file and reports the missing settings. It never reads the `.env` of Compose there. A missing file counts as empty, which covers the Nix sandbox. A release binary never opens a file, so a stray `.env` next to a production binary cannot change its configuration.

The settings files take an optional directory. `from_environment_and_file` passes the working directory when `cfg!(debug_assertions)` holds and nothing otherwise. Unit tests pass a temporary directory of their own, so they neither touch the files of the checkout nor change the working directory of the test process. A pure parser over a string covers the line format.

The tests in `server/tests/startup.rs` spawn the debug binary with a cleared environment and expect settings to be missing. Each test creates a fresh empty directory below the temporary directory of the system and runs the child there. The shared temporary directory itself could hold a `.env` from another user.

Alternatives considered:

- A search upward from the working directory. From the repository root it would find the `.env` of Compose.
- A path fixed at compile time through `CARGO_MANIFEST_DIR`. A spawned debug binary would always see the files of its checkout, so the startup tests could not opt out.
- An empty value in the environment that hides the file value. The server already treats an empty value as unset. Changing that would alter behavior in production.

The parser takes `KEY=value` lines, skips blank lines and lines starting with `#` and needs no dependency. Leading whitespace is ignored. A line without `=`, with a key that is not a valid variable name, such as `export X=1`, or with a quoted value or a trailing ` #` comment is a problem that names the file and the line number. Other dotenv readers strip quotes and trailing comments, so the server rejects them instead of reading a different value. A key that appears twice in one file takes its last value, as in other dotenv readers. The server reports it with the other problems of the configuration. An empty value counts as unset, as it does in the environment, so `KEY=` in `.env.local` does not hide the value in `.env`.

### Tests use the same lookup

Not architecturally significant, because it changes only how tests find their services.

`empty_database` builds its server connection from `ADNT_DATABASE_URL` instead of the `PG` variables. The mail tests take the SMTP port from `ADNT_SMTP_PORT` and the API port of Mailpit from `MAILPIT_UI_PORT`. `ADUNATA_TEST_MAILPIT_SMTP_PORT` and `ADUNATA_TEST_MAILPIT_HTTP_PORT` go away. The Nix checks export `ADNT_DATABASE_URL=postgres:///postgres?host=$PGHOST&user=adunata` and the two ports instead.

The configuration module exposes a lookup for tests over the environment and both settings files. It reads the files once and stops the test run if one of them is malformed, so a broken `.env.local` never sends tests to the services of another checkout. Test code in other modules calls it, so the rule that only `configuration` reads settings still holds.

### Client reads its files

Not architecturally significant, because Vite and Playwright run only in development and tests.

`client/vite.config.ts` calls `loadEnv` with `client/` as directory and an empty prefix. Vite then reads `client/.env.local` and lets the environment win. `client/playwright.config.ts` loads `client/.env.local` with `process.loadEnvFile` if it exists. It never overrides a variable already set. Neither file exposes a variable to the browser bundle, because Vite exposes only the prefix `VITE_` there.

### One Compose project per worktree

Not architecturally significant, because Compose runs only the services for development.

`compose.yaml` keeps `name: adunata` and takes its host ports from variables with defaults:

```yaml
ports:
  - "127.0.0.1:${POSTGRES_PORT:-5432}:5432"
```

`MAILPIT_SMTP_PORT` and `MAILPIT_UI_PORT` work the same way with 1025 and 8025. In a worktree the plugin writes the root `.env`:

```
# Slot n of git-worktree-init
COMPOSE_PROJECT_NAME=adunata-<b>
POSTGRES_PORT=<b+2>
MAILPIT_SMTP_PORT=<b+3>
MAILPIT_UI_PORT=<b+4>
```

Compose reads `.env` on its own. `COMPOSE_PROJECT_NAME` from `.env` wins over `name:` in `compose.yaml`, and the port variables win over their defaults. Compose 5.7 shows both behaviors. The main checkout has no root `.env`, so it keeps the project `adunata` and ports 5432, 1025 and 8025 whatever its directory is called.

`<b>` is the port base of the slot, such as `adunata-18010`. It is unique wherever the ports are unique, also across clones with different `port_base`. It survives `git worktree move`. Teardown computes it from the slot and `port_base` without reading any file.

Alternatives considered for the name:

- The directory name of the worktree. Two worktrees such as `../a/feature` and `../b/feature` in one clone would share a project, and removing one would delete the database of the other. A move would orphan the project.
- `adunata-wt<n>`. It is unique per clone, but two clones collide.

Each project has its own volume, so its database and its test databases stay inside the worktree. The database `adunata` comes from `POSTGRES_DB` on the first `docker compose up`. Migrations run when the server starts.

Alternatives considered:

- A generated `compose.override.yaml` with `name:` and `ports: !override`. It works, but it needs a second file format and a less known Compose tag.
- One shared server with a database per worktree. That is the state this change removes.

### Plugin

Not architecturally significant, because the plugin is optional tooling for developers.

For a slot `n` with base `b = port_base + 10n` the plugin writes the root `.env` shown above and two more files.

`server/.env.local`:

```
ADNT_HTTP_LISTEN_ADDRESS=127.0.0.1:<b>
ADNT_HTTP_PUBLIC_URL=http://localhost:<b>
ADNT_DATABASE_URL=postgres://adunata:adunata@localhost:<b+2>/adunata
ADNT_SMTP_PORT=<b+3>
MAILPIT_UI_PORT=<b+4>
```

`client/.env.local`:

```
ADUNATA_VITE_PORT=<b+1>
ADNT_HTTP_LISTEN_ADDRESS=127.0.0.1:<b>
ADNT_HTTP_PUBLIC_URL=http://localhost:<b>
ADUNATA_E2E_URL=http://localhost:<b>
ADUNATA_E2E_MAILPIT_URL=http://127.0.0.1:<b+4>
```

`slot_of` reads the slot back from the first line of the root `.env`. The ports alone are ambiguous once `port_base` changes. A worktree that an earlier version of the plugin set up has no root `.env` yet. `slot_of` then reads its slot from the database name in `.envrc.local`, so the slot stays taken until the worktree is rebased and refreshed.

`apply` updates the three files. It replaces the lines of its own keys and the slot line, keeps every other line and appends keys that are missing. It keeps a backup of a file it changes as `<name>.bak`. `.gitignore` covers the files and their backups. Untracked files would make `git wt remove` refuse, because the tool skips a worktree with changes. On an older branch whose `.gitignore` lacks them, the plugin adds them to `info/exclude`. A file in the root gets a leading slash there, so that `.env` in a subdirectory stays visible. `.gitignore` also keeps `.envrc.local` and its backup, because a rebased worktree holds them until `git wt refresh` removes them. `apply` then builds the client. It runs `npm` inside `nix develop` when `nix` is on the path and directly otherwise. It never calls Docker or PostgreSQL.

A worktree on a branch older than this change has no tracked `server/.env`, and its server reads no file. `apply` then leaves `.envrc.local` in place and `next_steps` asks to rebase onto main. Otherwise it removes the `.envrc.local` that an earlier version of the plugin left behind. Removing it on an old branch would point that worktree at the database of the main checkout.

`teardown` runs `docker compose -p adunata-<b> down --volumes` from the main checkout. Compose finds the containers by project label, so the worktree directory may already be gone. If Docker is missing or stopped, `teardown` prints a warning with the command to run later and returns normally.

The options shrink to `port_base`. `clone_db` goes away.

### Shell and Nix

Not architecturally significant, because the development shell does not build or run any artifact.

`.envrc` keeps only `use flake`. The development shell keeps the Rust and Node tools and `RUSTC_WRAPPER`. It drops PostgreSQL, Mailpit and every `ADNT_`, `PG` and `MP_` variable.

No shell exports a setting, because the environment wins over every file. A process started from a shell in the main checkout, such as a coding agent or an IDE, would otherwise carry the database URL of the main checkout into every worktree it works in. Server and tests there would use the database of the main checkout without any warning.

`docker compose exec postgres psql -U adunata` reaches the database of the current checkout. `pg_dump` runs the same way, so no client tools are needed on the host.

Alternatives considered:

- `dotenv` in `.envrc`. It puts the settings into the shell for `psql`, but it brings back the leak between checkouts.

### Architecture documentation and decisions

The change is almost entirely about the development environment, which the README documents. Only the rule that a debug build of the server reads files touches the architecture, because section 08 names the sources of the configuration.

- `docs/arc42/08-cross-cutting-concepts.md`, section Configuration: a debug build also reads `.env.local` and `.env` in its working directory below the configuration file. A release build never reads them. 
- [ADR 0009](../../../docs/decisions/0009-optional-worktree-tooling.md) describes the shared database server, which goes away. Its status becomes `Deprecated` with a link to the README section on parallel worktrees. Section 09 shows the new status.
- [ADR 0005](../../../docs/decisions/0005-nix-flake-as-build-definition.md) says that contributors without Nix get a Compose file in the same versions as the development shell. After this change the shell has no services, and everyone uses the Compose file. The record stays accepted, because its decision, one flake for every build, still holds.
- No new decision record. Neither the services per worktree nor the settings files change the structure, the deployment or the quality goals of Adunata.

## Risks / Trade-offs

- [Nix users now need Docker or Podman] → The README names it as a requirement next to Nix. The Nix checks keep their own services and need no Docker.
- [A variable left in a login shell overrides a `.env.local`] → Neither `.envrc` nor the development shell exports a setting. The README says that the environment wins. `SANITIZE_ENV` keeps the plugin's own commands clean.
- [The server started outside `server/` finds no settings] → It exits and names every missing setting. The README runs it from `server/`.
- [Old worktrees keep a `.envrc.local` and their `adunata_wt<n>` databases] → After a rebase onto main, `git wt refresh` removes the file. The README shows the command that drops the old databases on the main server once.
- [A developer runs `docker compose` in a worktree before the plugin wrote its `.env`] → The project is `adunata` with the ports of the main checkout, so it shares the services of the main checkout. The README says to run `git wt refresh` first. It also warns that git-worktree-init skips worktrees under `.claude/worktrees/` by default, so they join the project of the main checkout, and `docker compose down --volumes` there deletes the main database.
- [A change of `port_base` renames the projects of existing worktrees] → Their old projects stay until `docker compose -p <name> down --volumes`. The README says so next to the option.

## Migration Plan

1. A Nix user who runs PostgreSQL through `pg_ctl` dumps the database and stops the server before the merge, because the shell loses `pg_dump` and `pg_ctl` and the old server blocks port 5432. `nix shell nixpkgs#postgresql_17` provides the tools afterwards too.
2. Merge the change. The main checkout keeps its ports and the Compose project `adunata` with its existing volume. Restore the dump into the Compose service once, or start empty.
3. Rebase every existing worktree onto main, then run `git wt refresh` and `docker compose up --detach --wait` there.
4. Drop the old `adunata_wt<n>` databases from the main server.

Rollback means reverting the commits. Compose projects of worktrees stay until `docker compose -p <name> down --volumes`.
