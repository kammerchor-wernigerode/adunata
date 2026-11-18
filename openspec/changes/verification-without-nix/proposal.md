# Proposal

## Why

A maintainer without Nix can run only part of the automated verification. The contract check, the start of the end-to-end tests, the editorconfig check and the strict mode of the client lint exist only as snippets in `flake.nix`. Before each push the checks up to `integration` must pass, but without Nix there is no way to run all of them.

## What Changes

- Every check gets a documented command that runs without Nix, except some that stay with Nix. Where the flake runs the same check, it calls that command instead of a copy of its logic. The NixOS VM test, the linkage check of the release binary and the formatting of Nix files verify Nix artifacts. The memory measurement reads `/proc` and needs Linux. Two checks compare the versions that the paths without Nix pin, of Rust and of `editorconfig-checker`, with those of nixpkgs.
- `npm run lint` fails on warnings by itself, so a local run is as strict as CI.
- An npm package provides `editorconfig-checker`, and `npm run lint:editorconfig` checks the whole repository. The flake keeps the tool from nixpkgs, and both read the same `.editorconfig`.
- The contract check splits in two. A server test compares the OpenAPI document of the API with `client/openapi.json`. `npm run check:generated` regenerates the client code and compares it with the code it found, which it restores afterwards.
- The end-to-end tests leave the flake. Before the tests, Playwright starts the services of `compose.yaml` as a fresh Compose project on free ports and then the server, from source or from a given binary, with an environment of its own. `npm run e2e` is the whole run. The worktree plugin removes that project with its worktree.
- The CI workflow splits its job. The job `verify` runs the checks up to `integration` and hands the release binary to two jobs that run in parallel. The job `e2e` runs the end-to-end tests without Nix against that binary on Ubuntu. The job `nixos` runs the memory measurement and the NixOS VM test.
- The end-to-end tests give the same result on Linux and macOS, and WSL is expected to behave like Linux. On NixOS the browsers of Playwright need nix-ld.
- The release workflow relies on the end-to-end tests of CI on `main` and stops claiming to run them.
- **BREAKING** for development: `nix build .#checks.<system>.e2e` and `nix build .#vmChecks.e2e` go away. `npm run e2e` replaces them and needs Docker with Compose.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. The change touches the development environment, CI and documentation, and no requirement of Adunata changes.

## Impact

- `flake.nix`: `clientLint` and `contract` call the new commands. `endToEnd`, the target `e2e`, `vmChecks.e2e` and the derivation `openapi` go away.
- `.github/workflows/ci.yaml`: the jobs `verify`, `e2e` and `nixos` replace the job `checks`. `.github/workflows/release.yaml` renames its check step.
- `client/package.json`, `client/playwright.config.ts`, a global setup for the end-to-end tests, the fallbacks of the end-to-end tests and a script that compares the generated code. New dev dependency `editorconfig-checker`. The override of `playwright-core` goes away.
- `dev/worktree_plugin.py` stops writing the URLs of the end-to-end tests and removes their Compose project with the worktree.
- A new test in `server/src/api.rs` compares the OpenAPI document.
- `README.md`, `AGENTS.md`, `docs/arc42/` and a note in the status of ADR 0005. No new decision record, because no decision of this change is architecturally significant.
