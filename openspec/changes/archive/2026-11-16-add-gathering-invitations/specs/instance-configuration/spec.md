# Spec Delta

## ADDED Requirements

### Requirement: Time zone of the instance
The operator SHALL set the time zone of the instance as an IANA name. Adunata SHALL refuse to start without it or with a name it does not know.

#### Scenario: Unknown time zone
- **WHEN** an operator starts Adunata with the time zone `Europe/Wernigerode`
- **THEN** Adunata names the setting as invalid among all other problems and exits before it touches the database

### Requirement: Trusted proxies
The operator SHALL be able to name the networks of reverse proxies that Adunata trusts. Adunata SHALL take the network address of a request from the header `X-Forwarded-For` only when the request comes from such a network. Without the setting Adunata SHALL trust only the loopback addresses.

#### Scenario: Header from an untrusted client
- **WHEN** a client that is not a trusted proxy sends a request with its own `X-Forwarded-For` header
- **THEN** Adunata uses the address of the connection and ignores the header

### Requirement: Privacy notice and imprint
The operator SHALL be able to set the address of a privacy notice and of an imprint. Adunata SHALL link each one that is set on every page and in every invitation mail.

#### Scenario: Both addresses set
- **WHEN** the operator set both addresses and a member opens the response page
- **THEN** the page links the privacy notice and the imprint

#### Scenario: No address set
- **WHEN** the operator set neither address
- **THEN** pages and mail show no such links
