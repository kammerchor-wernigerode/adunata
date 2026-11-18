# Design

## Context

See proposal.md for the motivation and the prerequisites, which are `license-compliance` and the fix of `lettre`.

The flake groups its checks into targets. `lint` builds on nothing and runs offline in the Nix sandbox. CI runs the targets up to `integration` in the job `verify` on every push to a pull request. It runs the jobs `e2e` and `nixos` on pull requests ready for review and on `main`. Every action in `ci.yaml` and `release.yaml` is pinned to a SHA. Both workflows set `permissions: contents: read` at the top.

Three lint checks compare pins for work without Nix against nixpkgs. `rustToolchain` compares the channel in `server/rust-toolchain.toml` with `rustc` of nixpkgs. `editorconfigCheckerVersion` compares `EC_VERSION` in the script `lint:editorconfig` of `client/package.json` with `editorconfig-checker` of nixpkgs. `cargoDenyVersion` compares `cargo-deny@<version>` in the README with `cargo-deny` of nixpkgs. Today all pins match, with Rust 1.95.0, `editorconfig-checker` 3.6.1 and `cargo-deny` 0.19.6.

The client builds in Nix through `importNpmLock`. Its `npmConfigHook` runs `npm install --ignore-scripts` offline against a patched lock file and then `npm rebuild`. The fileset of the client takes only files that git tracks. `nodejs_24` in nixpkgs ships npm 11.19.1, which reads `min-release-age` and turns it into the option `before`.

Findings from probes against the current tree:

- zizmor 1.32.0 from the pinned nixpkgs reports `artipacked` on all four `actions/checkout` steps and `cache-poisoning` on `nix-community/cache-nix-action` in `release.yaml`.
- `cargo deny check advisories` fails on six advisories against `lettre` 0.11.23, all fixed in 0.11.24. The separate fix that the proposal names removes them. `bans` and `sources` pass. `bans` warns about ten crates with two versions each.
- `npm audit` reports two advisories that affect 29 packages. `minimatch` (GHSA-g3hv-75wg-m249) comes in only through `shadcn`, which `license-compliance` removes. That leaves `picomatch` 4.0.7 (GHSA-fc5q-39xc-vjpf, high), which reaches the client through the dev dependencies `vite`, `vitest` and `orval`. Its newest release is 4.0.7, so no fix exists.
- Only `esbuild` and `fsevents` declare install scripts. Both are dev dependencies. `fsevents` is optional and macOS only.

## Goals / Non-Goals

**Goals:**

- Every check this change adds to the flake runs offline, so `nix build .#checks.<system>.integration` stays the one command before a push.
- The advisory workflow starts green once `license-compliance` is merged.

**Non-Goals:**

- A switch from npm to pnpm. pnpm can ignore advisories, allow build scripts per package and enforce a minimum release age itself. `fetchPnpmDeps` needs a fixed hash in `flake.nix` that every Dependabot pull request would break. That trade needs a change of its own.
- Scanning the OCI image, the release binary or the nixpkgs closure.

## Decisions

### Dependabot for version updates

`.github/dependabot.yml` defines four updates, each with a weekly schedule and `cooldown.default-days: 7`.

|Ecosystem|Directory|Groups|
|-|-|-|
|`cargo`|`/server`|One group for minor and patch updates. Each major update gets its own pull request.|
|`npm`|`/client`|One group for minor and patch updates of `dependencies`, one for those of `devDependencies`. Each major update gets its own pull request.|
|`nix`|`/`|One group for the inputs `nixpkgs` and `crane`.|
|`github-actions`|`/`|One group for every action.|

`github-actions` and `nix` accept only `default-days` for the cooldown. `cargo` and `npm` would accept `semver-*-days` as well, but one value for all four keeps the file simple. Security updates ignore the cooldown, as GitHub documents.

Each pull request from Dependabot is ready for review, so it runs `e2e` and `nixos` as well. Grouping keeps this to about six pull requests a week. The default `open-pull-requests-limit` of 5 per ecosystem stays.

Dependabot moves only the locked revisions in `flake.lock`. It does not edit refs in `flake.nix`, as GitHub documents. `nixpkgs` follows the branch `nixos-26.05`, so it gets updates within that release. `crane` is pinned to the tag `v0.24.0`, so its lock entry never moves. A maintainer changes both refs by hand, for example when a new NixOS release appears.

