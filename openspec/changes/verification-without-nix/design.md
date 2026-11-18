# Design

## Context

See proposal.md for the motivation.

The flake groups its checks into the targets `lint`, `build`, `test`, `integration`, `e2e`, `memory` and `nixos`. crane runs the Rust checks with its own artifacts. The client checks run `npm` in a derivation that holds only `client/` and `server/locales/`. Four checks keep logic that exists nowhere else:

|Check|Logic in `flake.nix`|
|-|-|
|`clientLint`|`npm run lint -- --deny-warnings`, the flag exists only here|
|`editorconfig`|`editorconfig-checker` from nixpkgs over the whole source|
|`contract`|compares `client/openapi.json` with `adunata openapi`, runs Orval and compares the generated code|
|`endToEnd`|`startInstance` starts PostgreSQL, Mailpit and the binary in the sandbox, then Playwright runs with browsers from nixpkgs|

The end-to-end tests expect an empty database with one organizer from the bootstrap settings and an empty Mailpit. `invitation.e2e.ts` invites the members not yet invited and reads the newest mail. They must not run against the services of a worktree in use.

The Nix sandbox has no Docker. `client/package.json` pins `playwright-core` to the version of the browsers in nixpkgs. The flake sets `FONTCONFIG_FILE` to DejaVu for the end-to-end tests.

The CI workflow has one job `checks` with one step per target. `e2e`, `memory` and `nixos` run only on pull requests ready for review and on pushes to `main`.

Playwright starts the servers of `webServer` before it runs the global setup. `client/playwright.config.ts` loads `client/.env.local`, where the worktree plugin writes `ADUNATA_E2E_URL` and `ADUNATA_E2E_MAILPIT_URL` for the development services. The tests fall back to ports 8080 and 8025.

`compose.yaml` takes every host port from a variable and keeps the data of PostgreSQL in the named volume `postgres`. `.github/workflows/release.yaml` runs `nix flake check` in a step named "Every check, including end-to-end, memory and the NixOS VM test" before it publishes.

## Goals / Non-Goals

**Goals:**

- Each check in the scope of the proposal has one place that holds its logic, which the flake and a maintainer without Nix both use. `editorconfig` is the exception, where only the tool is shared through `.editorconfig`.
- The end-to-end tests run the same way on a laptop and in CI.

**Non-Goals:**

- One command that runs every check without Nix. The README lists the commands in the order of the lifecycle.
- Rust checks that crane runs through a script. crane keeps calling `cargo`, and the flags of Clippy stay in the README and in the flake.
- Running the memory measurement or the Rust integration tests outside Nix in CI.
- Starting services from the Rust integration tests. They run in the Nix sandbox in CI, and locally they keep the Compose services of the worktree.

## Decisions

### Lint commands carry their own strictness

`npm run lint` becomes `oxlint --deny-warnings`. The flake runs `npm run lint` without extra flags.

`editorconfig-checker` comes from the npm package of the same name as a dev dependency of the client, MIT licensed. The wrapper downloads the release of the tool that `EC_VERSION` names and otherwise the newest one. `npm run lint:editorconfig` sets `EC_VERSION` to the release that nixpkgs ships, today `v3.6.1`, and runs it in the root of the repository. The flake keeps `editorconfig-checker` from nixpkgs, because its check needs the whole source and the client derivation holds only `client/`. Both read `.editorconfig` and run the same release of the tool.

Alternatives considered: a separate manifest for tools in the root, which adds a second Node project for one tool. Documentation only, which leaves installation and version to each maintainer.

Not architecturally significant, because it changes only the tooling of development.

### Contract check in two halves

The server half becomes a test in the module `integration` of `server/src/api.rs`, because it reads a file. It compares the text of the file with `to_pretty_json()` of `api::document()` plus a newline, which is exactly what the command writes. On a difference it names `cargo run -- openapi > ../client/openapi.json`. It reads the path from `ADUNATA_OPENAPI_DOCUMENT` at run time and falls back to `../client/openapi.json` relative to the crate. The flake sets the variable to the file in the store, because the source of the server derivation holds only `server/`. Reading at run time keeps the shared test artifacts valid. The derivation `openapi` goes away.

The client half becomes `npm run check:generated`, which runs `sh scripts/check-generated.sh`. The script copies `src/api/generated` to a temporary directory, runs Orval in place and compares both. A trap restores the copy on every exit, so a difference, a failing Orval or an interruption leaves the working tree as it was. On a difference it names `npm run generate`. The flake check `contract` runs the same script.

Alternatives considered: an npm script that runs `cargo run -- openapi`, which builds the whole server from the client and reaches across directories. Generating into a separate directory, which needs a second configuration of Orval.

Not architecturally significant, because it moves a check without changing what it verifies.

### End-to-end tests outside the flake with Compose

