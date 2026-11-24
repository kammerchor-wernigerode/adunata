## 1. Outbox

- [x] 1.1 Add migration `server/migrations/0004_attach_calendar_file.sql` with the nullable `TEXT` column `calendar` and the check `mail_outbox_calendar_while_pending`
- [x] 1.2 Add `calendar: Option<String>` to `mail::Mail`, write it in `enqueue`, read it into the pending mail and clear it in `mark_sent` and in the final branch of `schedule_retry`
- [x] 1.3 In `send`, wrap the alternative parts and a `gathering.ics` attachment of type `text/calendar; charset=utf-8` in `multipart/mixed` when a file is present, and leave the message unchanged otherwise
- [x] 1.4 Integration tests in `server/src/mail.rs` against Mailpit: attachment name, content type and byte-equal decoded content with umlauts, no attachment without a file, file kept across a retry, `content_left` includes `calendar` for sent and failed mail

## 2. Mails about a gathering

- [x] 2.1 Add `invitation-mail-calendar` and `cancellation-mail-calendar` to `server/locales/en.ftl` and `server/locales/de.ftl`
- [x] 2.2 Let `invitation_mail` and `cancellation_mail` take the calendar file, set it on `Mail` and put the sentence after the link or the body in both parts
- [x] 2.3 In `invite_members`, `resend_invitation`, `remind_pending` and `cancel_gathering`, load the revision with `gatherings::find_with_revision` in the transaction and build the file with `calendar::file`, after the cancellation for `cancel_gathering`
- [x] 2.4 Set `calendar: None` for the sign-in mail and every other `Mail`
- [x] 2.5 Tests: unit tests of both builders for the sentences in German and English, and integration tests that the outbox of an invitation, a reminder to a pending member, a resend and a cancellation holds the file of `calendar::file` with the expected `SEQUENCE` and `STATUS`

## 3. Documentation

- [x] 3.1 Write `docs/decisions/0026-calendar-file-attached-to-mails-about-a-gathering.md` from the draft in design.md
- [x] 3.2 Update `docs/arc42/08-cross-cutting-concepts.md`, `docs/arc42/05-building-block-view.md` and `docs/arc42/09-architecture-decisions.md` as design.md names
