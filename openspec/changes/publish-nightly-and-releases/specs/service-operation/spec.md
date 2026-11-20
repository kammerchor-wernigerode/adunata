# Spec Delta

## MODIFIED Requirements

### Requirement: Supported installation methods
Adunata SHALL be installable as a NixOS module, as an OCI image and as a release binary for Linux with glibc 2.34 or newer. The OCI image and the release binary SHALL exist for `x86_64` and `aarch64`. Each method SHALL run Adunata built from the same source. The container image and the release binary SHALL contain the same binary. No method SHALL need a runtime other than PostgreSQL and a mail server. Besides them, the release binary SHALL need only glibc and libgcc_s of the distribution.

#### Scenario: Install on NixOS
- **WHEN** an operator enables the Adunata module in a NixOS configuration and points it at a database and a mail server
- **THEN** the system runs Adunata as a systemd service that starts on boot

#### Scenario: Install as container
- **WHEN** an operator runs the published OCI image with a valid configuration
- **THEN** Adunata starts and serves the browser client and the API on the configured port

#### Scenario: Install from release binary
- **WHEN** an operator installs the release binary with the example systemd unit and a valid configuration
- **THEN** Adunata starts as a systemd service

#### Scenario: Pull a release on arm64
- **WHEN** an operator pulls the image of a release on a `linux/arm64` host
- **THEN** the container runtime receives the image for `linux/arm64`

#### Scenario: Release archive for arm64
- **WHEN** an operator downloads the release archive for `aarch64-linux`
- **THEN** it holds a binary for `aarch64` with the same files as the archive for `x86_64-linux`

## ADDED Requirements

### Requirement: Image per commit
Every push to `main` that builds Adunata and passes all checks SHALL publish the OCI image for `linux/amd64` and `linux/arm64` as an index under the tag `sha-<commit>`, with the commit as the annotation `org.opencontainers.image.revision`. A push that changes only planning artifacts under `openspec/` builds nothing and SHALL get no index. Anyone SHALL be able to pull the index without credentials.

#### Scenario: Pull a specific commit
- **WHEN** an operator pulls `sha-<commit>` for a commit on `main` that built Adunata and passed its checks
- **THEN** the container runtime receives the image built from that commit for the architecture of the host

#### Scenario: Planning-only commit
- **WHEN** a commit that changes only files under `openspec/` lands on `main`
- **THEN** no index `sha-<commit>` exists for it

### Requirement: Nightly image
The tag `nightly` SHALL point to the index of the newest commit on `main` that has an index `sha-<commit>`.

#### Scenario: Pull nightly
- **WHEN** an operator pulls `ghcr.io/kammerchor-wernigerode/adunata:nightly` on a `linux/arm64` or `linux/amd64` host without signing in to the registry
- **THEN** the container runtime receives the image for the architecture of the host

#### Scenario: Failed checks
- **WHEN** a check fails for a push to `main`
- **THEN** the tag `nightly` keeps pointing to the newest commit on `main` that has an index

#### Scenario: Later commit without a build
- **WHEN** a commit that changes only `openspec/` lands on `main` while the checks of the commit before it still run
- **THEN** the tag `nightly` moves to the commit before it once its checks passed

#### Scenario: Older run finishes late
- **WHEN** the checks of a commit pass after the checks of a newer commit on `main` passed
- **THEN** the tag `nightly` stays on the newer commit

### Requirement: Releases
A release SHALL publish, under the version of the project, exactly the image and the archives that passed every check for its commit, without building them again. It SHALL attach the archives of both architectures with their SHA-256 checksums. Its version SHALL be the same in its tag, in the tags and the annotation `org.opencontainers.image.version` of its image and in the output of `adunata --version`.

#### Scenario: Version of a release
- **WHEN** an operator installs the release `v1.2.0` and runs `adunata --version`
- **THEN** the command prints version `1.2.0`

#### Scenario: Checksums of a release
- **WHEN** an operator downloads the archives of a release
- **THEN** the release holds a file with the SHA-256 checksum of each archive

#### Scenario: Tag does not match the version
- **WHEN** someone pushes the tag `v1.2.0` on a commit whose project version is not `1.2.0`
- **THEN** nothing is published and the release workflow names the mismatch

#### Scenario: Tag on a commit that does not raise the version
- **WHEN** someone pushes a tag on a commit whose version is not higher than the version of its first parent
- **THEN** nothing is published and the release workflow names the reason

#### Scenario: Tag outside main
- **WHEN** someone pushes a tag on a commit that is not on `main`
- **THEN** nothing is published and the release workflow names the reason

#### Scenario: Tag on a commit without passed checks
- **WHEN** someone pushes a tag on a commit for which no run of all checks passed
- **THEN** nothing is published and the release workflow names the missing run

#### Scenario: Artifacts expired
- **WHEN** someone pushes a tag on a commit whose verified archives are no longer kept
- **THEN** nothing is published and the release workflow says that a new commit that raises the version is needed

### Requirement: Tag latest
The tag `latest` SHALL name the release with the highest SemVer precedence among all releases without a pre-release part. The first such release SHALL create it.

#### Scenario: First stable release
- **WHEN** the first release without a pre-release part is published
- **THEN** `latest` names it

#### Scenario: Newer stable release
- **WHEN** `latest` names `1.2.0` and the release `1.3.0` is published
- **THEN** `latest` names `1.3.0`

#### Scenario: Older stable release
- **WHEN** `latest` names `1.2.0` and someone pushes the tag `v1.1.0` on the commit that raised the version to `1.1.0`
- **THEN** the release `1.1.0` is published and the release workflow says that `latest` stays on `1.2.0`

#### Scenario: Pre-release
- **WHEN** a release has the version `1.3.0-rc.1`
- **THEN** its image is published as `1.3.0-rc.1` and `latest` stays where it was