A global setup for Playwright starts the services of `compose.yaml` as a Compose project of its own. The project name is `adunata-e2e-` followed by the first 12 hexadecimal characters of SHA-256 over the root of the checkout as `git rev-parse --show-toplevel` prints it, without resolving symlinks. Parallel worktrees never share one. The worktree plugin computes the same name for its teardown. One run per checkout is the limit: a second run in the same checkout removes the services of the first. Before the start the setup runs `docker compose -p <project> down --volumes` for that project, which removes what a killed run left behind and gives a fresh, empty volume. It sets `POSTGRES_PORT`, `MAILPIT_SMTP_PORT` and `MAILPIT_UI_PORT` to `0`, so the engine picks free host ports, starts with `up --detach --wait` and reads the ports back with `docker compose port`. The images and their versions come from `compose.yaml`, their one home. Compose scopes containers and volumes by project and `-p` wins over `COMPOSE_PROJECT_NAME` from the root `.env`, so neither the setup nor its teardown touches the development services or the data of manual tests. Its teardown runs `down --volumes` again. A run that dies without its teardown leaves the project until the next run in the same checkout. When the worktree plugin removes a worktree, it also removes that project, as it does with the Compose project of the development services.

The setup then starts the server with a cleared environment, as `server/tests/startup.rs` starts the binary, and passes every setting explicitly: the database URL and SMTP port of the project, a free port for HTTP and the public URL that matches it, SMTP host `127.0.0.1` without encryption, the sender `Adunata <adunata@example.org>`, the time zone `Europe/Berlin` and the organizer `organizer@example.org` with the first name `Ada` and the last name `Lovelace` for the bootstrap. The server writes its output to a file in `test-results`. No setting of the worktree or the shell, such as `ADNT_CONFIG_FILE`, reaches it. It waits until the server answers and hands its URL and the URL of Mailpit to the tests through `ADUNATA_E2E_URL` and `ADUNATA_E2E_MAILPIT_URL`. `webServer` cannot do this, because Playwright starts it before the setup knows the ports.

`playwright.config.ts` stops loading `client/.env.local`. `baseURL` comes from `ADUNATA_E2E_URL` without a fallback, and the helper for Mailpit throws when `ADUNATA_E2E_MAILPIT_URL` is missing. Playwright loads the config before the global setup sets the variables and again in its workers, so the check cannot sit in the config itself. A missing URL then fails the tests instead of reaching the development services. The worktree plugin stops writing `ADUNATA_E2E_URL` and `ADUNATA_E2E_MAILPIT_URL`. Existing worktrees keep both lines, which nothing reads any more. Both names stay in the variables that the plugin clears.

We considered Ryuk, the reaper of Testcontainers, to remove the project after a killed run. It needs a privileged container with access to the socket of the engine for a case that the next run and the worktree plugin already cover.

The server is the binary in `ADUNATA_E2E_SERVER` if set. Otherwise the setup builds the client and runs `cargo build --release` in `server/`, then starts that binary. A release build reads no settings file either.

A spike at the start of the work confirmed that host port `0` works with Docker Compose 5.7 on Docker 29.9. With all three port variables set to `0`, the engine bound PostgreSQL and both ports of Mailpit to free ports on `127.0.0.1`, and `docker compose port` printed each of them.

Without Nix the run is:

```sh
cd client
npx playwright install chromium
npm run e2e
```

It needs Docker with Compose. Podman is no goal of the end-to-end setup.

The flake drops `endToEnd`, the target `e2e` and `vmChecks.e2e`. Browsers come from Playwright, so the override of `playwright-core` goes away. `startInstance` stays for the memory measurement.

Alternatives considered: Testcontainers, which adds a large tree of dependencies with install scripts to every client derivation of the flake. A Compose profile with fixed ports, which needs a manual start before each run and four more ports per slot of the worktree plugin. Keeping the tests in the flake, which leaves maintainers without Nix with a path that CI never runs.

Not architecturally significant, because it changes where tests run and not what Adunata is or how it is installed.

### CI in three jobs

|Job|Runs|Needs|
|-|-|-|
|`verify`|evaluation of the flake, `lint`, `build`, `test`, `integration`, then builds `packages.x86_64-linux.release` and uploads the binary as an artifact|nothing|
|`e2e`|downloads the artifact, restores its executable bit, installs Node 24 and Chromium with its system dependencies, runs `npm run e2e` with `ADUNATA_E2E_SERVER` pointing at the binary|`verify`|
|`nixos`|the checks `memoryMeasurement` and `nixosVmTest`, which hold the memory measurement and the NixOS VM test without the targets up to `integration`, with the KVM setup|`verify`|

`e2e` and `nixos` run only on pull requests ready for review and on pushes to `main`, as their steps do today. `verify` uploads the binary only then. The artifact keeps one day. New actions are pinned by commit SHA like the existing ones. `e2e` caches npm by `package-lock.json` and the browsers by the version of Playwright. The teardown of the global setup writes `docker compose logs` of the project to `test-results/services.log` before it removes the project. On a failure the job prints that file and the output of the server and uploads `test-results`. `verify` saves the Nix cache, and `nixos` only restores it, so both use one key without a race.

