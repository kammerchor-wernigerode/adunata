# mail-delivery Specification

## Purpose

Sends mail from Adunata through the mail server an operator configures, so that no mail is lost when that server is briefly unavailable.

## Requirements

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

### Requirement: Content removed after delivery
Adunata SHALL remove the recipient, the subject and the content of a mail from the outbox as soon as the mail server accepts the mail or Adunata marks it as failed. It SHALL keep the times and the number of attempts.

#### Scenario: Accepted mail
- **WHEN** the mail server accepts an invitation mail
- **THEN** the outbox keeps no recipient, subject, content or link of that mail

#### Scenario: Failed mail
- **WHEN** Adunata marks a mail as failed after the configured number of attempts
- **THEN** the outbox keeps no recipient, subject, content or link of that mail and still records that it failed

### Requirement: Mail of deleted members
Adunata SHALL delete every mail in the outbox that it addressed to a member when that member is deleted.

#### Scenario: Member deleted while a mail waits
- **WHEN** an organizer deletes a member whose invitation mail has not yet been delivered
- **THEN** the mail is gone from the outbox and Adunata never sends it

### Requirement: Entered values in HTML mail
Adunata SHALL escape every value that organizers or members entered before it puts the value into the HTML part of a mail.

#### Scenario: Title with markup
- **WHEN** an organizer names a gathering `<b>Probe</b>` and Adunata sends the invitation
- **THEN** the HTML part shows the text `<b>Probe</b>` and contains no bold element

### Requirement: Server certificate checked against the host's trust store
On an encrypted connection, Adunata SHALL verify the certificate of the mail server against the certificate authorities in the trust store of the host it runs on. It SHALL send no mail over a connection whose certificate that store does not trust, and SHALL log the failure as one of encryption.

#### Scenario: Certificate from an authority in the store
- **WHEN** the host's trust store contains the authority that issued the mail server's certificate, whether public or added by the operator
- **THEN** Adunata delivers the mail

#### Scenario: Untrusted certificate
- **WHEN** the mail server presents a certificate that no authority in the host's trust store vouches for
- **THEN** Adunata does not send the mail and logs the failure as one of encryption

#### Scenario: Image without further settings
- **WHEN** Adunata runs in its OCI image and the operator sets no trust store
- **THEN** Adunata verifies the certificate against the bundle of certificate authorities that the image ships

#### Scenario: Image with the operator's bundle
- **WHEN** the operator of the image sets `SSL_CERT_FILE` to a bundle that contains the authority of the mail server
- **THEN** Adunata verifies against that bundle instead of the one the image ships and delivers the mail
