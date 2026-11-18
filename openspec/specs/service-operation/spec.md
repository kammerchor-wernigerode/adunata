# service-operation Specification

## Purpose

Lets operators install, start, update, monitor and stop an Adunata instance on their own server without knowing its internals.

## Requirements

### Requirement: Supported installation methods
Adunata SHALL be installable as a NixOS module, as an OCI image and as a release binary for Linux with glibc 2.34 or newer. Each method SHALL run Adunata built from the same source. The container image and the release binary SHALL contain the same binary. No method SHALL need a runtime other than PostgreSQL and a mail server. Besides them, the release binary SHALL need only glibc and libgcc_s of the distribution.

#### Scenario: Install on NixOS
- **WHEN** an operator enables the Adunata module in a NixOS configuration and points it at a database and a mail server
- **THEN** the system runs Adunata as a systemd service that starts on boot

#### Scenario: Install as container
- **WHEN** an operator runs the published OCI image with a valid configuration
- **THEN** Adunata starts and serves the browser client and the API on the configured port

#### Scenario: Install from release binary
- **WHEN** an operator installs the release binary with the example systemd unit and a valid configuration
- **THEN** Adunata starts as a systemd service

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
- **THEN** Adunata exits with a message that names both schema versions and leaves the database unchanged

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
Adunata SHALL write its logs to standard output in a structured format. Logs SHALL contain no secrets, no invitation links and no message bodies.

#### Scenario: Log a failed mail delivery
- **WHEN** a mail delivery fails
- **THEN** the log entry names the failure and the outbox entry but not the recipient's address or the message text

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
