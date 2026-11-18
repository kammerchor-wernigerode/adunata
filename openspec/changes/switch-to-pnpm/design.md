# Design

## Context

See proposal.md for the motivation.

The flake builds the client with `buildNpmPackage` and `importNpmLock`, which read `client/package-lock.json` without a fixed hash. `clientPackage` wraps the build and backs the checks `contract`, `clientUnitTests`, `clientIntegrationTests` and `clientLint`. The check `clientLicenses` runs `client/scripts/check-licenses.mjs` with `nodejs_24`. The script reads the field `license` of every entry in `package-lock.json`. `toolsOf` lists `nodejs_24`, which the development shell and `toolLicenses` use. `ci.yaml` and `release.yaml` key the Nix cache with `hashFiles` over `client/package-lock.json`. The job `e2e` runs without Nix through `actions/setup-node` with Node.js 24 and `npm ci`. The workflow `advisories` runs `npm audit --json` and `client/scripts/check-advisories.mjs` in a Nix shell with `nodejs_24`. `client/.npmrc` sets `min-release-age=7` and `ignore-scripts=true`. Dependabot updates the ecosystem `npm` in `client/` with a cooldown of seven days. `ci.yaml` already accepts `workflow_dispatch`.

The pinned nixpkgs ships `pnpm` 11.27.0, `pnpm_10` 10.34.0 and `nodejs_26` 26.13.0. It has `fetchPnpmDeps` with `fetcherVersion` 5 and `pnpmConfigHook` at the top level. `fetchPnpmDeps` installs with `--force` and `--ignore-scripts` and ignores the platform. It names its output `<pname>-pnpm-deps`. `pnpmConfigHook` sets `pnpm_config_trust_lockfile=true`, so the offline install in a build skips the supply-chain policies that `fetchPnpmDeps` already applied. pnpm is under MIT.

Findings from probes against a copy of the current client with pnpm 11.27.0 and Node.js 26.13.0 on macOS:

- `pnpm import` turns `package-lock.json` into `pnpm-lock.yaml` with the same versions.
- A frozen install fails with `ERR_PNPM_IGNORED_BUILDS` for `esbuild`, because pnpm 11 rejects a dependency whose build script nobody decided on.
- With `minimumReleaseAge: 10080`, a frozen install rejects the lock file, because `tldts` and `tldts-core` 7.5.2 were released on 2026-11-13. npm installed them, because it checks the age only when it resolves.
- Build, unit tests, integration tests, oxlint and `scripts/check-generated.sh` pass. `dist/licenses.txt` is identical to the one npm builds.
- `pnpm licenses list --json` works without a registry. It groups the installed packages by license and lists 323 packages on macOS.
- `pnpm audit --json` writes an object `advisories` and `metadata.vulnerabilities`. A failed request to the registry writes an object `error` and exits with 0.
- Neither `npm audit` nor `pnpm audit` reports an advisory today, so the probe saw no advisory entry. The exception for `GHSA-fc5q-39xc-vjpf` in `client/audit-exceptions.json` matches nothing.

Dependabot supports pnpm 7 to 12 under the ecosystem `npm`. For pnpm 12 with a pinned version it cannot read `pnpm-lock.yaml`, so alerts and security updates stop.

## Goals / Non-Goals

**Goals:**

- `nix build .#checks.<system>.integration` stays the one command before a push, and it runs offline after the dependencies are fetched.
- Every safeguard of `client/.npmrc` holds with pnpm, and the minimum release age covers locked releases as well.
- `dist/licenses.txt` stays byte for byte the same.

**Non-Goals:**

- A software bill of materials. A follow-up change plans it.
- pnpm 12 and workspaces with more than one package.
- Removing the stale advisory exception. That is the daily work the README describes, not part of the switch.

## Decisions

### pnpm 11 from nixpkgs, pinned through `packageManager`

`client/package.json` sets `"packageManager": "pnpm@11.27.0"`, the version of nixpkgs. The development shell and every Nix build use `pnpm` from nixpkgs. Without Nix, `pnpm/action-setup` in CI and contributors on their machines install the version that `packageManager` names. A new lint check `pnpmVersion` compares the field with `pnpm.version` of nixpkgs and names the version to set, like `editorconfigCheckerVersion`. pnpm 11 switches to the version in `packageManager` by downloading it, which the Nix sandbox forbids. `fetchPnpmDeps` and `pnpmConfigHook` turn that off, and the check keeps both versions equal anyway.

