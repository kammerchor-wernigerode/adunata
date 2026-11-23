# Design

## Context

`open_gathering` in `server/src/invitations.rs` closes a gathering only when it is cancelled or has ended. `remind_pending`, `invite_members` and `resend_invitation` all use it. `server/src/responses.rs` ends the member's form with `Gathering::has_begun(today)`, which compares the first day with today in the time zone of the instance. `Gathering::from_row` in `server/src/gatherings.rs` already computes `ended` from today the same way. The client has no value that says a gathering has begun, so `Overview.tsx` cannot hide the button at the right moment.

## Goals / Non-Goals

**Goals:**
- Refuse a reminder from the moment the member's form closes, and hide the button from the same moment.
- Keep one rule for "has begun" on server and client.

**Non-Goals:**
- Refusing invitations or resends for a gathering that has begun. They stay possible until it has ended.
- The wording of the invitation mail for a gathering that has begun.
- A deadline for responses (#92).

## Decisions

### Reuse the error code `begun`

A refused reminder answers with status 422 and `{"errors":[{"field":"gathering","code":"begun"}]}`. The response endpoint already returns this code on the same field when a member answers after the first day has begun, so it means the same thing in both places. A new code would add a second name for one fact. Not architecturally significant, because it extends an existing error contract with an existing code.

### The gathering reports `begun`

`Gathering` gets the field `begun`, computed in `from_row` as `first_day <= today`, next to `ended`. The field replaces the method `has_begun`, and `link_state` in `responses.rs` reads it. One computation then decides the member's form, the refusal and the button, so they cannot drift apart. The client hides the button by this field and so follows the time zone of the instance, not of the browser. The alternative, comparing the first day with the date of the browser, could show the button while the server already refuses. Not architecturally significant, because it adds one field to an existing response and changes no operation of an instance.

### Check in `remind_pending`, not in `open_gathering`

`remind_pending` refuses with `begun` after `open_gathering` returns the gathering. `open_gathering` stays as it is, because inviting and resending keep their rule. Cancelled and ended keep their codes, so a cancelled gathering that has begun still answers `cancelled` and an ended one `ended`. Not architecturally significant, because it is the local placement of one check.

### arc42

No section changes. `05-building-block-view.md` and `08-cross-cutting-concepts.md` mention reminders without the moment they stop.

## Risks / Trade-offs

- A browser that keeps the overview open across midnight of the first day still shows the button until it reloads. The server refuses and the dialog names the failure, as the requirement "Remind action on the overview" already describes.
