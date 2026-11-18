# Proposal

## Why

The client was set up with npm and Node.js 24 during the spike. npm checks its minimum release age only when it resolves versions, so a young release that is already in the lock file installs without notice. Its only control over install scripts is to switch them off for every package. Every checkout keeps a full copy of `node_modules`, which parallel worktrees multiply. pnpm checks the age of every locked release, decides on build scripts per package, links packages from one store into every checkout and can later write a software bill of materials. The price is a fixed hash of the fetched dependencies in the Nix build, which every update of the lock file changes, and a license check that depends on pnpm's output instead of npm's lock file. Node.js 24 is no longer the newest LTS line, and Dependabot already proposes `@types/node` 26 for the client.

## What Changes

- The client uses pnpm 11 instead of npm. `client/pnpm-lock.yaml` replaces `client/package-lock.json`. `client/pnpm-workspace.yaml` replaces `client/.npmrc` and holds the minimum release age and the packages that may run build scripts. `packageManager` in `client/package.json` pins the version of pnpm.
- pnpm checks every locked release against the minimum release age when it fetches or installs dependencies. A release younger than seven days fails instead of passing silently.
- Node.js 26 replaces Node.js 24 in the flake, the development shell, CI and the README. `@types/node` moves to 26.
- The flake builds the client with `fetchPnpmDeps` and `pnpmConfigHook` from nixpkgs. The hash of the fetched dependencies lives in a file of its own.
- A new workflow updates that hash on pull requests from Dependabot, pushes it with `GITHUB_TOKEN` and dispatches `CI` for the branch, so CI checks the corrected state without a stored secret.
- The license check reads `pnpm licenses list --json` instead of the lock file. The advisory workflow reads `pnpm audit --json`. Exceptions keep their reason and expiry date.
- A lint check compares the pinned pnpm version with nixpkgs, as the existing checks do for Rust and `editorconfig-checker`.
- The main checkout and every worktree share one global virtual store of pnpm. `node_modules` of a checkout holds only symlinks, so a new worktree installs in seconds and costs almost no disk space.
- The CI jobs with Nix keep caching the Nix store, which now holds the fetched pnpm dependencies. The job `e2e` caches the pnpm store. The workflow for the dependency hash and the advisory workflow restore no cache.
- Contributors install with `pnpm install` instead of `npm ci`. Scripts, the worktree tooling and the README follow.

Out of scope are a software bill of materials, workspaces with more than one package and pnpm 12. A follow-up change plans the bill of materials.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. The change touches the build, CI and development tooling. The built client and its license notices stay the same.

## Impact

- `client/`: `package.json`, the new `pnpm-lock.yaml`, `pnpm-workspace.yaml` and hash file, `scripts/check-licenses.mjs`, `scripts/check-advisories.mjs`, `scripts/check-generated.sh` and `e2e/`. `package-lock.json` and `.npmrc` go away.
- `flake.nix`: the client builds with pnpm, Node.js 26 replaces 24 and a new lint check pins pnpm.
- `.github/workflows/`: `ci.yaml`, `release.yaml` and `advisories.yaml` change, a workflow for the dependency hash is new.
- `dev/worktree_plugin.py` installs with pnpm.
- Contributors without Nix install pnpm and Node.js 26.
- Pull request #35 from Dependabot is superseded.
- Documentation: README and the arc42 sections on cross-cutting concepts and on risks.