Alternatives:

- pnpm 10 would keep the lock file readable for pnpm 9 tooling. pnpm 11 is what `fetchPnpmDeps` uses by default and what Dependabot supports.
- No field `packageManager` would leave the version for work without Nix unpinned.

Significant: no. The package manager builds the client. No operator or running instance notices which one.

### Node.js 26

`nodejs_26` replaces `nodejs_24` in `toolsOf`, the development shell and `clientLicenses`. The job `e2e` sets `node-version: 26`. `@types/node` moves to `^26`. Node.js 26 is the current LTS line. Node.js 25 and later ship no Corepack, so pnpm comes from nixpkgs or `pnpm/action-setup`, as above. Node.js 26 lands in a commit of its own before the switch of the package manager, so either can be reverted alone.

Significant: no. Node.js builds and tests the client. Neither the server nor the release carries it.

### `client/pnpm-workspace.yaml` instead of `client/.npmrc`

pnpm 11 reads its settings from `pnpm-workspace.yaml` and leaves `.npmrc` to registries and authentication. The file holds:

```yaml
minimumReleaseAge: 10080
allowBuilds:
  esbuild: false
enableGlobalVirtualStore: true
```

The last setting belongs to the decision on worktrees below.

`minimumReleaseAge` counts minutes, so 10080 is seven days, as before. `allowBuilds` names every package with a build script and whether it may run it. `esbuild` only verifies its platform binary in its script, which arrives as an optional dependency, so it stays off. pnpm fails the install when a new dependency brings a build script that the list does not name, so the decision is explicit per package. `ignore-scripts` from `.npmrc` had no such distinction. The client's own `package.json` has no `pre` or `post` scripts.

pnpm checks every locked release against `minimumReleaseAge` where it resolves or fetches. That covers `fetchPnpmDeps` in the Nix build, `pnpm install` on a laptop and in the job `e2e` and every install of the worktree tooling. The offline install inside a Nix build skips the check, because `pnpmConfigHook` trusts the lock file that `fetchPnpmDeps` already checked. The Nix build therefore re-checks only when the fetch runs again, which the name of the fetch derivation ensures (see below).

A security fix younger than seven days goes into `minimumReleaseAgeExclude` with its exact version, for example `picomatch@4.0.8`. The entry stays until the release is older than seven days. The README describes this.

Dependabot's cooldown of seven days covers the package it updates. It is unknown whether Dependabot applies `minimumReleaseAge` to transitive releases that it resolves afresh. If it does not, a version update can lock a young transitive release and fail the fetch. The remedy for a version update is to wait and comment `@dependabot recreate` once the release is old enough. Only a security fix gets an exclude entry. The first Dependabot pull request after the merge shows which case applies, and the README then states it.

Alternatives: keeping `.npmrc` would leave pnpm 11 without any of the settings.

Significant: no. It concerns how contributors install dependencies.

### Client build in Nix with `fetchPnpmDeps`

`clientPackage` becomes a `stdenv.mkDerivation` with `nodejs_26`, `pnpm` and `pnpmConfigHook` in `nativeBuildInputs` and `pnpmDeps` from one shared `fetchPnpmDeps` call with `fetcherVersion = 5`. The source keeps the fileset of git-tracked files in `client/` and `server/locales`. `sourceRoot` stays `source/client`. Builds call `pnpm run <script>` instead of `npm run <script>`.

`fetchPnpmDeps` is a fixed-output derivation and needs the hash of its output. It lives in `client/pnpm-deps.hash` as one SRI string, which `flake.nix` reads with `lib.trim (builtins.readFile ...)`. A file of its own lets a workflow replace it without editing Nix code.

Nix finds a fixed-output derivation by its name and hash. With a name that stays the same, a stale hash would reuse the old output from the store, from a laptop or from the restored CI cache, without fetching. The build would then fail later in `pnpmConfigHook` without naming the new hash, or pass if the lock file only lost packages. The output also depends on pnpm itself, because the fetch archives the store index that pnpm writes. So `pname` of the fetch carries the first eight characters of a SHA-256 over `client/pnpm-lock.yaml`, `client/pnpm-workspace.yaml`, the version of `pnpm` from nixpkgs and the `fetcherVersion`. Any change to one of them gives the fetch a new name. An update of `flake.lock` that moves pnpm then fails on its own pull request, where a maintainer already pushes the commit with the new pins. Nix then fetches again, checks the release age again and fails with the expected hash if the file is stale.