The end-to-end job runs the release binary on Ubuntu, a distribution without Nix. That covers the release archive outside a simulated distribution as well.

Alternatives considered: a separate workflow started by `workflow_run`, which runs in the context of the default branch, does not show as a check of the pull request in the usual way and needs the run id for the artifact. One job with `needs` after all checks, which runs the end-to-end tests last instead of in parallel. A job `e2e` that builds the binary itself, which builds it twice when the cache misses.

The release workflow keeps `nix flake check` and runs no end-to-end tests of its own. A release comes from a commit on `main` whose CI ran them. The step loses the words "including end-to-end" from its name.

Not architecturally significant on its own. It follows from the previous decision.

### Checks before a push without Nix

Before a push, a maintainer without Nix runs the commands of `lint` without the formatting of Nix files, `build`, `test` and `integration`, as the README lists them. CI runs the formatting of Nix files and the linkage check of the release binary. The end-to-end tests stay optional before a push, as they are today with Nix.

Not architecturally significant, because it is a rule of the workflow.

### Same result on every system

The end-to-end tests take colors from computed styles, not from pixels, and wait for transitions to end. Fonts are the known dependency on the system: the flake pins DejaVu today, and the reflow check at 200 % depends on the widths of text. The client ships its own fonts, so the tests must render with them and not with fonts of the system. A test that still depends on the system, such as on a font, a time zone or the speed of the machine, changes to remove that dependency. No assertion of axe, contrast or reflow becomes weaker to reach this. A run on macOS and the job `e2e` verify it.

Not architecturally significant, because it is a rule for tests.

### Affected arc42 sections

- 04 Solution strategy: the row for build and deployment says that the flake builds every artifact and runs every check except the end-to-end tests.
- 08 Cross-cutting concepts, section Testing:
  - The sentence that every check runs as a check of the flake names the end-to-end tests as the exception.
  - The sentence that CI runs each target in a step of its own describes the three jobs.
  - The table of targets loses `e2e`.
  - The row for contract tests in the table of levels names the test in `server/src/api.rs` and `npm run check:generated`.
  - The row for end-to-end tests drops the note about Linux.
  - A paragraph says where the end-to-end tests run, against which binary and how they get their services from `compose.yaml`. It names what follows: the tests need Docker with Compose, the browsers come from Playwright and need nix-ld on NixOS, Node and the versions of PostgreSQL and Mailpit come from the runner and from `compose.yaml` instead of nixpkgs, a green `nix flake check` no longer covers them, and a release relies on their run on `main`.
  - The paragraph on `vmChecks` names only `nixos`.

### Decision records

The change drafts no record. Where the end-to-end tests run and how CI splits its jobs changes neither the structure of Adunata nor its interfaces, dependencies or quality goals, and no operator or running instance notices it. The deployment that ADR 0005 decides, one flake that builds every way of installing from the same source, stays as it is.

ADR 0005 also says that CI runs the checks of the flake and nothing else. Its status gains one sentence that the end-to-end tests run outside the flake and that section 08 of arc42 describes where.

## Risks / Trade-offs

- [The npm package `editorconfig-checker` downloads its tool from GitHub on the first run, outside the hashes of `package-lock.json`] → `EC_VERSION` pins the release to the one in nixpkgs. The commit that adds the wrapper names the download next to the license. The flake does not use it, so CI does not depend on that download.
- [nixpkgs updates `editorconfig-checker` and the pinned `EC_VERSION` falls behind] → A difference shows as a failing check in one place, and `EC_VERSION` follows nixpkgs when it does.
- [Browsers and images download from the network in CI] → The cache keeps the browsers per version of Playwright. The images are small and come from the registries that `compose.yaml` already uses.
- [`cargo build --release` in the global setup compiles for minutes on the first run] → The timeout of the global setup allows for it, and later runs reuse the build.
- [A killed run leaves its server process running] → It holds only a free port and loses its database when the next run removes the project. The README names how to find and end it.
- [A release from a commit that CI never ran skips the end-to-end tests] → Releases come from `main`. A tag or a manual run on another commit is a deliberate exception.
- [The commands before a push without Nix and the targets of the flake list the same checks in two places, and CI never runs the README list] → Task 5.1 compares them once, and a change to a target updates both.
- [On NixOS the browsers of Playwright do not start without nix-ld] → The README and section 08 of arc42 name it. Development happens on macOS, other Linux distributions and WSL.
- [Only `verify` saves the Nix cache, so `nixos` builds the server and the closure of the VM test again on every run] → The first run of `nixos` took 7 to 8 minutes, most of it for rebuilding the targets up to `integration` that `verify` had already run. `nixos` therefore builds `memoryMeasurement` and `nixosVmTest`, which leave those targets out.
