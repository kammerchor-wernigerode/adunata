# Spec Delta

## MODIFIED Requirements

### Requirement: Logs for operators
Adunata SHALL write its logs to standard output in a structured format. Logs SHALL contain no secrets, no invitation links and no message bodies. Development mode SHALL be the only exception. It SHALL add sign-in links with their secrets to the logs, as the capability development-mode describes, and nothing else.

#### Scenario: Log a failed mail delivery
- **WHEN** a mail delivery fails
- **THEN** the log entry names the failure and the outbox entry but not the recipient's address or the message text

#### Scenario: Sign-in link outside development mode
- **WHEN** Adunata sends a sign-in link while development mode is off
- **THEN** no log entry contains the link or its secret
