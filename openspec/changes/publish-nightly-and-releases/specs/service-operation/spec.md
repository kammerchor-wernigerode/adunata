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

### Requirement: Nightly image
Every push to `main` that passes all checks SHALL publish the OCI image for `linux/amd64` and `linux/arm64` as an index under the tag `sha-<commit>`. The tag `nightly` SHALL point to the index of the newest commit on `main` whose checks passed. Each index SHALL carry the commit as the annotation `org.opencontainers.image.revision`. Anyone SHALL be able to pull it without credentials.

#### Scenario: Pull nightly
- **WHEN** an operator pulls `ghcr.io/kammerchor-wernigerode/adunata:nightly` on a `linux/arm64` or `linux/amd64` host without signing in to the registry
- **THEN** the container runtime receives the image for the architecture of the host

#### Scenario: Pull a specific commit
- **WHEN** an operator pulls `sha-<commit>` for a commit on `main` whose checks passed
- **THEN** the container runtime receives the image built from that commit

#### Scenario: Failed checks
- **WHEN** a check fails for a push to `main`
- **THEN** the tag `nightly` keeps pointing to the newest commit whose checks passed

#### Scenario: Later commit without a build
- **WHEN** a commit that changes only `openspec/` lands on `main` while the checks of the commit before it still run
- **THEN** the tag `nightly` moves to the commit before it once its checks passed

#### Scenario: Older run finishes late
- **WHEN** the checks of a commit pass after the checks of a newer commit on `main` passed
- **THEN** the tag `nightly` stays on the newer commit

### Requirement: Releases
A release SHALL publish, under the version of the project, exactly the image and the archives that passed every check for its commit, without building them again. It SHALL attach the archives of both architectures with their SHA-256 checksums. Its version SHALL be the same in its tag, in the tags and the annotation `org.opencontainers.image.version` of its image and in the output of `adunata --version`. The tag `latest` SHALL move only for a version without a pre-release part.

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
- **WHEN** someone pushes a tag on a commit whose version equals the version of its first parent
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

#### Scenario: Pre-release
- **WHEN** a release has the version `1.3.0-rc.1`
- **THEN** its image is published as `1.3.0-rc.1` and `latest` stays where it was
