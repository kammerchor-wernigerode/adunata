# instance-configuration Specification

## Purpose

Defines how operators configure an Adunata instance and how the service checks that configuration before it starts.

## Requirements

### Requirement: Configuration from the environment
Adunata SHALL read its configuration from environment variables. Every setting SHALL have a documented name. Every setting without a safe default SHALL be required.

#### Scenario: Start with all required settings
- **WHEN** an operator sets every required environment variable to a valid value
- **THEN** Adunata starts with that configuration

### Requirement: Secrets from files
Adunata SHALL accept each secret, such as the database password or the mail server password, as the path to a file that holds the value. It SHALL never require a secret to appear in a command line.

#### Scenario: Read a secret from a file
- **WHEN** an operator sets the variable for the database password file to a readable file
- **THEN** Adunata uses the content of that file as the database password

#### Scenario: Secret file unreadable
- **WHEN** a configured secret file does not exist or cannot be read
- **THEN** Adunata refuses to start with a message that names the setting but not any secret value

### Requirement: Validation before start
Adunata SHALL validate the whole configuration before it opens a port or touches the database. It SHALL report every invalid or missing setting at once and exit with a non-zero status.

#### Scenario: Several settings missing
- **WHEN** an operator starts Adunata without the public base URL and without the mail server host
- **THEN** Adunata exits with a non-zero status and a message that names both settings

#### Scenario: Malformed value
- **WHEN** the public base URL is not an absolute HTTPS URL
- **THEN** Adunata exits with a message that names the setting and the expected format

### Requirement: Public base URL
Adunata SHALL build every link it sends by mail from the configured public base URL. It SHALL require that URL to use HTTPS unless its host is localhost.

#### Scenario: Link in a mail
- **WHEN** Adunata sends a mail that contains a link to the instance
- **THEN** the link starts with the configured public base URL
