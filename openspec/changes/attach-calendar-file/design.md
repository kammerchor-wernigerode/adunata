## Context

`calendar::file(&Gathering, &Revision)` in `server/src/calendar.rs` builds the file of the response page. `gatherings::find_with_revision` is public and returns both inputs. `mail::Mail` holds recipient, subject, text and HTML. `enqueue` writes them to `mail_outbox`, `send` builds `MultiPart::alternative_plain_html`, and `mark_sent` and `schedule_retry` clear the content once delivery ends (ADR 0013). The check constraint `mail_outbox_content_while_pending` ties recipient, subject and both bodies to a pending mail. `invitation_mail` and `cancellation_mail` in `server/src/invitations.rs` build the mails about a gathering. `gatherings::cancel` raises `version` and returns the gathering without its revision.

## Goals / Non-Goals

**Goals:** attach the existing calendar file to the three mails about a gathering, with no change to the file itself.

**Non-Goals:** everything the proposal leaves out. No new dependency, no generic attachment list.

## Decisions

All decisions from the issue stand as written: the three mails carry the file, the sign-in mail none, the attachment has no `METHOD`, `ORGANIZER` or `ATTENDEE`, it shows the state at queue time, every invited member gets it, the sentences are as described, and the file lives in the nullable `TEXT` column `calendar` of `mail_outbox` that is cleared with the rest of the content.

1. **`Mail` gets `calendar: Option<String>`.** One optional field, not a list of attachments, because one kind of attachment exists. The sign-in mail sets `None`. Alternative: a generic attachment table. Rejected, YAGNI.
2. **Message structure.** With a calendar file, `send` builds `multipart/mixed` holding the existing `multipart/alternative` and one `SinglePart` attachment through lettre's `Attachment::new("gathering.ics")` with content type `text/calendar; charset=utf-8`. lettre picks quoted-printable or base64 for non-ASCII text, which satisfies RFC 6047 section 2.5. Without a file the message stays exactly as today.
3. **Revision at queue time.** The actions that queue invitation and reminder mails load the revision with `gatherings::find_with_revision` inside their transaction after `open_gathering` has locked the row, and build the file once per action. The cancellation loads it with `find_with_revision` after `gatherings::cancel` in the same transaction, so it sees the raised `SEQUENCE` and `STATUS:CANCELLED`. The mail builders take the finished file text as a parameter. Alternative: widen `gatherings::cancel` to return the revision. Rejected, a second read in the same transaction is smaller and leaves `gatherings` untouched.
4. **Constraint.** The migration does not add `calendar` to `mail_outbox_content_while_pending`, because the column is optional for a pending mail. Instead it adds the check `mail_outbox_calendar_while_pending` with `calendar IS NULL OR (sent_at IS NULL AND failed_at IS NULL)`, so the database refuses a finished mail that kept its file.
5. **Sentences.** Two new Fluent messages, `invitation-mail-calendar` and `cancellation-mail-calendar`, in `server/locales/en.ftl` and `de.ftl`. The invitation sentence follows the link in both parts. The cancellation sentence follows the body.

**Architectural significance.** Decision 1 together with the column is significant, because an operator sees the new column and the attachment in every mail about a gathering, and because it extends what ADR 0013 promises to remove. It is drafted below. Decisions 2 to 5 are internal and no operator notices them.

### Draft record: ADR 0026 Calendar file attached to mails about a gathering

- **Context:** members read invitations in a mail program. The calendar file of the response page (ADR 0025) is small UTF-8 text. ADR 0006 puts every mail into the outbox in the transaction of its action, and ADR 0013 removes the content once delivery ends.
- **Decision:** the invitation, reminder and cancellation mails carry the calendar file as the ordinary attachment `gathering.ics`, without `METHOD`, built when the action queues the mail. The outbox stores it in the nullable column `calendar` of `mail_outbox` and clears it in the statements that clear the other content.
- **Status:** Accepted. Supersedes none. It extends the content that ADR 0013 removes.
- **Consequences:** no storage outside the database and no setting. A mail program shows an ordinary attachment, no controls for accepting. A member who has not answered gets the file too. A change to the gathering after queuing is not in a waiting mail.

### arc42

- `08-cross-cutting-concepts.md`, mail: a mail about a gathering carries the calendar file, and the content that is cleared includes it.
- `05-building-block-view.md`, `mail`: it sends an optional calendar attachment.
- `09-architecture-decisions.md`: list ADR 0026.

## Risks / Trade-offs

- [A mail waiting for retry carries an outdated file if the gathering changes meanwhile] → accepted. `SEQUENCE` and `DTSTAMP` make a later download win in the calendar.
- [Pending members receive a file the response page refuses them] → the file holds nothing of the member and only what the invitation text already names.

## Migration Plan

Migration `0004` adds the column and the check. Existing rows have `NULL` and satisfy it. Going back to an older release means restoring a backup from before the update.
