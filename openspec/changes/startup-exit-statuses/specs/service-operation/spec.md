# Spec Delta

## MODIFIED Requirements

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

## ADDED Requirements

### Requirement: Exit status of a failed start
The commands `serve` and `grant-organizer` SHALL exit with 78 when the configuration is invalid, with 65 when the data in the database does not fit this build and with 75 when the connection to the database fails for a temporary reason. Wrong arguments SHALL exit with 2 and every other failure with 1. The log SHALL name the cause.

#### Scenario: Invalid configuration
- **WHEN** Adunata starts with a required setting missing
- **THEN** it exits with status 78 before it touches the database

#### Scenario: Development mode outside localhost
- **WHEN** an operator starts `serve --dev` with a public URL that is not on localhost
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
