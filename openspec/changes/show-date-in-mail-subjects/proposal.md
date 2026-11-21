# Proposal

## Why

The subjects of the invitation mail and the cancellation mail name only the title of the gathering. A choir holds gatherings with the same title several times a year, so in the inbox a member cannot tell which gathering a mail belongs to without opening it.

## What Changes

- The subject of the invitation mail reads `<prefix>: <title>, <span>`, for example `Einladung: Probenwochenende, 14.–16. März 2027` or `Invitation: Choir weekend, March 14–16, 2027`.
- The subject of the cancellation mail follows the same form, for example `Abgesagt: Probenwochenende, 14.–16. März 2027` or `Cancelled: Choir weekend, March 14–16, 2027`.
- The span is the one the body of each mail already shows, in the member's language.
- The subject of the sign-in mail stays as it is. No subject names the location.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `invitations`: the requirement "Invitation mail" states what the subject names.
- `gatherings`: the requirement "Cancel gatherings" and its scenario "Cancellation mail" state what the subject of the cancellation mail names.

## Impact

- `server/locales/de.ftl` and `server/locales/en.ftl`: the messages `invitation-mail-subject` and `cancellation-mail-subject`.
- `server/src/invitations.rs`: tests that assert a full subject. The code that builds the mails already passes the span to both subject messages.
- No API, schema, configuration or dependency changes.
