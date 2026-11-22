## MODIFIED Requirements

### Requirement: Logs for operators
Adunata SHALL write its logs to standard output in a structured format. Logs SHALL contain no secrets, no invitation links and no message bodies. Local mode SHALL be the only exception. It SHALL add sign-in links with their secrets to the logs, as the capability local-mode describes, and nothing else.

#### Scenario: Log a failed mail delivery
- **WHEN** a mail delivery fails
- **THEN** the log entry names the failure and the outbox entry but not the recipient's address or the message text

#### Scenario: Sign-in link outside development mode
- **WHEN** Adunata sends a sign-in link while local mode is off
- **THEN** no log entry contains the link or its secret

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

