# Tasks

## 1. Lint without Nix

- [x] 1.1 Change the script `lint` in `client/package.json` to `oxlint --deny-warnings` and let `clientLint` in `flake.nix` run `npm run lint` without extra flags. Verify that `npm run lint` fails on a temporary unused variable and that `nix build .#checks.<system>.lint` passes.
- [x] 1.2 Add the npm package `editorconfig-checker` as a dev dependency of the client and the script `lint:editorconfig` that sets `EC_VERSION` to the release in nixpkgs and checks the root of the repository. Name its license and its download on the first run in the commit. Verify that `npm run lint:editorconfig` passes and fails on a temporary line with trailing spaces.
- [x] 1.3 Update the table of tests and lints in `README.md` with `npm run lint` and `npm run lint:editorconfig`, and run each listed command once as written.

## 2. Contract check in two halves

- [x] 2.1 Add a test to the module `integration` of `server/src/api.rs` that reads the file named by `ADUNATA_OPENAPI_DOCUMENT` at run time, with the fallback `../client/openapi.json`, compares the text of the file with `to_pretty_json()` plus a newline and names the command that regenerates the file. Let `serverIntegrationTests` in `flake.nix` set the variable to `client/openapi.json` in the store. Verify that the test passes, fails after a temporary change in `client/openapi.json`, also of whitespace alone, and passes in `nix build .#checks.<system>.integration`.
- [x] 2.2 Add `client/scripts/check-generated.sh` and the script `check:generated`, which runs it with `sh`. It copies `src/api/generated` to a temporary directory, runs Orval, compares both and restores the copy through a trap on every exit. Let `contract` in `flake.nix` run the script and remove the derivation `openapi`. Verify that `npm run check:generated` passes, fails after a temporary edit in `src/api/generated` with the edit still present afterwards, and that `nix build .#checks.<system>.integration` passes.
- [x] 2.3 Update the section on the API contract in `README.md` with both halves and how to fix each failure, and run the documented commands once.

## 3. End-to-end tests outside the flake

- [x] 3.1 Spike whether host port `0` in `compose.yaml` gives free ports with Docker Compose and whether `docker compose port` reads them back. Record the result in design.md. Verify by starting the project once with all three port variables set to `0`.
- [x] 3.2 In one commit:
  - Add a global setup to `client/playwright.config.ts` that starts the project `adunata-e2e-<hash of the checkout>` from `compose.yaml` after a `down --volumes`, on the ports of 3.1, then starts the server from `ADUNATA_E2E_SERVER` or from a release build of the source with a cleared environment and every setting design.md lists, passes `ADUNATA_E2E_URL` and `ADUNATA_E2E_MAILPIT_URL` to the tests and runs `down --volumes` in its teardown.
  - Stop loading `client/.env.local` in `playwright.config.ts` and remove the fallbacks to ports 8080 and 8025 from the tests.
  - Replace the job `checks` in `.github/workflows/ci.yaml` with the jobs `verify`, `e2e` and `nixos` as design.md describes. `verify` saves the Nix cache and uploads the release binary only when `e2e` runs, with a retention of one day. `nixos` only restores the cache. `e2e` restores the executable bit, installs Chromium with its system dependencies, caches npm and the browsers and runs `npm run e2e` against the binary. New actions are pinned by commit SHA.
  - Remove `endToEnd`, the target `e2e` and `vmChecks.e2e` from `flake.nix`, and drop "including end-to-end" from the name of the check step in `.github/workflows/release.yaml`.
  - Make every end-to-end test independent of the system it runs on, with fonts as the known dependency, without weakening any assertion of axe, contrast or reflow.
  Verify that `npm run e2e` passes twice in a row next to running development services and that a member created by hand in the development database is still there afterwards, that a run with `ADNT_CONFIG_FILE` set in the shell still passes, that no project `adunata-e2e-*` remains afterwards, that a test fails when `ADUNATA_E2E_URL` is missing, that a run passes on macOS and in the job `e2e` on Ubuntu, that `nix flake check --no-build` passes and that only `verify` runs on a draft pull request.
- [x] 3.3 Remove `ADUNATA_E2E_URL` and `ADUNATA_E2E_MAILPIT_URL` from what `dev/worktree_plugin.py` writes, keep both in its list of cleared variables and let its teardown also run `docker compose -p adunata-e2e-<hash> down --volumes` with the name design.md defines. Verify that a new worktree gets no such keys in `client/.env.local`, that the plugin and the global setup compute the same project name for it, and that removing it after an aborted run removes its project.
- [x] 3.4 Remove the override of `playwright-core` from `client/package.json`. Verify that `npm ci` and `npm run e2e` pass.
- [x] 3.5 Update `README.md`: the section on end-to-end tests with Docker with Compose as the requirement, the installation of Chromium, `npm run e2e` how to end a server left by a killed run, the limit of one run per checkout and nix-ld on NixOS, the table of settings files at the line for `client/.env.local`, and the description of what the worktree plugin writes. Run the documented commands once as written.

## 4. Documentation

- [x] 4.1 Add one sentence to the status of `docs/decisions/0005-nix-flake-as-build-definition.md` that the end-to-end tests run outside the flake, with a link to the section Testing in `docs/arc42/08-cross-cutting-concepts.md`. Verify that the link resolves.
- [x] 4.2 Update `docs/arc42/04-solution-strategy.md` and the section Testing in `docs/arc42/08-cross-cutting-concepts.md` as design.md lists. Verify that every link resolves and that no text names a check that no longer exists.
- [x] 4.3 Update the section on pull requests in `AGENTS.md` and the checks before a push in `README.md` with the commands without Nix in the order of the lifecycle, as design.md fixes them, and with the jobs of CI. Verify that the commands run as written.

## 5. Integration

- [x] 5.1 Run every command from the README for lint, build, unit, integration, contract and end-to-end tests in a shell without the Nix development shell and with the tools the README lists for development without Nix. Verify that each passes.
- [x] 5.2 Mark the pull request ready for review and verify that `verify`, `e2e` and `nixos` pass in CI, that `e2e` and `nixos` run in parallel and how long `nixos` takes compared with the steps of today.
