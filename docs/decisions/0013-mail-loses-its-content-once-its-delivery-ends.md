# ADR 0013: Mail loses its content once its delivery ends

## Context

Every mail goes through the outbox, see [ADR 0006](0006-outbox-for-mail.md). An invitation mail contains a personal link that grants access to the member's invitation. A sign-in mail contains a link that starts an organizer session. Content kept in the outbox stays readable in the database and in every backup.

Operators sometimes want to see why a mail failed. The mail server's reply is not stored, see the logging concept in arc42 section 8, and the body of a mail rarely explains a failure.

A member who is deleted must not receive mail that was queued before. The address of a member may have changed since the mail was queued.

## Decision

We will clear recipient, subject and both bodies of a mail in the statement that marks it as sent or as failed for good. The row keeps its times and the number of attempts. A check constraint in the database ties content to a pending mail.

We will store the bare mail address as recipient. Delivery takes the display name from the member that the mail references.

We will reference the member and the invitation of a mail with nullable columns. Deleting a member deletes the member's mail through the foreign key.

We considered keeping the content of failed mail for diagnosis, keeping content for a fixed number of days and encrypting the outbox. The first two keep personal links in backups. The key for the third would sit next to the database on the same host.

## Status

Accepted

## Consequences

A personal link stays in the database only while its mail waits, which is minutes in normal operation and up to the retry period of about a day when the mail server is down.

An operator cannot read the content of a delivered or failed mail. The times and the number of attempts remain for diagnosis.

Organizers resend a failed invitation instead of repairing its mail, which creates a new link.

Mail to a member carries the member's current name at delivery, not the name at the time it was queued.
