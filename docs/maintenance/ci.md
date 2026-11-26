# Continuous integration

The workflows live in [`.github/workflows/`](../../.github/workflows/). The [testing concept](../arc42/08-cross-cutting-concepts.md#testing) says what each level of tests proves. The [contributing guide](../../CONTRIBUTING.md#checks-before-a-push) lists the targets of the flake.

## Workflow `CI`

The workflow `CI` runs its jobs for `x86_64-linux` and `aarch64-linux`, each on a native runner. The job `build` builds the target `integration`, the release binary and the image of its architecture in one call and in one Nix store. It uploads the binary in a release archive and the image as artifacts of the run. On `x86_64` the same call runs the check `nixosVmTest`, so the VM test checks the bytes that the job uploads. The arm64 runner has no KVM, so the NixOS module and the example unit stay untested on `aarch64`. The job `e2e` runs the end-to-end tests on both architectures against the uploaded archive and then starts the uploaded image against PostgreSQL until `adunata healthcheck` succeeds inside the container. On `aarch64` this start is the only test of the image.

A draft pull request builds only `x86_64` without image and VM test and skips `e2e`.

The job `changes` runs first. If a pull request or a push changes only files that no check but `editorconfig` reads, it checks only `editorconfig` and every other job skips. The job lists these paths: the OpenSpec artifacts, the software factory, the files of coding agents and the documentation in `docs/`. A change to `README.md` or `CONTRIBUTING.md` runs every check.

The job `Verdict` needs every other job and fails if one of them failed or was cancelled. It is the only check that the ruleset of `main` requires, so the matrix can change without touching the ruleset. A draft run reports it as `Verdict (draft)`, so its result cannot stand in for the full run of the same commit.

## Workflow `client-dependencies`

The workflow `client-dependencies` writes the hash of the client dependencies to `client/pnpm-deps.hash` on pull requests of Dependabot, so the flake can fetch them. It pushes with the token of a GitHub App, so the push starts CI on the pull request. [Dependency updates](dependency-updates.md) describes what a maintainer does with the hash.

## Workflow `advisories`

The workflow `advisories` checks `server/Cargo.lock` with `cargo deny check advisories` and `client/pnpm-lock.yaml` with `pnpm audit` against the advisory databases. It needs network access, so it runs outside the flake with the tools of its nixpkgs. It runs daily and on pull requests that change a lock file or its configuration. It fails on an advisory of a crate and on an advisory of a client package of severity high or critical that `client/audit-exceptions.json` does not except until a date. [Dependency updates](dependency-updates.md) describes how to except an advisory.