The hash is expected to be the same on every system, because `fetchPnpmDeps` fetches the packages of all platforms. The probe ran only on macOS, so the implementation confirms that the hash computed on macOS builds on `x86_64-linux`.

`fetchPnpmDeps` throws without `fetcherVersion`. `--option abort-on-warn true` in CI turns any warning of the build helpers into a failure as well.

Alternatives:

- `buildNpmPackage` with a lock file converted back to npm would keep two lock files in sync.
- A lock importer without a hash, such as `pnpm2nix`, needs import from derivation. `nix flake check --no-build` on four systems would then build the client during evaluation.

Significant: no. The client in the binary is the same, and `dist/licenses.txt` is identical.

### Workflow that updates the hash on Dependabot pull requests

Each Dependabot pull request that changes `client/pnpm-lock.yaml` changes the hash. The new workflow `.github/workflows/client-dependencies.yaml` updates it.

- It runs on `pull_request` for changes to `client/pnpm-lock.yaml` and only when `github.event.pull_request.user.login` is `dependabot[bot]` and the head repository is this one.
- A first job with `permissions: contents: read` builds the fetch through the attribute `pnpmDeps` of an existing client check, such as `.#checks.x86_64-linux.clientLint.pnpmDeps`. On a mismatch it takes the hash after `got:` from the output. It accepts the value only if it matches `^sha256-[A-Za-z0-9+/]{43}=$`, and hands it on as a job output. Any other failure fails the run. If the hash equals the file, the workflow ends.
- A second job keeps `GITHUB_TOKEN` read-only. It checks out the head commit that the first job built and runs nothing from the pull request. Only then `actions/create-github-app-token` creates a token of the GitHub App `adunata-client-dependencies`, limited to this repository and `contents: write`. The job writes the hash to `client/pnpm-deps.hash`, commits it with `[dependabot skip]` in the message and pushes with that token. If Dependabot moved the branch in between, the push is no fast-forward and fails, and the run for the new head writes the hash instead. A push with the token of an App starts `CI` as a `pull_request` event, so its `Verify` satisfies the ruleset of `main`. The new run of this workflow finds the hash current and ends.
- `[dependabot skip]` lets Dependabot keep rebasing the branch and force-push over the hash commit. The force-push triggers the workflow again, which restores the hash. Without the marker Dependabot stops rebasing once someone else has pushed.

The client ID and the private key of the App are the Dependabot secrets `CLIENT_DEPENDENCIES_APP_CLIENT_ID` and `CLIENT_DEPENDENCIES_APP_PRIVATE_KEY`. A run that Dependabot triggers sees only Dependabot secrets. Secrets of GitHub Actions, and with them secrets of an environment, stay out of its reach. A maintainer creates the App with the repository permission Contents read and write, installs it on this repository only and stores both secrets.

zizmor in the target `lint` audits the workflow. A contributor who changes dependencies by hand updates the file from the error of the build. Without Nix, CI shows that error in the job `verify`.

Alternatives:

- `GITHUB_TOKEN` with `contents: write` and a dispatch of `CI` afterwards. A push with that token starts no run, and the ruleset counts no `Verify` from a run of `workflow_dispatch`, so the pull request would stay blocked.
- A commit status `Verify` that the dispatched run writes with `GITHUB_TOKEN`. Any job with `statuses: write` could then satisfy the ruleset.
- A maintainer pushes the hash by hand, as for pins after an update of `flake.lock`. That adds a manual step to every client pull request from Dependabot.
- `pull_request_target` would give the run write access while it checks out code of the pull request.

Significant: no. It is tooling for CI. No operator or running instance notices it.

### License check from `pnpm licenses list --json`

`pnpm-lock.yaml` holds no licenses. `check-licenses.mjs` reads the JSON of `pnpm licenses list --json` from a file that its first argument names. It keeps the SPDX parser and the list `allow` from `server/deny.toml`. Each key of the JSON is the license expression, and each entry names the package and its versions. A package without a license appears under `Unknown` in pnpm's output and fails the check. The script `lint:licenses` in `package.json` writes the file and runs the script.

The check `clientLicenses` becomes a `clientPackage` that runs the same commands, so it has the installed packages. Its fileset gains `server/deny.toml`. The check runs offline, as the probe shows.

pnpm lists the installed packages, so the check covers the packages that the build, the tests and the artifacts use on the platform of CI. Optional binaries for other platforms, which only run on a contributor's machine, are outside its scope.