The npm cooldown must be at least the `min-release-age` in `client/.npmrc`. Dependabot runs npm in `client/` with that file, so a shorter cooldown would make it pick a version that npm cannot resolve. The update then fails. Both are 7 days.

Alternatives:

- Renovate can bump versions inside arbitrary files with a regex in `customManagers`. That would cover `EC_VERSION` in `client/package.json` and the version in the header of `client/src/shadcn.css` from `license-compliance`. For `shadcn.css` a bot pull request could only change the number in the header. The new content of the stylesheet still needs `npx shadcn@<version>` by hand. The row on copied UI components in the risk register already covers the copy. So Dependabot stays and nothing tracks shadcn releases. `tailwindcss` and `@tailwindcss/vite` stay normal npm dependencies that Dependabot updates.
- A daily schedule would open more pull requests. Each runs the slow jobs.

Significant: no. It is tooling for development that no operator or running instance notices.

### Updates of `flake.lock` that move pinned versions

An update of `nixpkgs` can bring a new `rustc`, `editorconfig-checker` or `cargo-deny`. The checks `rustToolchain`, `editorconfigCheckerVersion` and `cargoDenyVersion` then fail on the Dependabot pull request, by design. No bot can bump `EC_VERSION` or the version in the README, because they sit inside an npm script and in prose.

A maintainer pushes one commit to the Dependabot branch that sets `channel` in `server/rust-toolchain.toml`, `EC_VERSION` in `client/package.json` and the version of `cargo-deny` in the README to the versions that the error messages name. Rust warnings that a new Clippy brings belong in the same pull request. AGENTS.md asks for draft pull requests and forbids pushing a state with failing checks. Dependabot opens its pull requests ready for review. This one fails by design, so AGENTS.md gets a sentence that exempts pull requests opened by Dependabot. Dependabot stops rebasing a branch that has commits of someone else. If `main` moves before the merge, the maintainer rebases the branch by hand or closes it and lets Dependabot open a fresh one with `@dependabot recreate`, which drops the extra commit. The README describes this under dependency updates.

Alternative: drop the three checks. They exist so that work without Nix uses the same versions as CI, which the README promises.

Significant: no. It concerns the development environment.

### Repository settings

Dependabot alerts are on. Dependabot security updates, secret scanning and push protection for secrets are off. Neither `main` nor tags are protected, so a pull request with red checks can be merged and any token with write access can push a `v*` tag, which publishes an image to ghcr. These are settings of the GitHub repository, not files, so no pull request can switch them.

A maintainer enables security updates, secret scanning and push protection under Settings, Advanced Security. A ruleset for `main` requires the check `Verify` before a merge. A ruleset for tags `v*` restricts their creation, update and deletion. Both let repository admins bypass them, so the archive commit that AGENTS.md puts directly on `main` still works. `Verify` alone is required because `e2e` and `nixos` do not run on drafts. The README lists the settings the repository expects, so a fork or a new maintainer can restore them.

Significant: no. No operator or running instance notices them.

### npm configuration in `client/.npmrc`

`client/.npmrc` sets `min-release-age=7` and `ignore-scripts=true`. npm reads the file for every command run in `client/`, including `npm ci` in the job `e2e` and the build in Nix.

`ignore-scripts=true` stops install scripts of every package. `esbuild` only checks its platform binary in its script. The binary arrives as an optional dependency. `fsevents` is optional and has a prebuilt binary. The Nix build already installs with `--ignore-scripts`, but its `npm rebuild` would now skip scripts as well. `ignore-scripts` also stops `pre` and `post` scripts of `package.json` itself. The client has none. `npm run` still runs the named script.

`min-release-age` sets the option `before`. The Nix build runs offline against a lock file with resolved versions, which `before` should not touch. If it does, `clientPackage` in `flake.nix` sets `npm_config_min_release_age=0`, which npm reads as no limit. The task that adds the file verifies this.

A security fix younger than 7 days cannot be installed with the file in place. Dependabot security updates ignore their cooldown but still run npm with the file. A maintainer then takes the fix with `npm update <package> --min-release-age=0`, because a flag on the command line overrides `.npmrc`. `npm update` also moves a package that comes in through another one, while `npm install` would add it as a direct dependency and leave the nested copies alone. A fix outside the range that the parent package requires goes under `overrides` in `client/package.json`. A probe with npm 11.19 showed that a later `npm ci`, online or with `--offline`, installs a locked version younger than the minimum age. `npm install` keeps it as well. The option only limits which versions npm resolves. `min-release-age-exclude` would exempt a package for good, which is not wanted.

