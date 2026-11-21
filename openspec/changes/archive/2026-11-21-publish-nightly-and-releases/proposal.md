# Proposal

## Why

Ensembles run Adunata on their own hardware, and some of that hardware is `aarch64-linux`, such as small single-board computers. Adunata publishes its image and its release archive only for `x86_64`, and only for tags, so an operator on arm64 has to build Adunata, and nobody can follow `main` without building it. A release builds and checks everything again in a workflow of its own, so what it publishes can differ from what CI verified. The version lives in several files that nothing compares, and a tag can name any version.

## What Changes

- Every published image and release archive exists for `x86_64` and `aarch64`.
- Every push to `main` that builds Adunata and passes CI publishes the image as one OCI index under `sha-<commit>`, from the artifacts of that run. `nightly` points to the newest such commit. A push that changes only `openspec/` builds nothing and gets no index.
- A release is a tag `v<version>` on the commit of `main` that raises the version. It promotes the verified artifacts of that commit without building them again, and attaches checksums. `latest` names the highest release without a pre-release part.
- Each index names source, commit and license in OCI annotations, and a release also its version.
- `server/Cargo.toml` holds the one version of the project. `flake.nix` reads it, `client/package.json` carries the same, and a check keeps them equal. The version starts at `0.0.0`.
- The release archive becomes a derivation of the flake with a checksum.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `service-operation`: the installation methods cover both architectures, and new requirements cover the nightly image and releases.

## Impact

- `flake.nix`: the release archive, the version from `server/Cargo.toml` and the version check.
- `server/Cargo.toml`, `server/Cargo.lock` and `client/package.json`: version `0.0.0`.
- `.github/workflows/ci.yaml`: uploads the release archive.
- `.github/workflows/publish.yaml` (new) and `.github/workflows/release.yaml` (rewritten).
- `README.md`, `docs/operating/installation.md`, `docs/arc42/` and `docs/decisions/`.
- Depends on `verify-both-architectures`, which builds both architectures and keeps their artifacts.
- The package on ghcr.io has to be made public once after its first publish, because GitHub offers no way to do that from a workflow.
