# Adunata

/a.duˈna.ta/, Italian for gathering.

Adunata is an open-source web application for choirs and instrumental ensembles to organize rehearsals and concerts. Organizers invite members by email. Members reply through a form whether and when they will attend.

## Contents

- [Installation](#installation)
- [Documentation](#documentation)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)

## Installation

Adunata runs on NixOS, in a container or as a systemd service on any Linux distribution. The [installation guide](docs/operating/installation.md) describes each way.

## Documentation

|Path|Content|
|-|-|
|[`docs/operating/`](docs/operating/)|Installation and configuration for operators|
|[`docs/arc42/`](docs/arc42/README.md)|Architecture documentation|
|[`docs/decisions/`](docs/decisions/)|Architecture decision records|
|[`openspec/specs/`](openspec/specs/)|Current behavior as requirements|

## Development

The server in [`server/`](server/) is written in Rust. The client in [`client/`](client/) is written in React with TypeScript. The server embeds the built client, so the client has to be built before the server compiles.

Development needs PostgreSQL to store data and Mailpit to catch outgoing mail. Both run from [`compose.yaml`](compose.yaml), with and without Nix. Mailpit shows every mail at <http://localhost:8025>.

### With Nix

Install [Nix](https://nixos.org/download/) and enable flakes. Install Docker or Podman with Compose as well. The development shell provides Rust and Node.js 24. It compiles Rust through sccache, which shares compiled dependencies between checkouts.

```sh
nix develop
```

With [direnv](https://direnv.net) the shell loads by itself whenever you enter the repository and keeps your own shell with its prompt and aliases. [nix-direnv](https://github.com/nix-community/nix-direnv) caches the shell, so entering takes no time after the first load. Allow the tracked `.envrc` once:

```sh
direnv allow
```

### Without Nix

Install these tools:

- Rust, stable, with `clippy` and `rustfmt`
- Node.js 24
- Docker or Podman with Compose

### Services

Start PostgreSQL and Mailpit in the root of the checkout:

```sh
docker compose up --detach --wait
```

Open the database with `psql` inside its container:

```sh
docker compose exec postgres psql -U adunata
```

Stop the services with `docker compose down`. Add `--volumes` to delete the database as well.

### Settings

Compose, the server and the client each read their settings from files in their own directory. A `.env.local` overrides `.env`. A variable set in the environment wins over both files. No shell exports a setting, so a setting in the environment comes only from you.

|Directory|Read by|Tracked|Untracked|
|-|-|-|-|
|root|Compose|none, defaults in `compose.yaml`|`.env` with `COMPOSE_PROJECT_NAME`, `POSTGRES_PORT`, `MAILPIT_SMTP_PORT` and `MAILPIT_UI_PORT`|
|`server/`|server and its tests|`.env` with the settings for development|`.env.local`|
|`client/`|Vite and Playwright|none, defaults in the configs|`.env.local`|

The server reads its files only in a debug build and only in its working directory, so run it and its tests in `server/`. The [configuration reference](docs/operating/configuration.md) explains each `ADNT_` variable. Without an untracked file, PostgreSQL listens on port 5432, Mailpit on 1025 and 8025, the server on 8080 and Vite on 5173.

### Run Adunata

Build the client once, then start the server:

```sh
cd client
npm ci
npm run build
cd ../server
cargo run -- serve
```

The server listens on <http://localhost:8080> and applies database migrations on start.

To work on the client, keep the server running and start the Vite development server in `client/`:

```sh
npm run dev
```

It serves the client on <http://localhost:5173> and reloads on every change. It forwards requests to `/api` to the server on port 8080. `ADUNATA_VITE_PORT` in `client/.env.local` changes the port of Vite. The forwarding follows `ADNT_HTTP_LISTEN_ADDRESS` there.

### Tests and lints

|Directory|Command|Runs|
|-|-|-|
|`server/`|`cargo test`|Unit and slice tests against PostgreSQL and Mailpit|
|`server/`|`cargo clippy --all-targets -- --deny warnings`|Lints|
|`server/`|`cargo fmt --check`|Formatting|
|`client/`|`npm test`|Unit tests|
|`client/`|`npm run lint`|Lints|

`cargo test` finds PostgreSQL through `ADNT_DATABASE_URL` and Mailpit through `ADNT_SMTP_PORT` and `MAILPIT_UI_PORT`. It creates a separate database for each test and drops it when the test ends, so the user in `ADNT_DATABASE_URL` needs the right to create databases.

A test run that is killed leaves its databases behind. Drop them in the root of the checkout while no tests run there:

```sh
docker compose exec -T postgres psql -U adunata -tAc "select datname from pg_database where datname like 'adunata\_test\_%'" | xargs -rn1 docker compose exec -T postgres dropdb -U adunata --force
```

The end-to-end tests run against a release build in Nix:

```sh
nix build .#checks.x86_64-linux.endToEnd
```

### API contract

The client calls the server through code that [Orval](https://orval.dev) generates from the OpenAPI document of the server. After a change to the API, regenerate both and commit them:

```sh
cd server
cargo run -- openapi > ../client/openapi.json
cd ../client
npm run generate
```

### Checks before a push

CI runs the checks of the flake. Run the fast ones locally before each push. Replace `x86_64-linux` with your system, such as `aarch64-darwin`.

```sh
nix build .#checks.x86_64-linux.fast
```

### Parallel worktrees

Several worktrees run side by side when each one has its own ports and database. The plugin [`dev/worktree_plugin.py`](dev/worktree_plugin.py) for [git-worktree-init](https://github.com/V1ncNet/git-worktree-init) sets them up for every new linked worktree and again on `git wt refresh`. It needs Nix and direnv as described in [With Nix](#with-nix). Install git-worktree-init as its README describes, then enable the plugin once in the main checkout:

```sh
git config worktree-init.plugin dev/worktree_plugin.py
```

Each worktree gets a slot `n`. Slot 0 belongs to the main checkout, which keeps ports 8080 and 5173 and the database `adunata`. For a new worktree, the plugin:

- writes `.envrc.local` with the variables of its slot, keeps the previous file as `.envrc.local.bak` and runs `direnv allow` if direnv is installed
- creates the database `adunata_wt<n>` on the shared PostgreSQL server, which migrations fill on the first start
- installs and builds the client, so the server compiles right away

The server listens on `port_base + 10n` and Vite on `port_base + 10n + 1`. Mail of every worktree goes to the same Mailpit. The database name depends only on the slot, so only one clone of Adunata per PostgreSQL server can use the plugin.

`.envrc.local` keeps the names of the settings from the time the plugin wrote it. After rebasing a worktree onto a commit that renames a setting, run `git wt refresh` in that worktree.

|Option|Default|Meaning|
|-|-|-|
|`port_base`|`18000`|Port from which the ports of the slots count. Change it per repository if another project uses the same range.|
|`clone_db`|off|Copies the database of the main checkout instead of starting empty. If the copy is interrupted, drop the database with `dropdb adunata_wt<n>` and run `git wt refresh` again.|

Set an option per repository with `git config worktree-init.port-base 19000` or for one worktree with a flag:

```sh
git wt add --clone-db -- -b feature/foo ../adunata.foo
```

Remove a worktree with `git wt remove`, which drops its database first. It fails while PostgreSQL is down. A worktree removed with plain `git worktree remove` keeps its database until the next sweep. `git wt prune`, `git wt add` and `git wt refresh` each run one.

git-worktree-init skips the worktrees of Claude Code under `.claude/worktrees/` by default. Include them by clearing that pattern:

```sh
git config worktree-init.ignorePaths ""
```

Claude Code removes its worktrees with plain `git worktree remove`, so their databases go away at the next sweep. Its shell has no direnv hook, so it runs commands through `direnv exec . <command>`.

## Contributing

[`AGENTS.md`](AGENTS.md) holds the rules for workflow, pull requests and code style. New features start as a change proposal in [`openspec/changes/`](openspec/changes/).

## License

Adunata is licensed under the [GNU Affero General Public License v3.0](LICENSE).
