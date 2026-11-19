# Spec Delta

## ADDED Requirements

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