Alternatives: Setting both in the CI workflows only would leave laptops unprotected. pnpm is a non-goal.

Significant: no. It concerns how contributors install dependencies, not the artifacts.

### cargo-deny `bans` and `sources` in the target `lint`

The check `rustLicenses` becomes `rustDependencies` and drops its attribute `cargoDenyChecks`, because crane's default is `"bans licenses sources"`. cargo-deny reads `cargo metadata`, which needs `Cargo.toml` and `Cargo.lock`. It also reads `deny.toml`. crane vendors the crates, so the check runs offline. The name changes because the check now covers more than licenses.

`server/deny.toml` gains these sections:

- `[bans]` with `multiple-versions = "allow"` and `wildcards = "deny"`. Ten crates come in two versions today through upstream crates, which the project cannot change. A Nix check fails only on errors, so a warning would only add noise to the log. `wildcards = "deny"` rejects a dependency without a version requirement.
- `[sources]` with `unknown-registry = "deny"` and `unknown-git = "deny"`. Every crate must come from crates.io.
- `[advisories]`, which only the advisory workflow reads.
- `[licenses]` stays the first table with a key `allow`. `client/scripts/check-licenses.mjs` from `license-compliance` takes the first line `allow = [` in the file. `[bans]` and `[sources]` therefore get no list `allow`. It keeps the defaults and an empty `ignore` list. cargo-deny takes each entry there as `{ id, reason }`. The reason names why the advisory does not apply or when a fix is expected.

Alternative: a separate check per cargo-deny check would start cargo-deny three times for one lock file.

Significant: no. It is a check in CI.

### zizmor in the target `lint`

The check `workflows` runs zizmor from nixpkgs with `--offline` over a source of its own that holds only `.github`. zizmor finds the workflows and `dependabot.yml` there and audits both, including the cooldowns. zizmor is under MIT. Any finding fails the check, so a new workflow starts without known weaknesses.

The existing workflows get these fixes:

- `artipacked`: each `actions/checkout` sets `persist-credentials: false`. No job pushes with git. `release.yaml` creates the release with `gh` and the token in `GH_TOKEN`.
- `cache-poisoning`: the release restores the Nix store from `cache-nix-action`. Removing that cache is out of scope. The step gets an inline `# zizmor: ignore[cache-poisoning]` with the reason. A run for a tag restores only caches that runs on `main` or for tags saved. Caches of pull requests stay in the scope of their merge ref, and pull requests from forks cannot write any. A run through `workflow_dispatch` on another branch can restore caches of that branch, but it publishes nothing, because the step that publishes runs only for tags.
- zizmor does not see it, but `release.yaml` runs `nix run nixpkgs#skopeo` with credentials that can publish. That resolves `nixpkgs` through the global registry instead of `flake.lock`. It becomes `nix run --inputs-from . nixpkgs#skopeo`.

Alternative: only `.github/workflows`. It would leave the cooldowns of `dependabot.yml` unchecked.

Significant: no. It is a check in CI.

### Workflow for advisories

`.github/workflows/advisories.yaml` runs daily on a schedule, on `workflow_dispatch` and on pull requests that change `server/Cargo.lock`, `client/package-lock.json`, `server/deny.toml`, `client/audit-exceptions.json`, `client/scripts/check-advisories.mjs` or the workflow itself. It has `permissions: contents: read` and actions pinned to SHAs, as in `ci.yaml`. It needs network access for the advisory databases, so it stays outside the flake.

One job installs Nix with `cachix/install-nix-action`, which `ci.yaml` already pins. `nix shell --inputs-from . nixpkgs#cargo nixpkgs#rustc nixpkgs#cargo-deny nixpkgs#nodejs_24` then provides the same tools that the flake uses, without a new action. cargo-deny needs `cargo` for `cargo metadata`. Without it from nixpkgs the step would use the rustup of the runner, which installs an unpinned toolchain. One step runs `cargo deny --locked check advisories` in `server/`. The other writes `npm audit --json` in `client/` to a file and ignores its exit code, because npm audit fails on any finding. `node scripts/check-advisories.mjs` then reads that file. The second step also runs when the first fails, so one run reports both.

