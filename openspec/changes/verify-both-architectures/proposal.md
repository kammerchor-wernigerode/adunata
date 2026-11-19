# Proposal

## Why

CI verifies only `x86_64-linux`, although the flake builds Adunata for `aarch64-linux` too. Its jobs also pass work between runners through a cache that is nearly full, so the VM test sometimes checks a binary that it built again instead of the one `verify` uploaded. Commits that change only planning artifacts under `openspec/` run the whole pipeline, which is why archive commits go straight to `main` with `[skip ci]`.

## What Changes

- CI verifies `x86_64-linux` and `aarch64-linux` natively in a matrix. Each architecture builds its checks, its release binary and its image once, in one Nix store, and keeps them as artifacts of the run.
- The NixOS VM test runs in the x86 job on the same store paths that the job uploads.
- The end-to-end tests run on both architectures against the uploaded binary and also start the uploaded image.
- For a change that touches only `openspec/`, CI runs only the check `editorconfig`.
- One aggregate check `ci` replaces `Verify` as the check the ruleset of `main` requires.
- The caches are kept per architecture and saved only on `main`.
- Merged OpenSpec changes are archived together in a pull request of their own, so nobody pushes to `main` directly.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. No behavior of Adunata changes, so the change sets `skip_specs`.

## Impact

- `.github/workflows/ci.yaml`, and the cache prefix in `.github/workflows/release.yaml`.
- `README.md`: the jobs and the required check of the ruleset.
- `AGENTS.md`: how OpenSpec changes are archived and which jobs CI runs.
- `docs/arc42/08-cross-cutting-concepts.md`: "Testing".
- The ruleset of `main` requires `ci` instead of `Verify`. A maintainer changes it before the merge.
