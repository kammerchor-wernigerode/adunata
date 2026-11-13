# Spec Delta

## Purpose

Lets operators install, start, update, monitor and stop an Adunata instance on their own server without knowing its internals.

## ADDED Requirements

### Requirement: Supported installation methods
Adunata SHALL be installable as a NixOS module, as an OCI image and as a release binary for Linux. Each method SHALL run the same build of Adunata and SHALL need no runtime other than PostgreSQL and a mail server.

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
