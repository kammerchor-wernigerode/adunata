## Why

A member who reads an invitation in a mail program wants the gathering in a calendar. Today the calendar file exists only on the response page, and only after an acceptance or a tentative answer, so the member has to open the page and answer first.

## What Changes

- The invitation mail, the reminder mail and the cancellation mail carry the calendar file of the gathering as the ordinary attachment `gathering.ics` with the media type `text/calendar` and the charset UTF-8. A resent invitation is an invitation mail and carries it too.
- The attachment is the file the response page offers, built by the same code, with no `METHOD`, `ORGANIZER` or `ATTENDEE`. It shows the gathering as it is when the action queues the mail, so the attachment of a cancellation mail has `STATUS:CANCELLED` and a higher `SEQUENCE`.
- Every invited member receives the file, whatever the answer. The rule of the response page stays as it is.
- The invitation mail and the reminder mail say in one sentence next to the personal link that the attached file adds the gathering to a calendar and that saving it does not answer the invitation. The cancellation mail says that the attached file marks the calendar entry as cancelled.
- A migration adds the nullable column `calendar` of type `TEXT` to `mail_outbox`. It is cleared with the rest of the content once a delivery ends and is kept across a retry.
- A mail without a calendar file, such as the sign-in mail, leaves the outbox as it does today.

Left out: scheduling messages with `METHOD:REQUEST` or `METHOD:CANCEL` and answers from a mail program, a mail that confirms an acceptance, a mail to members when a gathering changes, a calendar subscription, storage for large attachments and any setting for the operator.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `mail-delivery`: a mail may carry a calendar attachment next to its text and HTML parts, and the outbox removes it with the rest of the content once delivery ends.
- `invitations`: the invitation mail and the reminder mail carry the calendar file and a sentence about it.
- `gatherings`: the cancellation mail carries the calendar file of the cancelled gathering and a sentence about it.

## Impact

- `server/src/mail.rs`: `Mail`, `enqueue`, `send`, `mark_sent`, `schedule_retry` and the pending mail.
- `server/src/invitations.rs`: `invitation_mail`, `cancellation_mail` and the actions that queue them.
- `server/locales/en.ftl` and `server/locales/de.ftl`: two new messages.
- A new migration in `server/migrations/`.
- `docs/arc42/` and a new decision record in `docs/decisions/`.
