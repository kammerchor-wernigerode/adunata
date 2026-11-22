# Design

## Context

`invitation_mail` and `cancellation_mail` in `server/src/invitations.rs` build one set of Fluent arguments for the whole mail. It already holds `title`, `location` and `span`, where `span` comes from `Localizer::date_span` in the member's language. The bodies use `$span`. The subject messages `invitation-mail-subject` and `cancellation-mail-subject` in `server/locales/de.ftl` and `server/locales/en.ftl` use only `$title`.

## Goals / Non-Goals

**Goals:**
- Both subjects name the span of the gathering, so a member can match a mail to a gathering in the inbox.

**Non-Goals:**
- The subject of the sign-in mail, which belongs to no gathering.
- The location in any subject.
- Any change to the Rust code that builds the mails.

## Decisions

### Span in both subjects

The invitation mail and the cancellation mail both get the span, because a member has to match both to a gathering. This decision comes from the issue. It is not architecturally significant, because it changes the wording of two mails and no operator or running instance notices it beyond that.

### Comma between title and span

The subjects read `<prefix>: <title>, <span>`, because the body of the mail already writes the span after a comma. This decision comes from the issue. It is not architecturally significant, because it is wording only.

### Span from `Localizer::date_span` in the member's language

The subjects reuse the `$span` argument the mails already pass, because the rest of the mail uses the same span. Only the four Fluent messages change. This decision comes from the issue. It is not architecturally significant, because it reuses an existing helper and adds no new code path.

### Subject goes into the specs

The issue asks that a scenario covers the subject. The issue left open whether the cancellation subject gets a new requirement or extends an existing one. The cancellation mail belongs to the spec `gatherings`, not `invitations`, so each spec extends its existing requirement. `invitations` extends "Invitation mail" and adds the scenario "Subject names title and span". `gatherings` extends "Cancel gatherings" and its scenario "Cancellation mail". No new requirement is needed. This decision is not architecturally significant, because it only places requirements in the specs.

### Tests

The unit tests `cancellation_mail_in_german` and `mail_content` assert the full German subject with the span of their sample gathering. The integration test `cancel_with_mail_to_every_invited_member` asserts the full English subject. Its gathering lies a number of days from today, so the test builds the expected span with `Localizer::date_span("en", …)` from the same days rather than matching a prefix. Queries that select mails by the prefix of their subject keep working unchanged.

## arc42 sections

None. `docs/arc42/` does not describe the wording of mail subjects, so no section changes.

## Risks / Trade-offs

- [A long title and span make the subject long and some clients cut it off] → The title comes first, so a cut keeps what the subject named before.
