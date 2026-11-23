## Why

An organizer who wants an answer from the members who have not responded resends each invitation by hand. The overview of a gathering filters by pending, but it offers no action for all of these members at once.

## What Changes

- New action on the overview of a gathering that reminds every invited member who has not answered and whose link is active.
- Each reminder carries a new personal link. The old link stops working, as it does for a resend.
- New reminder mail in the member's language, with the subject `Erinnerung: <title>, <span>` or `Reminder: <title>, <span>`.
- A confirmation dialog names the number of members before anything is sent.
- The server refuses the action for a gathering that is cancelled or has ended.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `invitations`: adds the requirements for reminding pending members and for the reminder mail.
- `responses`: the overview offers the action, its confirmation and its result.

## Impact

- Server: `server/src/invitations.rs` gains the route `POST /api/gatherings/{id}/reminders` and the reminder mail. `server/locales/*.ftl` gain the texts of the mail and the page.
- Client: `Overview.tsx` gains the action. The shadcn component `dialog` is added under `client/src/components/ui/`.
- No migration.
- Docs: `docs/arc42/05-building-block-view.md` and `docs/arc42/08-cross-cutting-concepts.md`.