Alternatives:

- Installing every platform in the check would cover those binaries too, for no package that CI or an artifact uses.
- Reading licenses from the store that `fetchPnpmDeps` produces depends on the internal layout of that store.

Significant: no. It is a check in CI.

### Advisories from `pnpm audit --json`

`advisories.yaml` watches `client/pnpm-lock.yaml` instead of `client/package-lock.json`. Its Nix shell provides `nixpkgs#pnpm` and `nixpkgs#nodejs_26`. The step writes `pnpm audit --json` to a file and ignores its exit code, as before.

`check-advisories.mjs` reads the object `advisories`. The names of the fields of an entry come from a real report. The implementation runs `pnpm audit --json` of pnpm 11.27.0 in a scratch project with a known vulnerable release and keeps that output as the test input. The script fails when the output has a key `error` or no object `advisories`, because pnpm reports a failed request with exit code 0. It also fails on an entry without an advisory ID or with a severity outside `info`, `low`, `moderate`, `high` and `critical`, so a renamed field cannot let a critical advisory pass. Exceptions in `client/audit-exceptions.json` keep their ID, reason and expiry date and their handling on schedules and pull requests.

Alternatives: `auditConfig.ignoreGhsas` in `pnpm-workspace.yaml` ignores advisories natively but has no reason and no expiry date.

Significant: no. It is a check in CI.

### Dependencies shared between worktrees

Each worktree of the worktree tooling runs `npm ci --prefer-offline` today and gets its own `node_modules` of about 350 MB. npm copies every file from its cache, so ten worktrees hold ten copies.

`client/pnpm-workspace.yaml` sets `enableGlobalVirtualStore: true`. pnpm then keeps each package once in its global store, under `links/` with a hash of its dependencies, and `node_modules` of a checkout holds only symlinks into it. The main checkout and every worktree share these directories. A probe with pnpm 11.27.0 on macOS gave `node_modules` of 344 KB and about 10 MB of new disk space for a second checkout. The install from the warm store took 2.1 seconds. Build, unit tests and integration tests passed, and `dist/licenses.txt` stayed identical. The Vite development server served a font through `/@fs/` from the store with status 200, so its file access rules do not block the store outside the checkout.

`dev/worktree_plugin.py` runs `pnpm install --frozen-lockfile --prefer-offline && pnpm run build`. `--prefer-offline` takes metadata from the store, so a new worktree needs the network only for packages that no other checkout has fetched. pnpm locks the store, so worktrees can install at the same time. Removing a worktree leaves the store untouched. `pnpm store prune` removes packages that no checkout references any more. The README names it.

Inside the Nix build `pnpmConfigHook` unpacks the archive of `fetchPnpmDeps` into a writable temporary directory, and `fetcherVersion` 5 keeps the virtual store out of that archive. The setting therefore works there too. The task that builds the client in Nix verifies that it passes. The job `e2e` and the advisory workflow use the setting as it is.

Alternatives:

- Without the setting pnpm imports each file into `node_modules/.pnpm` of the checkout, as a clone on APFS and as a hard link on most Linux file systems. That saves disk space as well, but it still writes every file per worktree and falls back to copies across file systems.
- A shared `node_modules` through a symlink between worktrees breaks as soon as two worktrees lock different versions.

Significant: no. It concerns the development environment.

### Caching in GitHub workflows

Each workflow caches what it rebuilds and nothing that it must compute afresh.

- `verify`, `nixos` and the release restore the Nix store with `cache-nix-action`. Their keys hash `client/pnpm-lock.yaml` and `client/pnpm-deps.hash` instead of `client/package-lock.json`. `hashFiles` ignores a missing file, so a forgotten key would stop following the client dependencies without an error. The output of `fetchPnpmDeps` is part of the saved store. On a key miss, `restore-prefixes-first-match` restores the newest store, so only the changed fetch and what depends on it build again.
- `e2e` runs without Nix. `pnpm/action-setup` with `package_json_file: client/package.json` installs pnpm before `actions/setup-node`, because `defaults.run.working-directory` does not apply to actions and the action reads `package.json` in the root otherwise. `actions/setup-node` with `cache: pnpm` and `cache-dependency-path: client/pnpm-lock.yaml` saves and restores the store of pnpm. The step `pnpm install --frozen-lockfile` then links from the restored store. The cache of the Playwright browsers stays as it is.
- The workflow for the dependency hash restores no Nix store. Its job exists because the lock file changed, and the name of the fetch follows the lock file, so a restored store would hold no usable output and would only cost the download of several gigabytes. The workflow saves nothing either, so Dependabot runs do not evict the cache of `main`.
- `advisories` caches nothing. `pnpm audit` reads only `pnpm-lock.yaml` and needs no installed packages. The implementation confirms this, otherwise the step installs with `--lockfile-only` first.

