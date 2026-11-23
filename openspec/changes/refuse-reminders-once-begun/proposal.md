# Proposal

## Why

An organizer can remind pending members while a gathering is under way. Each of them receives a new personal link and the old one stops working. A member can answer only until the first day begins, so the new link shows the gathering without a form. The reminder asks for an answer the member can no longer give.

## What Changes

- Adunata refuses to remind pending members once the first day of the gathering has begun in the time zone of the instance. It then changes no link and queues no mail. The API answers with status 422 and the error code `begun` on the field `gathering`.
- The gathering that organizers see reports whether its first day has begun, from the same rule that ends the member's form.
- The overview hides the remind action from that moment on, as it does for a cancelled or an ended gathering.
- Inviting members and resending an invitation stay possible until the gathering has ended.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `invitations`: "Remind pending members" refuses a gathering that has begun.
- `responses`: "Remind action on the overview" hides the button for a gathering that has begun.

## Impact

- `server/src/gatherings.rs`: `Gathering` gets the field `begun`, which replaces the method `has_begun`.
- `server/src/invitations.rs`: `remind_pending` refuses a gathering that has begun. The OpenAPI description of the route names the new refusal.
- `server/src/responses.rs`: `link_state` reads the field `begun`.
- `client/src/api/generated/`: regenerated from the OpenAPI document.
- `client/src/routes/_organizer/gatherings/$gatheringId/-components/Overview.tsx`: the remind button depends on `begun`.
- No new dependency and no change to the database.
