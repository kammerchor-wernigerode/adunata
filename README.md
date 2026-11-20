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

Install [Nix](https://nixos.org/download/) and enable flakes. Install Docker or Podman with Compose as well. The development shell provides Rust, Node.js 26, pnpm, `cargo-deny` and `cargo-about`. It compiles Rust through sccache, which shares compiled dependencies between checkouts.

```sh
nix develop
```

With [direnv](https://direnv.net) the shell loads by itself whenever you enter the repository and keeps your own shell with its prompt and aliases. [nix-direnv](https://github.com/nix-community/nix-direnv) caches the shell, so entering takes no time after the first load. Allow the tracked `.envrc` once:

```sh
direnv allow
```

### Without Nix

Install these tools:

- [rustup](https://rustup.rs), which installs the version of Rust that [`server/rust-toolchain.toml`](server/rust-toolchain.toml) pins, with `clippy`, `rustfmt` and the sources of the standard library
- Node.js in the version that [`.node-version`](.node-version) names. fnm reads this file and picks the newest installed release of that major version
- [pnpm](https://pnpm.io/installation) in the version that `packageManager` in [`client/package.json`](client/package.json) names
- Docker or Podman with Compose

### RustRover

The repository tracks the project files of [RustRover](https://www.jetbrains.com/rust/) in `.idea/`. They exclude build output and linked worktrees from indexing, require the plugin NixIDEA and map the JSON schema of `rust-toolchain.toml`. Open the root of the checkout, not `server/`.

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
|`client/`|Vite|none, defaults in the configs|`.env.local`|

The server reads its files only in a debug build and only in its working directory, so run it and its tests in `server/`. The [configuration reference](docs/operating/configuration.md) explains each `ADNT_` variable. Without an untracked file, PostgreSQL listens on port 5432, Mailpit on 1025 and 8025, the server on 8080 and Vite on 5173.

### Run Adunata

Build the client once, then start the server:

```sh
cd client
pnpm install --frozen-lockfile
pnpm run build
cd ../server
cargo run -- serve --dev
```

The server listens on <http://localhost:8080> and applies database migrations on start. The option `--dev` starts [development mode](docs/operating/installation.md#development-mode). On a fresh database it creates the organizer `organizer@example.org`, and before it accepts requests it logs a sign-in link for the oldest active organizer. Open that link to sign in. Every link you request on the sign-in page also appears in the log, so you need Mailpit only for other mail.

An agent gets a session without a browser. It starts the server with its log in a file, waits until the server accepts requests, takes the link from the log and sends its secret to the API:

```sh
log=$(mktemp)
cargo run -- serve --dev > "$log" &
server=$!
until grep -q '"accepting requests"' "$log"; do
  kill -0 "$server" 2>/dev/null || { cat "$log"; break; }
  sleep 1
done
jar=$(mktemp)
link=$(jq -r 'select(.fields.message == "sign-in link for development") | .fields.link' "$log" | tail -n 1)
curl -s -c "$jar" -X POST -H "Authorization: Bearer ${link#*#}" "${link%%/sign-in#*}/api/sessions"
curl -s -b "$jar" "${link%%/sign-in#*}/api/session"
```

To work on the client, keep the server running and start the Vite development server in `client/`:

```sh
pnpm run dev
```

It serves the client on <http://localhost:5173> and reloads on every change. It forwards requests to `/api` to the server on port 8080. `ADUNATA_VITE_PORT` in `client/.env.local` changes the port of Vite. The forwarding follows `ADNT_HTTP_LISTEN_ADDRESS` there.

### Tests and lints

|Directory|Command|Runs|
|-|-|-|
|`server/`|`cargo test`|Unit tests and integration tests against PostgreSQL and Mailpit|
|`server/`|`cargo test --bins -- --skip integration::`|Unit tests only, without any service|
|`server/`|`cargo clippy --all-targets -- --deny warnings`|Lints|
|`server/`|`cargo fmt --check`|Formatting|
|`server/`|`cargo deny check bans licenses sources`|Licenses of the crates against the list in `server/deny.toml`, version requirements without wildcards and crates.io as the only source. Needs `cargo-deny`, which the development shell provides. Without Nix, `cargo install --locked cargo-deny@0.19.6` installs it in the version of nixpkgs|
|`server/`|`cargo about generate --frozen --fail about.hbs >> ../client/dist/licenses.txt`|Appends the notices of the crates to the notices of the client after `pnpm run build`, so the server embeds both. Needs `cargo-about`, which the development shell provides. Without Nix, `cargo install --locked cargo-about@0.9.0 --features cli` installs it in the version of nixpkgs|
|`client/`|`pnpm test`|Unit tests|
|`client/`|`pnpm run test:integration`|Integration tests|
|`client/`|`pnpm run lint`|Lints, failing on warnings|
|`client/`|`pnpm run lint:editorconfig`|Rules of `.editorconfig` in the whole repository. Downloads `editorconfig-checker` on its first run|
|`client/`|`pnpm run lint:licenses`|Licenses of the installed packages against the list in `server/deny.toml` and the exceptions in `client/license-exceptions.json`|

`cargo test` finds PostgreSQL through `ADNT_DATABASE_URL` and Mailpit through `ADNT_SMTP_PORT` and `MAILPIT_UI_PORT`. It creates a separate database for each test and drops it when the test ends, so the user in `ADNT_DATABASE_URL` needs the right to create databases.

A test run that is killed leaves its databases behind. Drop them in the root of the checkout while no tests run there:

```sh
docker compose exec -T postgres psql -U adunata -tAc "select datname from pg_database where datname like 'adunata\_test\_%'" | xargs -rn1 docker compose exec -T postgres dropdb -U adunata --force
```

### End-to-end tests

The end-to-end tests drive Chromium against a release build of Adunata. They need Docker with Compose. Install Chromium once, then run them in `client/`:

```sh
cd client
pnpm exec playwright install chromium
pnpm run e2e
```

Before the tests, Playwright builds the client and runs `cargo build --release` in `server/`. With `ADUNATA_E2E_SERVER` set to a binary, it starts that binary instead. It starts PostgreSQL and Mailpit from `compose.yaml` as a fresh Compose project `adunata-e2e-<hash>` on free ports, then the server with only the settings of the tests. No settings file and no variable of your shell reach the server. After the tests it removes the project with its volume. The development services and their data stay untouched.

The server writes its output to `client/test-results/server.log`, and the services write theirs to `client/test-results/services.log`.

One run per checkout is the limit, because a second run removes the services of the first. A run that is killed leaves its server running on a free port. Find and end it:

```sh
pgrep -fl 'adunata serve'
kill <pid>
```

On NixOS the browsers of Playwright start only with [nix-ld](https://github.com/nix-community/nix-ld).

### API contract

The client calls the server through code that [Orval](https://orval.dev) generates from the OpenAPI document of the server. After a change to the API, regenerate both and commit them:

```sh
cd server
cargo run -- openapi > ../client/openapi.json
cd ../client
pnpm run generate
```

The contract check has two halves:

|Directory|Command|Fails when|Fix|
|-|-|-|-|
|`server/`|`cargo test --bins -- api::integration::the_client_holds_the_current_document`|`client/openapi.json` differs from the document of the server, even in whitespace|`cargo run -- openapi > ../client/openapi.json` in `server/`|
|`client/`|`pnpm run check:generated`|The code in `src/api/generated` differs from what Orval generates from `openapi.json`|`pnpm run generate` in `client/`|

`cargo test` runs the first half among the integration tests. `pnpm run check:generated` regenerates the code in place and restores the code it found afterwards, also when it fails.

### UI components

The components in `client/src/components/ui/` are copies from shadcn. The client depends on the package `shadcn`, whose stylesheet it imports. Its command line tool adds a component together with the packages it needs:

```sh
cd client
pnpm exec shadcn add <component>
```

Commit the component together with those packages.

### Third-party notices

`pnpm run build` writes `client/dist/licenses.txt` with the license texts of every package in the client. Most texts come from the bundle. [`client/notices.json`](client/notices.json) adds what the bundle does not show. Each copy of third-party code needs an entry under `copied` with its path, source, version, license and a text under `client/licenses/`, and copies of components go into `client/src/components/ui/`, which already has one. A package that reaches the browser only through the stylesheet goes under `stylesheetPackages`, and a package that ships no license file gets a text under `licenseTexts`. The build fails when a bundled package has no license text or an entry names a path that does not exist.

### Checks before a push

Run the checks up to `integration` locally before each push. CI runs them in the job `build` and runs the end-to-end tests, the image and the NixOS VM test once a pull request is ready for review. The [testing concept](docs/arc42/08-cross-cutting-concepts.md#testing) lists every target and job.

With Nix, one command runs them. Replace `x86_64-linux` with your system, such as `aarch64-darwin`:

```sh
nix build .#checks.x86_64-linux.integration
```

Without Nix, run the same checks in the order of the lifecycle, with the services running:

```sh
cd client
pnpm install --frozen-lockfile
pnpm run lint
pnpm run lint:editorconfig
pnpm run lint:licenses
cd ../server
cargo fmt --check
cargo clippy --all-targets -- --deny warnings
cargo deny check bans licenses sources
cd ../client
pnpm run build
cd ../server
cargo about generate --frozen --fail about.hbs >> ../client/dist/licenses.txt
cargo build
cargo test --bins -- --skip integration::
cd ../client
pnpm test
cd ../server
cargo test --bins -- integration::
cargo test --test startup
cd ../client
pnpm run test:integration
pnpm run check:generated
```

CI also checks the formatting of Nix files, the linkage of the release binary, the GitHub workflows with zizmor and whether the pinned versions of Rust, pnpm, `editorconfig-checker`, `cargo-deny` and `cargo-about` match nixpkgs, which needs Nix. The end-to-end tests stay optional before a push.

### Dependency updates

Dependabot opens pull requests once a week for the crates in `server/`, the packages in `client/`, the inputs in `flake.lock` and the GitHub actions, as [`.github/dependabot.yml`](.github/dependabot.yml) configures. Minor and patch updates come grouped. Each update waits seven days after its release. Dependabot opens its pull requests ready for review, so they run every CI job.

An update of `flake.lock` can bring new versions of Rust, pnpm, `editorconfig-checker`, `cargo-deny` or `cargo-about`. The lint checks that compare the pins for work without Nix then fail and name the version of nixpkgs. A new pnpm also changes the hash of the client dependencies, and the build names the new one. Push one commit to the Dependabot branch that sets `channel` in `server/rust-toolchain.toml`, `packageManager` and `EC_VERSION` in `client/package.json`, the hash in `client/pnpm-deps.hash` and the versions of `cargo-deny` and `cargo-about` under [Tests and lints](#tests-and-lints) to those versions, together with fixes for new Clippy lints. Dependabot no longer rebases a branch with commits of someone else. If `main` moves before the merge, rebase the branch by hand or comment `@dependabot recreate`, which drops your commit. Dependabot moves only the revisions in `flake.lock`. Change the branch of `nixpkgs` and the tag of `crane` in `flake.nix` by hand.

[`client/pnpm-workspace.yaml`](client/pnpm-workspace.yaml) lets pnpm install only releases older than seven days, also those already in `pnpm-lock.yaml`. Only the packages under `allowBuilds` may run build scripts, and pnpm fails on a new package with a build script until it gets an entry there. To take a security fix younger than seven days, add its exact version to `minimumReleaseAgeExclude`, such as `picomatch@4.0.8`, then update it. `pnpm update` also moves a package that comes in through another one, as long as the fixed version fits the range that the other package requires. Remove the entry once the release is older than seven days. A version update whose fetch fails on a young release waits instead. Comment `@dependabot recreate` once the release is old enough.

```sh
cd client
pnpm update <package>
```

If the fix lies outside that range, pin it under `overrides` in `client/pnpm-workspace.yaml`.

The Nix build fetches the client dependencies once for `pnpm-lock.yaml` and checks them against the hash in [`client/pnpm-deps.hash`](client/pnpm-deps.hash). Every change of `pnpm-lock.yaml` or `pnpm-workspace.yaml` makes the build fetch the dependencies anew, which also checks the release age. If the fetched packages differ, the build fails with `got: sha256-…`. Write that value to the file. Without Nix, the job `build` of CI shows the same error. On pull requests of Dependabot, the workflow [`client-dependencies`](.github/workflows/client-dependencies.yaml) writes the hash and pushes it with the token of the GitHub App `adunata-client-dependencies`, so the push starts CI on the pull request. `[dependabot skip]` in its message lets Dependabot keep rebasing the branch.

The workflow [`advisories`](.github/workflows/advisories.yaml) checks both lock files against the advisory databases every day and on pull requests that change them. It fails on any advisory of a crate and on an advisory of a client package of severity high or critical. If such an advisory has no fix yet, add it to `client/audit-exceptions.json` with its GHSA ID, the reason and an expiry date. The run fails once the date has passed and, outside pull requests, once the exception matches no advisory. GitHub notifies only the person who last changed the schedule of a failed scheduled run, so check the workflow in the Actions tab. For a crate, add `{ id, reason }` to `ignore` under `[advisories]` in `server/deny.toml`.

The repository expects these settings, which no file holds:

|Setting|Where|
|-|-|
|Dependabot alerts and security updates|Settings, Advanced Security|
|Secret Protection with push protection|Settings, Advanced Security|
|Ruleset for `main` that requires the check `Verdict`, with a bypass for admins|Settings, Rules|
|Ruleset for tags `v*` that restricts creation, update and deletion, with a bypass for admins|Settings, Rules|
|GitHub App `adunata-client-dependencies` with the repository permission Contents read and write, installed on this repository only|Settings of the organization, Developer settings, GitHub Apps|
|Dependabot secrets `CLIENT_DEPENDENCIES_APP_CLIENT_ID` and `CLIENT_DEPENDENCIES_APP_PRIVATE_KEY` with the client ID and a private key of that App|Settings, Secrets and variables, Dependabot|

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

- writes the root `.env` with the Compose project `adunata-<b>` and its ports, `server/.env.local` with the addresses of server, database and Mailpit as well as `client/.env.local` with the port of Vite and the address of the server
- replaces only the lines of its own keys in these files and keeps every other line, so you can add your own settings there. It keeps the previous version of a file it changes as `<name>.bak`
- installs and builds the client, inside `nix develop` if Nix is installed, so the server compiles right away

All checkouts share the global store of pnpm. `node_modules` of a worktree holds only links into it, so a new worktree installs in seconds and takes almost no disk space. The store keeps every version that any checkout installed. `pnpm store prune` removes those that no checkout uses any more.

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

Remove a worktree with `git wt remove`. It runs `docker compose -p adunata-<b> down --volumes`, which deletes the containers and the database of the worktree. It also removes the Compose project of the end-to-end tests that a killed run left behind. While Docker is stopped, it prints a warning with that command and removes the worktree anyway. A worktree removed with plain `git worktree remove` keeps its Compose project until the next sweep. `git wt prune`, `git wt add` and `git wt refresh` each run one.

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
