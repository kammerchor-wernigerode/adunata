# service-operation Specification

## Purpose

Lets operators install, start, update, monitor and stop an Adunata instance on their own server without knowing its internals.

## Requirements

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

### Requirement: Single service serves client and API
Adunata SHALL serve the browser client and its API from one process on one port.

#### Scenario: Open the client
- **WHEN** a browser requests the root path of a running instance
- **THEN** Adunata responds with the browser client

#### Scenario: Call the API
- **WHEN** the browser client calls the API on the same instance
- **THEN** the same process answers the request

### Requirement: Database migration on start
Adunata SHALL bring the database schema to the version it expects before it accepts requests. It SHALL refuse to start if a migration fails or if the database schema is newer than it supports.

#### Scenario: Start against an empty database
- **WHEN** Adunata starts against an empty PostgreSQL database
- **THEN** it creates the schema and then accepts requests

#### Scenario: Start after an update
- **WHEN** an operator starts a newer release against the database of an older release
- **THEN** Adunata applies the pending migrations and then accepts requests

#### Scenario: Start after a downgrade
- **WHEN** an operator starts an older release against a database migrated by a newer release
- **THEN** Adunata exits with status 65 and a message that names both schema versions and leaves the database unchanged

### Requirement: Health reporting
Adunata SHALL report on a health endpoint whether it is ready to serve requests. The response SHALL contain no personal data and no configuration values.

#### Scenario: Healthy instance
- **WHEN** a monitor requests the health endpoint while the database is reachable
- **THEN** Adunata responds with a success status

#### Scenario: Database unreachable
- **WHEN** a monitor requests the health endpoint while the database is unreachable
- **THEN** Adunata responds with a failure status that names the database as the cause

### Requirement: Graceful shutdown
Adunata SHALL finish requests in progress before it exits on a termination signal. It SHALL stop accepting new requests once the signal arrives.

#### Scenario: Stop during a request
- **WHEN** the service manager sends a termination signal while a request is in progress
- **THEN** Adunata completes that request and then exits with status zero

### Requirement: Logs for operators
Adunata SHALL write its logs to standard output in a structured format. Logs SHALL contain no secrets, no invitation links and no message bodies. Local mode SHALL be the only exception. It SHALL add sign-in links with their secrets to the logs, as the capability local-mode describes, and nothing else.

#### Scenario: Log a failed mail delivery
- **WHEN** a mail delivery fails
- **THEN** the log entry names the failure and the outbox entry but not the recipient's address or the message text

#### Scenario: Sign-in link outside development mode
- **WHEN** Adunata sends a sign-in link while local mode is off
- **THEN** no log entry contains the link or its secret

### Requirement: Third-party notices served by the instance
A running instance SHALL serve its third-party notices without authentication. `/licenses.txt` SHALL be UTF-8 plain text with the license text and copyright notices of every third-party crate compiled into the server, of every third-party package in the browser client and of third-party code copied into the repository. `/licenses-rust.html` SHALL hold the copyright notices of the Rust standard library, and `/licenses.txt` SHALL name it.

#### Scenario: Read the notices of an instance
- **WHEN** anyone requests `/licenses.txt` from a running instance
- **THEN** Adunata responds with `text/plain; charset=utf-8` and a file that names each third-party crate and client package with its version, license text and copyright notices

#### Scenario: Notices of the Rust standard library
- **WHEN** anyone requests `/licenses-rust.html` from a running instance
- **THEN** Adunata responds with the copyright notices of the Rust standard library that the binary was built with, including its own third-party dependencies

#### Scenario: Notices match the build
- **WHEN** a dependency is added to the server or to the client
- **THEN** the next build of the binary serves notices that contain it

#### Scenario: Code that reaches the browser outside the JavaScript bundle
- **WHEN** a package reaches the browser only through the stylesheet, such as Tailwind CSS
- **THEN** the notices contain its license text and copyright notice

#### Scenario: Copied code
- **WHEN** a running instance serves its notices
- **THEN** they contain the source, version, license text and copyright notice of each piece of third-party code copied into the repository, such as the UI components and the stylesheet from shadcn

#### Scenario: The client points to the notices
- **WHEN** a browser loads the client
- **THEN** its HTML document links `/licenses.txt` as its license

### Requirement: Third-party notices in the release archive
The release archive SHALL contain the same notice files next to `LICENSE`.

#### Scenario: Unpack the release archive
- **WHEN** an operator unpacks the release archive
- **THEN** it contains `licenses.txt` and `licenses-rust.html` with the same content the binary serves

### Requirement: Third-party notices in the OCI image
The OCI image SHALL contain the notices of the binary and, for each package from nixpkgs in its runtime closure, the license text and a reference to the exact source the package was built from.

#### Scenario: Inspect the image
- **WHEN** an operator copies the license directory out of the OCI image as the operating guide describes
- **THEN** they find the notices of the binary and an entry for each package from nixpkgs in the image, such as glibc and the CA bundle, with its license text and source reference

### Requirement: Exit status of a failed start
The commands `serve` and `grant-organizer` SHALL exit with 78 when the configuration is invalid, with 65 when the data in the database does not fit this build and with 75 when the connection to the database fails for a temporary reason. Wrong arguments SHALL exit with 2 and every other failure with 1. The log SHALL name the cause.

