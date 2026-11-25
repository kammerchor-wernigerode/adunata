## ADDED Requirements

### Requirement: Calendar attachment
A mail MAY carry a calendar file. Adunata SHALL send such a mail with its plain text part and its HTML part as alternatives, followed by the calendar file as an attachment named `gathering.ics` with the media type `text/calendar` and the charset UTF-8. The attachment SHALL be encoded so that characters outside US-ASCII survive a transport that is not 8-bit clean. Adunata SHALL send a mail without a calendar file with only its plain text part and its HTML part, as before.

#### Scenario: Mail with a calendar file
- **WHEN** Adunata delivers a mail that carries a calendar file whose summary contains umlauts
- **THEN** the received mail has a plain text part, an HTML part and one attachment named `gathering.ics` with the content type `text/calendar; charset=utf-8`, and the decoded attachment equals the stored calendar file byte for byte

#### Scenario: Mail without a calendar file
- **WHEN** Adunata delivers the sign-in mail of an organizer
- **THEN** the received mail has a plain text part and an HTML part and no attachment

## MODIFIED Requirements

### Requirement: Content removed after delivery
Adunata SHALL remove the recipient, the subject, the content and the calendar file of a mail from the outbox as soon as the mail server accepts the mail or Adunata marks it as failed. It SHALL keep all of them while a failed delivery waits for its next attempt. It SHALL keep the times and the number of attempts.

#### Scenario: Accepted mail
- **WHEN** the mail server accepts an invitation mail
- **THEN** the outbox keeps no recipient, subject, content, calendar file or link of that mail

#### Scenario: Failed mail
- **WHEN** Adunata marks a mail as failed after the configured number of attempts
- **THEN** the outbox keeps no recipient, subject, content, calendar file or link of that mail and still records that it failed

#### Scenario: Retry keeps the calendar file
- **WHEN** the delivery of a mail with a calendar file fails and attempts remain
- **THEN** the outbox still holds the calendar file, and the next successful attempt delivers it as the attachment
