# Tasks

## 1. Subjects

- [x] 1.1 Change `invitation-mail-subject` and `cancellation-mail-subject` in `server/locales/de.ftl` to `Einladung: { $title }, { $span }` and `Abgesagt: { $title }, { $span }`, and in `server/locales/en.ftl` to `Invitation: { $title }, { $span }` and `Cancelled: { $title }, { $span }`. Leave `sign-in-mail-subject` as it is. Verify with `grep -n "mail-subject" server/locales/*.ftl`.
- [x] 1.2 In `server/src/invitations.rs`, make `cancellation_mail_in_german` assert the subject `Abgesagt: Probenwochenende, 12.–14. März 2027` and `mail_content` assert `Einladung: Probenwochenende, 12.–14. März 2027`. Verify both tests pass with `cargo test` in `server/`.
- [x] 1.3 In `cancel_with_mail_to_every_invited_member` in `server/src/invitations.rs`, expect the subject `Cancelled: Probenwochenende, <span>` for both recipients, where `<span>` is `localization::localizer().date_span("en", …)` of the gathering's first and last day (`api::testing::day(10)` and `api::testing::day(12)`). Keep an exact comparison, not a prefix match. Verify the test passes in the integration check.
