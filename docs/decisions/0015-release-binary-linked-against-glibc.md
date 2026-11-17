# ADR 0015: Release binary linked against glibc

## Context

[ADR 0005](0005-nix-flake-as-build-definition.md) ships a statically linked binary for musl in the OCI image and in the release archive, so that it runs on any Linux distribution. That binary needs a second toolchain from fenix and a second release build next to the one that the NixOS module and the tests use. The allocator of musl is slower when many threads allocate, and the conventions of musl differ from glibc in edge cases such as error values.

A binary built against glibc runs on distributions whose glibc is at least as new as the newest symbol it uses. The binary that nixpkgs builds needs glibc 2.34 from 2021. Every distribution with an older glibc is out of regular support or more than five years behind.

## Decision

We will build the server once against glibc from nixpkgs and ship that binary in the OCI image and in the release archive. The image carries glibc from nixpkgs. The release archive sets its loader to the standard path of the distribution and searches no path in the Nix store, so it loads glibc and libgcc_s of the distribution. A check fails when the binary needs another library or a version newer than glibc 2.34.

We considered keeping the static binary for musl, which costs a second toolchain and a second build. We considered building against an old glibc, which needs another toolchain as well.

## Status

Accepted

## Consequences

The release binary runs on Linux distributions with glibc 2.34 or newer. Distributions based on musl, such as Alpine, use the OCI image.

The image and the release archive run the binary that the tests check. The NixOS module builds the same source against the nixpkgs of its host, as before. CI compiles the server once, and fenix leaves the flake.

The OCI image grows by the runtime libraries of glibc and GCC, from about 12 MiB to about 69 MiB without compression. It contains no compiler.

An update of nixpkgs or Rust that raises the required glibc or adds a library fails the check, so the requirements change only on purpose.
