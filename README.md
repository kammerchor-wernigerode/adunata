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

Adunata runs on NixOS, in a container or as a systemd service on a Linux distribution with glibc 2.34 or newer. The [installation guide](docs/operating/installation.md) describes each way.

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
|`server/`|`cargo test`|Unit tests and integration tests against PostgreSQL and Mailpit|
|`server/`|`cargo test --bins -- --skip integration::`|Unit tests only, without any service|
|`server/`|`cargo clippy --all-targets -- --deny warnings`|Lints|
|`server/`|`cargo fmt --check`|Formatting|
|`client/`|`npm test`|Unit tests|
|`client/`|`npm run test:integration`|Integration tests|
|`client/`|`npm run lint`|Lints|

`cargo test` finds PostgreSQL through `ADNT_DATABASE_URL` and Mailpit through `ADNT_SMTP_PORT` and `MAILPIT_UI_PORT`. It creates a separate database for each test and drops it when the test ends, so the user in `ADNT_DATABASE_URL` needs the right to create databases.

A test run that is killed leaves its databases behind. Drop them in the root of the checkout while no tests run there:

```sh
docker compose exec -T postgres psql -U adunata -tAc "select datname from pg_database where datname like 'adunata\_test\_%'" | xargs -rn1 docker compose exec -T postgres dropdb -U adunata --force
```

The end-to-end tests run against a release build in Nix:

```sh
nix build .#checks.x86_64-linux.e2e
```

On macOS this needs a Linux builder, as the [testing concept](docs/arc42/08-cross-cutting-concepts.md#testing) describes:

```sh
nix build .#vmChecks.e2e
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

CI runs the checks of the flake. Run the checks up to `integration` locally before each push. Replace `x86_64-linux` with your system, such as `aarch64-darwin`. The [testing concept](docs/arc42/08-cross-cutting-concepts.md#testing) lists every target.

```sh
nix build .#checks.x86_64-linux.integration
```

### Parallel worktrees

Several worktrees run side by side when each one has its own ports and its own services. The plugin [`dev/worktree_plugin.py`](dev/worktree_plugin.py) for [git-worktree-init](https://github.com/V1ncNet/git-worktree-init) sets them up for every new linked worktree and again on `git wt refresh`. It touches no service, so it works while Docker is stopped. Install git-worktree-init as its README describes, then enable the plugin once in the main checkout:

```sh
git config worktree-init.plugin dev/worktree_plugin.py
```

Each worktree gets a slot `n` with the port base `b = port_base + 10n`. Slot 0 belongs to the main checkout, which keeps the defaults of [Settings](#settings) and the Compose project `adunata`.

|Port|Service|
|-|-|
|`b`|Server|
|`b + 1`|Vite|
|`b + 2`|PostgreSQL|
|`b + 3`|SMTP of Mailpit|
|`b + 4`|Web interface of Mailpit|

For a new worktree, the plugin:

- writes the root `.env` with the Compose project `adunata-<b>` and its ports, `server/.env.local` with the addresses of server, database and Mailpit as well as `client/.env.local` with the ports of Vite, the server and Mailpit
- replaces only the lines of its own keys in these files and keeps every other line, so you can add your own settings there. It keeps the previous version of a file it changes as `<name>.bak`
- installs and builds the client, inside `nix develop` if Nix is installed, so the server compiles right away

Start the services of a worktree in its root, as in the main checkout:

```sh
docker compose up --detach --wait
```

The database `adunata` of a worktree starts empty. Migrations fill it on the first start of the server. To copy the database of the main checkout instead, run this in the root of the worktree before the server starts there for the first time:

```sh
docker compose -p adunata exec -T postgres pg_dump -U adunata --no-owner adunata | docker compose exec -T postgres psql -U adunata -q -v ON_ERROR_STOP=1 adunata
```

|Option|Default|Meaning|
|-|-|-|
|`port_base`|`18000`|Port from which the ports of the slots count. Change it per repository if another project uses the same range. Two clones of Adunata on one machine need different values, because the Compose project of a worktree is named after its ports.|

Set the option per repository with `git config worktree-init.port-base 19000`. After a change, an existing worktree keeps its Compose project under the old name. `git wt remove` then no longer finds it. Remove such a project with `docker compose -p adunata-<b> down --volumes`.

Remove a worktree with `git wt remove`. It runs `docker compose -p adunata-<b> down --volumes`, which deletes the containers and the database of the worktree. While Docker is stopped, it prints a warning with that command and removes the worktree anyway. A worktree removed with plain `git worktree remove` keeps its Compose project until the next sweep. `git wt prune`, `git wt add` and `git wt refresh` each run one.

git-worktree-init skips the worktrees of Claude Code under `.claude/worktrees/` by default. Such a worktree gets no root `.env`, so `docker compose` there uses the project `adunata` of the main checkout and its services. `docker compose down --volumes` in such a worktree deletes the database of the main checkout. Include these worktrees by clearing the pattern:

```sh
git config worktree-init.ignorePaths ""
```

Run `git wt refresh` in an existing worktree before you start its services the first time.

#### Switching from a shared database server

If a `pg_ctl` server with its data in `~/.local/share/adunata/pgdata` still runs on port 5432, dump its database and stop it before you start the services with Compose. The development shell provides no PostgreSQL, so `nix shell nixpkgs#postgresql_17` provides `pg_dump` and `pg_ctl`:

```sh
PGPASSWORD=adunata pg_dump --no-owner --host=/tmp --username=adunata adunata > adunata.sql
pg_ctl stop --pgdata="$HOME/.local/share/adunata/pgdata"
```

Start the services with Compose and restore the dump into them once:

```sh
docker compose up --detach --wait
docker compose exec -T postgres psql -U adunata -q adunata < adunata.sql
```

If a worktree still has `.envrc.local`, rebase it onto `main` before you run `git wt refresh` there. On an older branch the plugin keeps `.envrc.local` and asks for the rebase instead. If the server of the main checkout still holds databases `adunata_wt<n>`, drop them in the root of the main checkout:

```sh
docker compose exec -T postgres psql -U adunata -tAc "select datname from pg_database where datname ~ '^adunata_wt[0-9]+$'" | xargs -rn1 docker compose exec -T postgres dropdb -U adunata --force
```

## Contributing

[`AGENTS.md`](AGENTS.md) holds the rules for workflow, pull requests and code style. New features start as a change proposal in [`openspec/changes/`](openspec/changes/).

## License

Adunata is licensed under the [GNU Affero General Public License v3.0](LICENSE).
