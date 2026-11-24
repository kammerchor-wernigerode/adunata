# ADR 0026: Calendar file attached to mails about a gathering

## Context

Members read invitations in a mail program. The calendar file of the response page ([ADR 0025](0025-stable-calendar-uids-for-gatherings.md)) is small UTF-8 text. [ADR 0006](0006-outbox-for-mail.md) puts every mail into the outbox in the transaction of its action, and [ADR 0013](0013-mail-loses-its-content-once-its-delivery-ends.md) removes the content once delivery ends.

## Decision

We will attach the calendar file to the invitation, reminder and cancellation mails as the ordinary attachment `gathering.ics`, without `METHOD`, built when the action queues the mail. The outbox stores it in the nullable column `calendar` of `mail_outbox` and clears it in the statements that clear the other content.

## Status

Accepted. It extends the content that ADR 0013 removes.

## Consequences

No storage outside the database and no setting. A mail program shows an ordinary attachment and no controls for accepting. A member who has not answered gets the file too. A change to the gathering after queuing is not in a waiting mail.