`client/scripts/check-advisories.mjs` fails on every advisory of severity high or critical whose GHSA ID is not in `client/audit-exceptions.json`. Each exception there has the ID, a reason and an expiry date. The script reads the advisory objects in the `via` list of each entry. An entry such as `vite` that is listed only because it depends on `picomatch` has no advisory of its own and does not count. The script fails when the output has an `error` key or no object `vulnerabilities`, so a failed request to the registry does not pass as a clean run. It also fails on an expired exception. An exception that no longer matches an advisory fails a run on `schedule` or `workflow_dispatch` and only warns on `pull_request`. The Dependabot pull request that brings the fix then merges green. The next daily run asks for the cleanup. Advisories of lower severity only show in the log. Dependabot alerts report them. A maintainer handles them in periodic rounds.

The first exception is `GHSA-fc5q-39xc-vjpf` for `picomatch`. It is reachable only during build and test. No fixed release exists. It expires three months after the change lands.

The npm part duplicates Dependabot alerts for the same advisories. It stays, because it is the only gate that fails a pull request that brings in a new one.

Alternatives:

- `npm audit --omit=dev` would hide the advisory by hiding every dev dependency, including the next one with a fix.
- `npm audit --audit-level=high` without exceptions would stay red until `picomatch` has a fix, so a new advisory would not change the result of the run.
- `patch-package` with the fix from the pull request upstream would add a dev dependency and a patch step that `ignore-scripts` blocks. `npm audit` would still report 4.0.7.
- `EmbarkStudios/cargo-deny-action` would add an action and a second version of cargo-deny.
- Only the cargo step, with npm left to Dependabot alerts. No script to maintain, but nothing fails a pull request on an npm advisory.
- A stale exception that always fails would turn the pull request that brings the fix red. A maintainer commit there stops Dependabot's rebases.

Significant: no. It is a check in CI.

### Documentation

- `docs/arc42/08-cross-cutting-concepts.md`, section Testing: the sentence that every check except the end-to-end tests runs in the flake gains the advisory workflow as a second exception. The paragraph on CI jobs names the workflow. The row `lint` names cargo-deny with bans, licenses and sources and zizmor over `.github`.
- `docs/arc42/11-risks-and-technical-debt.md`: a new row says that only the lock files are scanned. The OCI image and the nixpkgs closure are outside. The release binary carries no list of its crates for a scanner. The mitigation is the weekly `flake.lock` update and the lock file scan.
- README: the checks without Nix run `cargo deny check bans licenses sources`. The sentence on checks that need Nix names zizmor. A new section on dependency updates covers the Dependabot pull requests, the `flake.lock` pins, the refs in `flake.nix` that move only by hand, a security fix under `min-release-age`, the advisory exceptions and the expected repository settings.

- AGENTS.md: a sentence under pull requests that exempts pull requests opened by Dependabot from the rules on drafts and red checks.

No decision is architecturally significant, so the change drafts no decision record.

## Risks / Trade-offs

- [A failing scheduled run notifies only the person who last changed its cron line. GitHub disables schedules after 60 days without activity in the repository] → The README names the run under dependency updates. Dependabot pull requests keep the repository active. Dependabot alerts report the same advisories by mail.
- [The release restores a Nix store that a run on `main` saved] → The cache is out of scope. The inline ignore names the reason, so the finding stays visible in the file.
- [An advisory exception hides a real risk too long] → Each exception expires. The run fails on the expiry date.
- [`ignore-scripts` breaks a future dependency that needs its install script] → The build or the tests fail in that pull request. The maintainer then decides whether to run the script explicitly or drop the dependency.
- [A security fix needs to be younger than the cooldown] → The README names the flag that overrides `min-release-age`.
- [Six pull requests a week each run the slow jobs] → Grouping and a weekly schedule bound them. A maintainer can lower `open-pull-requests-limit` if CI time becomes a problem.
- [Each Dependabot pull request changes a lock file, so `verify` saves a new Nix store of up to 8 GB. The repository has 10 GB of cache, so these saves can evict the cache of `main` that the release restores] → If evictions show up, `verify` saves its cache only on `main`.

## Migration Plan

Contributors run `npm ci` again after the merge. Operators get the fix of `lettre` with the next release, independent of this change. Nothing else changes for them. Rollback reverts the commits. The repository settings stay until a maintainer switches them back.
