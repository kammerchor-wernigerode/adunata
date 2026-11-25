# Contributing

[`AGENTS.md`](AGENTS.md) holds the rules for workflow, pull requests and code style. New features start as a change proposal in [`openspec/changes/`](openspec/changes/). The [README](README.md#development) describes how to set up the tools and start Adunata.

## Settings

Compose, the server and the client each read their settings from files in their own directory. A `.env.local` overrides `.env`. A variable set in the environment wins over both files. No shell exports a setting, so a setting in the environment comes only from you.

|Directory|Read by|Tracked|Untracked|
|-|-|-|-|
|root|Compose|none, defaults in `compose.yaml`|`.env` with `COMPOSE_PROJECT_NAME`, `POSTGRES_PORT`, `MAILPIT_SMTP_PORT` and `MAILPIT_UI_PORT`|
|`server/`|server and its tests|`.env` with the settings for development|`.env.local`|
|`client/`|Vite|none, defaults in the configs|`.env.local`|

The server reads its files only in a debug build and only in its working directory, so run it and its tests in `server/`. The [configuration reference](docs/operating/configuration.md) explains each `ADNT_` variable. Without an untracked file, PostgreSQL listens on port 5432, Mailpit on 1025 and 8025, the server on 8080 and Vite on 5173.

## Sign in without a browser

An agent gets a session without a browser. It starts the server with its log in a file, waits until the server accepts requests, takes the link from the log and sends its secret to the API:

```sh
log=$(mktemp)
cargo run -- serve --local > "$log" &
server=$!
until grep -q '"accepting requests"' "$log"; do
  kill -0 "$server" 2>/dev/null || { cat "$log"; break; }
  sleep 1
done
jar=$(mktemp)
link=$(jq -r 'select(.fields.message == "sign-in link for local mode") | .fields.link' "$log" | tail -n 1)
curl -s -c "$jar" -X POST -H "Authorization: Bearer ${link#*#}" "${link%%/sign-in#*}/api/sessions"
curl -s -b "$jar" "${link%%/sign-in#*}/api/session"
```

## Tests and lints

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

The [testing concept](docs/arc42/08-cross-cutting-concepts.md#testing) defines which test is a unit test. The target `test` of the flake skips the integration tests and starts neither PostgreSQL nor Mailpit, so a test that needs them outside a module named `integration` fails there. Nothing enforces the rule for sockets, files and the router.

A test run that is killed leaves its databases behind. Drop them in the root of the checkout while no tests run there:

```sh
docker compose exec -T postgres psql -U adunata -tAc "select datname from pg_database where datname like 'adunata\_test\_%'" | xargs -rn1 docker compose exec -T postgres dropdb -U adunata --force
```

## End-to-end tests

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

The end-to-end tests run outside the flake, also in the CI job `e2e` on Ubuntu without Nix. Node comes from the runner, and the versions of PostgreSQL and Mailpit come from `compose.yaml` instead of nixpkgs. A green `nix flake check` does not cover the end-to-end tests, and a release relies on their run on `main`. The tests take colors from computed styles and measure reflow only after the fonts of the client have loaded, so they give the same result on Linux and macOS.

## API contract

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

## UI components

The components in `client/src/components/ui/` are copies from shadcn. The client depends on the package `shadcn`, whose stylesheet it imports. Its command line tool adds a component together with the packages it needs:

```sh
cd client
pnpm exec shadcn add <component>
```

Commit the component together with those packages.

## Third-party notices

`pnpm run build` writes `client/dist/licenses.txt` with the license texts of every package in the client. Most texts come from the bundle. [`client/notices.json`](client/notices.json) adds what the bundle does not show. Each copy of third-party code needs an entry under `copied` with its path, source, version, license and a text under `client/licenses/`, and copies of components go into `client/src/components/ui/`, which already has one. A package that reaches the browser only through the stylesheet goes under `stylesheetPackages`, and a package that ships no license file gets a text under `licenseTexts`. The build fails when a bundled package has no license text or an entry names a path that does not exist.

## Checks before a push

Run the checks up to `integration` locally before each push. CI runs them in the job `build` and runs the end-to-end tests, the image and the NixOS VM test once a pull request is ready for review. The [testing concept](docs/arc42/08-cross-cutting-concepts.md#testing) says what each level of tests proves. [Continuous integration](docs/maintenance/ci.md) describes the jobs.

One script runs them, in the root of the checkout:

```sh
factory/checks.sh
```

With Nix it builds `.#checks.<system>.integration` for the local system. Without Nix it runs the checks of that target that need no Nix, one by one in the order of the lifecycle, and needs the services running. It skips the checks that need Nix, which the next paragraph lists, so CI may still fail on one of them. The [software factory](AGENTS.md#software-factory) runs the same script on each of its branches.

CI also checks the formatting of Nix files, the linkage of the release binary, the closure of the image, the GitHub workflows with zizmor, whether `server/Cargo.toml`, `server/Cargo.lock` and `client/package.json` name the same version and whether the pinned versions of Rust, pnpm, `editorconfig-checker`, `cargo-deny` and `cargo-about` match nixpkgs, which needs Nix. The end-to-end tests stay optional before a push.

### Targets of the flake

The flake groups its checks into targets that build on each other like the phases of a build lifecycle. Each target contains the one before it, so `nix build .#checks.<system>.integration` runs lint, build, unit and integration tests together.

|Target|Contains|Builds on|
|-|-|-|
|`lint`|Formatting of Rust, Nix and every file per `.editorconfig`, Clippy, oxlint, the licenses of crates and installed client packages against the approved list in `server/deny.toml` and the exceptions for client packages in `client/license-exceptions.json`, the licenses of the tools from nixpkgs and of the packages in the OCI image against that list and a few licenses allowed only for them, crates only from crates.io and with a version requirement, zizmor over the GitHub workflows and the Dependabot configuration, the tests of the version comparison of the release workflow, whether `server/Cargo.toml`, `server/Cargo.lock` and `client/package.json` name the same version without build metadata, whether `server/about.toml` accepts the same licenses for the notices, whether the notices of the Rust standard library come from the version that builds the server and whether the versions of Rust, pnpm, `editorconfig-checker`, `cargo-deny` and `cargo-about` pinned for work without Nix match nixpkgs|nothing|
|`build`|The server with the embedded client and its license notices and, on Linux, whether the packages from nixpkgs in the OCI image match the list of packages whose notices the image carries|`lint`|
|`test`|Unit tests of server and client|`build`|
|`integration`|Integration tests of server and client, startup tests of the binary and the contract|`test`|
|`nixos`|The NixOS VM test, Linux only|`integration`|

Each check inside a target is also a check of its own, such as `clippy` or `serverIntegrationTests`, so a failing one runs again alone.

On macOS with a Linux builder, such as the `linux-builder` of nix-darwin, `nix build .#vmChecks.nixos` runs the target `nixos`. The builder builds its Linux artifacts for the same architecture. QEMU starts the virtual machines on the Mac with HVF. `nix flake check` leaves it out, so it needs no Linux builder.

## Parallel worktrees

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
