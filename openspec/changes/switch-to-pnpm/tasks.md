# Tasks

## 1. Node.js 26

- [x] 1.1 Replace `nodejs_24` with `nodejs_26` in `toolsOf`, the development shell, `clientLicenses` and `advisories.yaml`, pass `nodejs = tools.nodejs_26` to `buildNpmPackage`, set `node-version: 26` in the job `e2e`, move `@types/node` to `^26` and name Node.js 26 in the README, all in one commit. Verify that `nix develop --command node --version` prints 26.13.0, that `toolLicenses` passes and that `nix build .#checks.<system>.integration` passes

## 2. Client on pnpm

Tasks 2.1 to 2.5 land as one commit, because the flake cannot evaluate between the removal of `package-lock.json` and the new client build.

- [x] 2.1 Add `pnpm` to `toolsOf` and the development shell, add `"packageManager": "pnpm@11.27.0"` to `client/package.json` and the lint check `pnpmVersion`, and change the message of `editorconfigCheckerVersion` to name `pnpm run lint:editorconfig`. Verify that `pnpmVersion` fails with a changed version and passes with the right one
- [x] 2.2 Create `client/pnpm-lock.yaml` with `pnpm import`, add `client/pnpm-workspace.yaml` with `minimumReleaseAge: 10080`, `allowBuilds` for `esbuild` and `enableGlobalVirtualStore: true`, and delete `client/package-lock.json` and `client/.npmrc`. Add a `minimumReleaseAgeExclude` entry only for locked releases still younger than seven days. Verify that `pnpm install --frozen-lockfile` passes and that `pnpm config get minimumReleaseAge` prints 10080
- [x] 2.3 Switch `client/scripts/check-generated.sh`, `client/e2e/global-setup.ts`, `client/e2e/invitation.e2e.ts` and `dev/worktree_plugin.py` to pnpm, and verify with `pnpm run check:generated`. With the main checkout on this branch, create two worktrees through the plugin and verify that both clients build, that their `node_modules` hold only symlinks into the global store and that the second worktree adds only a few megabytes of disk space
- [x] 2.4 Rewrite `client/scripts/check-licenses.mjs` to read the JSON of `pnpm licenses list --json` and point `lint:licenses` at it. Verify that `pnpm run lint:licenses` passes and fails after changing one license in the JSON to one outside `server/deny.toml`
- [x] 2.5 Build `clientPackage` with `stdenv.mkDerivation`, `pnpmConfigHook` and one `fetchPnpmDeps` with `fetcherVersion = 5`, whose `pname` carries a digest of `pnpm-lock.yaml`, `pnpm-workspace.yaml`, the version of pnpm and the `fetcherVersion` and whose hash comes from `client/pnpm-deps.hash`. Switch the client checks to `pnpm run` and make `clientLicenses` a client package with `server/deny.toml` in its fileset. Verify that `nix build .#checks.<system>.integration` passes with `--option abort-on-warn true`, that the hash computed on macOS builds on `x86_64-linux`, that the build passes with `enableGlobalVirtualStore` and that `dist/licenses.txt` is identical to the one built before the switch

## 3. Hash of the client dependencies on Dependabot pull requests

- [x] 3.1 Add `.github/workflows/client-dependencies.yaml` as design.md describes, with a read-only job that builds and validates the hash, a writing job that pushes with `[dependabot skip]` and dispatches `CI`, and every action pinned to a SHA. Verify with `nix build .#checks.<system>.workflows`
- [x] 3.2 Verify the step that extracts the hash locally. Change one locked version while the previous fetch is still in the store and keep the old hash, then check that the build fetches again, fails with the new hash and that the step writes that hash, which a later build accepts. Check that a value outside the SRI pattern is rejected
- [x] 3.3 Push the branch of this change without opening a pull request, run `gh workflow run ci.yaml --ref <branch>` and record with `gh api repos/{owner}/{repo}/commits/<sha>/check-runs` whether `Verify` of the dispatched run reports on the head commit, which the ruleset of `main` requires. If it does not, switch the writing job to a GitHub App token as design.md describes before the hand-over

## 4. CI and advisories

- [x] 4.1 Change the cache keys in `ci.yaml` and `release.yaml` to `client/pnpm-lock.yaml` and `client/pnpm-deps.hash`, and switch the job `e2e` to `pnpm/action-setup`, `cache: pnpm` and pnpm commands, as the decision on caching in design.md describes. Verify with zizmor through `nix build .#checks.<system>.workflows` and with `pnpm run e2e` locally
- [x] 4.2 Verify that `pnpm audit --json` runs on a checkout without `node_modules`. Run `pnpm audit --json` of pnpm 11.27.0 in a scratch project with a known vulnerable release and keep its output as test input. Switch `advisories.yaml` to `client/pnpm-lock.yaml` and `nixpkgs#pnpm`, and rewrite `client/scripts/check-advisories.mjs` for that output. Verify the script with the real output, with that output under an expired exception, with an output that holds `error` and with an entry whose severity is unknown

## 5. Documentation

- [x] 5.1 Update the README as design.md lists, and verify that `git grep -nwE 'npm|npx|package-lock' README.md` and `git grep -n 'Node\.js 24' README.md` finds only the Dependabot ecosystem `npm`
- [x] 5.2 Update the section Testing in `docs/arc42/08-cross-cutting-concepts.md` and the row on scanned lock files in `docs/arc42/11-risks-and-technical-debt.md` with the skill `arc42`, and verify that every relative link resolves

## 6. Integration

- [x] 6.1 Verify that `git grep -nwE 'npm|npx|nodejs_24|package-lock' -- ':!openspec/changes'` and `git grep -n 'Node\.js 24' -- ':!openspec/changes'` finds only intended mentions such as the Dependabot ecosystem `npm`
- [x] 6.2 Run the end-to-end tests with `pnpm run e2e` and verify that they pass

## 7. Hand-over

- [x] 7.1 Run `openspec-verify-change` for `switch-to-pnpm`, fix what it finds and verify that a second run reports no critical issue
- [x] 7.2 Review the whole branch against `origin/main` with the review command of the agent, fix what it finds and verify that no finding is left open
- [x] 7.3 Run `nix build .#checks.<system>.integration` and verify that it passes
- [x] 7.4 Push the branch and open the pull request ready for review. Its description names that it supersedes #35. Verify that the pull request is not a draft and that the job `verify` starts

## Workflow follow-up

- From 2026-11-20 13:00 UTC on, remove the four entries of `minimumReleaseAgeExclude` in `client/pnpm-workspace.yaml` and update `client/pnpm-deps.hash`.
- After the merge, the first Dependabot pull request for the client shows whether the workflow pushes the hash and whether Dependabot respects `minimumReleaseAge` for transitive releases. The README then states the last point.
