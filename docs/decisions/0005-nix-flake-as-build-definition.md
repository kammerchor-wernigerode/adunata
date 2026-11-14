# ADR 0005: Nix flake as build definition

## Context

Ensembles run Adunata on NixOS, in containers or as a service on another Linux distribution. The reference installation runs NixOS. Development, CI and every way of installing need the same server, the same client and the same versions of PostgreSQL and Mailpit for tests.

Separate build definitions per target, such as a Dockerfile next to a Nix package, drift apart. A container image built differently from the NixOS package can behave differently.

No ensemble of the target size runs Kubernetes.

## Decision

We will describe every build in one Nix flake. It produces the server with the embedded client, the NixOS module, a statically linked binary and the development shell. The OCI image and the release archive with an example systemd unit both carry that static binary. Its checks cover formatting, lints, unit, slice, contract and end-to-end tests, the memory measurement and a NixOS test that installs Adunata in each supported way. CI runs the checks of the flake and nothing else.

Contributors without Nix get a Compose file with PostgreSQL and Mailpit in the same versions as the development shell.

We considered a Dockerfile next to a Nix package, which means two build definitions that drift apart. We considered Kubernetes manifests or a chart, which no ensemble of the target size would use. Anyone who runs Kubernetes can use the OCI image.

## Status

Accepted

## Consequences

Local checks and CI run the same commands and the same versions. A green `nix flake check` means the same on a laptop as in CI.

Contributors who build or test without Nix rely on the Compose file and their own toolchains, which may differ in versions.

The static binary needs a Rust standard library for musl, which comes from fenix rather than nixpkgs. The OCI image and the release binary build only on Linux.

The NixOS test needs virtual machines, so it runs only on Linux runners with KVM.