#### Scenario: Invalid configuration
- **WHEN** Adunata starts with a required setting missing
- **THEN** it exits with status 78 before it touches the database

#### Scenario: Development mode outside localhost
- **WHEN** an operator starts `serve --local` with a public URL that is not on localhost
- **THEN** Adunata exits with status 78

#### Scenario: Wrong arguments
- **WHEN** an operator runs `adunata` with an unknown command
- **THEN** it exits with status 2

#### Scenario: Applied migration changed or missing
- **WHEN** Adunata starts against a database that holds an applied migration which the build does not have or has with other content
- **THEN** it exits with status 65

#### Scenario: Migration fails on the data
- **WHEN** a pending migration fails because the data or the schema in the database violates one of its statements
- **THEN** Adunata exits with status 65

#### Scenario: Migration lacks a privilege
- **WHEN** a pending migration fails because the database user lacks a privilege
- **THEN** Adunata exits with status 1

#### Scenario: Database unreachable
- **WHEN** Adunata starts while no database answers at the configured address within the timeout
- **THEN** it exits with status 75

#### Scenario: Connection lost during a migration
- **WHEN** the connection to the database breaks off or the database shuts down while a migration runs
- **THEN** Adunata exits with status 75

#### Scenario: Database rejects the credentials
- **WHEN** the database rejects the password or the database does not exist
- **THEN** Adunata exits with status 1

#### Scenario: Listen address in use
- **WHEN** another process already listens on the configured listen address
- **THEN** Adunata exits with status 1 and a message that names the address

### Requirement: Restart after a failed start
The NixOS module and the example systemd unit SHALL restart Adunata after a failure, except after status 65 or 78. An operator SHALL be able to override that policy.

#### Scenario: Temporary failure
- **WHEN** the service exits with status 75
- **THEN** systemd starts it again after five seconds

#### Scenario: Failure that a restart cannot cure
- **WHEN** the service exits with status 65 or 78
- **THEN** systemd leaves it stopped

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
- **WHEN** a commit that changes only `openspec/` lands on `main` while the checks of the commit before it, which builds Adunata, still run
- **THEN** the tag `nightly` moves to the commit before it once that commit has its index, unless a newer commit on `main` has an index by then

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
- **WHEN** `latest` names `1.2.0` and someone pushes the tag `v1.1.0` on the commit of `main` that raised the version to `1.1.0` and whose verified archives are still kept
- **THEN** the release `1.1.0` is published and the release workflow says that `latest` stays on `1.2.0`

#### Scenario: Pre-release
- **WHEN** a release has the version `1.3.0-rc.1`
- **THEN** its image is published as `1.3.0-rc.1` and `latest` stays where it was

### Requirement: Healthcheck command
The command `healthcheck` SHALL ask the instance on the configured listen address for its readiness through the health endpoint and replace an unspecified address by the loopback address of the same family. It SHALL exit with 0 when the instance reports ready and with 1 in every other case, including an invalid configuration and no answer within ten seconds. It SHALL read the same settings as `serve`.

#### Scenario: Ready instance
- **WHEN** an operator runs `adunata healthcheck` with the settings of an instance that reports ready
- **THEN** the command exits with status 0

#### Scenario: Database unreachable
- **WHEN** the instance reports that the database is unavailable
- **THEN** the command exits with status 1

#### Scenario: No instance listening
- **WHEN** no instance listens on the configured address
- **THEN** the command exits with status 1

#### Scenario: No answer in time
- **WHEN** the instance accepts the connection but sends no answer within ten seconds
- **THEN** the command exits with status 1

#### Scenario: Invalid configuration
- **WHEN** the settings miss a required value
- **THEN** the command exits with status 1

#### Scenario: Listen address on all IPv4 interfaces
- **WHEN** the listen address is `0.0.0.0:8080`
- **THEN** the command asks `127.0.0.1:8080`

#### Scenario: Listen address on all IPv6 interfaces
- **WHEN** the listen address is `[::]:8080`
- **THEN** the command asks `[::1]:8080`

### Requirement: Output of the healthcheck
When the command `healthcheck` fails, it SHALL write one line that names the cause, such as the status of the answer, a failed connection, a timeout or an invalid setting. It SHALL never print the body of the answer or the value of a setting.

#### Scenario: Instance not ready
- **WHEN** the instance answers with status 503
- **THEN** the command writes a line that names status 503 and nothing from the body

### Requirement: Healthcheck left to the operator
The OCI image SHALL declare no healthcheck. The operating documentation SHALL show how an operator runs the command `healthcheck` as the healthcheck of Docker Compose and of Podman.

#### Scenario: Inspect the image
- **WHEN** an operator inspects the config of the published image
- **THEN** it holds no healthcheck

### Requirement: Warning while mail is confined
`serve` SHALL log a warning at start while mail confinement is on, before it accepts requests. The warning SHALL say that every mail goes to the redirect address. It SHALL not hold the redirect address.

#### Scenario: Start with confinement on
- **WHEN** an operator starts `serve --dev` with `ADNT_MAIL_CONFINE=true` and `ADNT_MAIL_CONFINE_TO=test@example.org`
- **THEN** the log holds one entry at level warning that says mail confinement is on, and no log entry holds `test@example.org`

#### Scenario: Start with confinement off
- **WHEN** an operator starts `serve --dev` without the confinement settings
- **THEN** the log holds no warning about mail confinement