Alternatives: a cache of the pnpm store in the Nix jobs would duplicate what the Nix store already holds.

Significant: no. It concerns CI.

### CI, Dependabot and tooling

- The job `e2e` runs `pnpm/action-setup`, `actions/setup-node` with `node-version: 26`, then `pnpm install --frozen-lockfile`, `pnpm exec playwright` and `pnpm run e2e`.
- `dependabot.yml` keeps the ecosystem `npm` and the cooldown of seven days, which equals `minimumReleaseAge`.
- `client/scripts/check-generated.sh` runs `pnpm exec orval` and names `pnpm run generate`. `client/e2e/global-setup.ts` runs `pnpm run build`. `client/e2e/invitation.e2e.ts` names `pnpm run e2e`.
- The message of `editorconfigCheckerVersion` names `pnpm run lint:editorconfig`.
- `.gitignore` of the client stays, because pnpm also installs into `node_modules`.

Significant: no. All of it is tooling for development and CI.

### Documentation

- `docs/arc42/08-cross-cutting-concepts.md`, section Testing: the workflow `advisories` checks `client/pnpm-lock.yaml` with `pnpm audit`. The row `lint` names the pin of pnpm. The contract row and the paragraph on end-to-end tests name `pnpm run`. The paragraph on CI jobs names the workflow that updates the hash of the client dependencies.
- `docs/arc42/11-risks-and-technical-debt.md`: the row on scanned lock files names `client/pnpm-lock.yaml`.
- README: Node.js 26 and pnpm in both setups, every `npm` and `npx` command as `pnpm`, `pnpm dlx shadcn@4.23.0` for components, the hash file and how to update it, the hash workflow on Dependabot pull requests, the security fix through `minimumReleaseAgeExclude`, the store shared by all checkouts with `pnpm store prune` and the pin of pnpm and `client/pnpm-deps.hash` among what an update of `flake.lock` can break.

No decision is architecturally significant, so the change drafts no decision record.

## Risks / Trade-offs

- [A Dependabot pull request runs `CI` once with the stale hash and fails before the workflow pushes] → The push of the App starts a new run on the corrected head, and the ruleset looks only at the head commit.
- [The private key of the App can write to the contents of the repository] → Only runs that Dependabot triggers see it. The token exists only in the second job, after the checkout, and is limited to this repository and `contents: write`. The job runs nothing from the pull request and writes only a value that matched the pattern of an SRI hash.
- [Dependabot may lock a release younger than `minimumReleaseAge`] → The fetch in CI fails. A version update waits and gets recreated, a security fix gets an exclude entry.
- [The switch to pnpm 12 would stop Dependabot alerts, as GitHub documents for its lock file] → The check `pnpmVersion` makes an update of nixpkgs to pnpm 12 a visible step. The move to pnpm 12 needs a change of its own.
- [A tool resolves the real path of a package in the global store and treats it as outside the project] → Build, tests and the Vite development server passed the probe. A tool that fails gets fixed in its configuration, or the setting goes away and pnpm falls back to its per-checkout layout.
- [The global store grows with every version that any checkout ever installed] → `pnpm store prune`, which the README names.
- [The packages that npm resolved include releases younger than seven days] → `jsdom` 31.0.1, `@testing-library/user-event` 14.6.10, `tldts` 7.5.2 and `tldts-core` 7.5.2 pass the age check from 2026-11-20 on. Until then they stand in `minimumReleaseAgeExclude`, and a follow-up removes them once they pass. The name of the fetch follows `pnpm-workspace.yaml`, so the removal makes the fetch check the age again.

## Migration Plan

Contributors delete `client/node_modules`, install pnpm in the version of `packageManager` and Node.js 26 or use the development shell, then run `pnpm install`. Worktrees created after the merge install with pnpm. A branch that predates the merge has no `pnpm-lock.yaml`, so it rebases onto `main` before the worktree tooling refreshes it. Operators notice nothing. Rollback reverts the commits.
