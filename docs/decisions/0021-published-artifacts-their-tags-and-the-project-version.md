# ADR 0021: Published artifacts, their tags and the project version

## Context

Ensembles run Adunata on their own hardware, which includes `aarch64-linux` boards. Adunata publishes its image and its release archive only for `x86_64` and only for releases. Nobody can follow `main` without building it. The version lives in several files that nothing compares, and a tag can name any version. ghcr.io creates packages as private, and only a person can make one public.

## Decision

We will publish every image as one OCI index for `linux/amd64` and `linux/arm64` and every release archive for `x86_64-linux` and `aarch64-linux`. Every push to `main` that builds Adunata and passes all checks publishes `sha-<commit>`, and `nightly` points to the newest of them. A release is a tag `v<version>` on the commit of `main` that raises the version. It publishes the bytes that the checks of that commit verified under its version. `latest` names the highest release without a pre-release part. `server/Cargo.toml` holds the one version of the project. Each index names its source, revision and license in OCI annotations, and a release also its version. The package is public.

This supersedes no record.

## Status

Accepted

## Consequences

Operators on either architecture install a release or follow `main` without building Adunata or holding credentials.

`nightly` changes with every merge that builds Adunata, including migrations. It suits test instances, not production. Its annotation `org.opencontainers.image.revision` names the commit.

A release names the same version in its tag, its image, its annotations and `adunata --version`. A tag on any other commit, or with another version, publishes nothing.

A release is possible only while the verified archives of its commit exist. After that, a new commit that raises the version is needed.

The registry holds `sha-<commit>` per commit and one tag per architecture. They accumulate until someone prunes them.

A maintainer makes the package public once after its first publish.
