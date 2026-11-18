# Proposal

## Why

Nothing in the repository updates dependencies or reports a known vulnerability in them. `server/Cargo.lock` holds `lettre` 0.11.23 with six RustSEC advisories. Nothing reported them. `client/package-lock.json` has two npm advisories that affect 29 packages. A newly published package version runs its install scripts on `npm install`, on `npm update`, through `npx` and in every pull request that a bot opens with it. No check looks at the workflows in `.github/workflows` for weaknesses such as credentials that persist after checkout.

## Prerequisite

The change `license-compliance` must be merged first. A separate fix on `main` must have moved `lettre` to 0.11.24 in `server/Cargo.toml` and `server/Cargo.lock`. That fix needs no proposal and does not wait for this change, because the advisories affect running instances. It adds `server/deny.toml` with the list of allowed licenses and the check `rustLicenses`, which runs crane's `cargoDeny` with `cargoDenyChecks = "licenses"`. This change extends both and creates neither. The sibling change `license-notices` covers license notices and the nixpkgs closure.

## What Changes

- Dependabot opens grouped pull requests for `cargo` in `server/`, `npm` in `client/`, `nix` for `flake.lock` and `github-actions`. Each ecosystem waits a cooldown before it proposes a new release.
- `client/.npmrc` makes npm install only releases older than a minimum age and run no install scripts.
- The target `lint` also runs the cargo-deny checks `bans` and `sources` offline and zizmor over `.github`. The existing workflows get fixed or justified for each zizmor finding.
- A workflow of its own checks `server/Cargo.lock` and `client/package-lock.json` against the advisory databases with network access, daily and on pull requests that change a lock file. npm advisories without a fix go into an exception list with a reason and an expiry date.
- The README names the repository settings that only a maintainer can switch. These are Dependabot security updates, secret scanning with push protection and a ruleset that requires the check `Verify` on `main` and restricts who creates `v*` tags.
- arc42 names the new checks and the new risk that artifacts built with Nix are not scanned.

Out of scope are release provenance attestations, a release without the Nix cache, a software bill of materials, Trivy and a move from npm to pnpm. Only the lock files are scanned. The OCI image has no package database of an operating system. The binary is not built with `cargo-auditable`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. The change touches CI, dev tooling and dependency versions, not behavior that a spec describes.

## Impact

- `.github/dependabot.yml` and `.github/workflows/advisories.yaml` are new. `ci.yaml` and `release.yaml` change where zizmor finds a weakness.
- `flake.nix`: the check `rustLicenses` becomes `rustDependencies` with `bans`, `licenses` and `sources`. The new check `workflows` runs zizmor from nixpkgs.
- `server/deny.toml` gains sections for `bans`, `sources` and `advisories`.
- `client/.npmrc`, `client/audit-exceptions.json` and `client/scripts/check-advisories.mjs` are new.
- Contributors who install with npm no longer run install scripts and get no release younger than the minimum age.
- `AGENTS.md` exempts pull requests opened by Dependabot from its rules on drafts and red checks.
- Maintainers handle Dependabot pull requests, including a `flake.lock` update that needs new pins for Rust and `editorconfig-checker`.
- Documentation: README and the arc42 sections on testing and risks.
