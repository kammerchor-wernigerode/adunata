## Why

A member who accepts an invitation enters the gathering into a calendar by hand, although Adunata knows the title, the location and the days. A calendar file lets the member add it in one step and replace it after a change.

## What Changes

- The response page offers the button "Add to calendar" ("In Kalender eintragen") to a member who accepted or accepted tentatively, until the gathering has ended, also when it is cancelled. One sentence says that the calendar does not follow later changes and that a new download replaces the entry.
- A new route `GET /api/invitation/calendar` behind the personal link returns `gathering.ics` as `text/calendar; charset=utf-8`. The client fetches it with the secret in the header `Authorization` and offers it for saving.
- The file is a plain RFC 5545 entry of whole days with a stable `UID`, `SEQUENCE`, `DTSTAMP`, `CREATED`, `LAST-MODIFIED` and `STATUS`.
- A migration adds `version` and `changed_at` to `gatherings`. Every stored change and the cancellation of a gathering raise `version` and set `changed_at`.
- `InvitationPage` gains a field `calendar` that says whether the page offers the file.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `responses`: the response page offers a calendar file, and the file's content and identity are specified.

## Impact

- Server: new module `calendar` with the writer of the format and the UUID of version 5, the new route in `responses`, the migration `0003`, `gatherings::change` and `gatherings::cancel`.
- Client: the response page in `client/src/routes/invitation/`, the catalogs of both locales and `client/openapi.json`.
- No new dependency. The server computes SHA-1 for the UUID itself.
- Docs: arc42 sections 05 and 08, one new decision record.
