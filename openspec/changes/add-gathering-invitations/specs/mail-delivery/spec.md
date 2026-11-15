# Spec Delta

## ADDED Requirements

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
