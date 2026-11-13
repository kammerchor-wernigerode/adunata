# Spec Delta

## Purpose

Sends mail from Adunata through the mail server an operator configures, so that no mail is lost when that server is briefly unavailable.

## ADDED Requirements

### Requirement: Mail server configuration
Adunata SHALL send mail through an SMTP server that the operator configures with host, port, sender address and optional credentials. It SHALL require an encrypted connection to the mail server unless the operator explicitly allows an unencrypted one.

#### Scenario: Send through the configured server
- **WHEN** Adunata sends a mail
- **THEN** it delivers the mail to the configured SMTP server with the configured sender address

#### Scenario: Server offers no encryption
- **WHEN** the configured mail server does not support an encrypted connection and the operator has not allowed an unencrypted one
- **THEN** Adunata does not send the mail and logs the reason

### Requirement: Outbox for every mail
Adunata SHALL store every mail in an outbox in the same transaction as the action that causes it. It SHALL send mail from the outbox, never directly from the action.

#### Scenario: Action fails
- **WHEN** an action that causes a mail fails and is rolled back
- **THEN** the outbox holds no mail for that action

#### Scenario: Action succeeds while the mail server is down
- **WHEN** an action that causes a mail succeeds while the mail server is unreachable
- **THEN** the action reports success and the mail stays in the outbox

### Requirement: Retry of failed deliveries
Adunata SHALL retry a failed delivery with increasing delays. It SHALL stop retrying after a configurable number of attempts and mark the mail as failed.

#### Scenario: Mail server comes back
- **WHEN** a delivery fails and the mail server is reachable again at a later attempt
- **THEN** Adunata delivers the mail and marks it as sent

#### Scenario: Permanent failure
- **WHEN** a delivery has failed for the configured number of attempts
- **THEN** Adunata marks the mail as failed and stops retrying

#### Scenario: Restart with pending mail
- **WHEN** Adunata restarts while the outbox holds mail that is not yet sent
- **THEN** it resumes sending that mail

### Requirement: No resend after success
Adunata SHALL mark a mail as sent as soon as the mail server accepts it. It SHALL never resend a mail that is marked as sent.

#### Scenario: Accepted mail
- **WHEN** the mail server accepts a mail
- **THEN** Adunata marks the mail as sent and does not send it again

### Requirement: Plain text and HTML
Adunata SHALL send every mail with a plain text part and an HTML part that carry the same content.

#### Scenario: Mail client without HTML
- **WHEN** a recipient opens a mail from Adunata in a client that shows only plain text
- **THEN** the recipient sees the complete message, including every link
