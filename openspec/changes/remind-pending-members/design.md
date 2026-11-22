## Context

`resend_invitation` in `server/src/invitations.rs` rotates the secret of one invitation and queues the invitation mail in one transaction. `cancel_gathering` loops over every invitation of a gathering and queues a mail for each. `open_gathering` refuses cancelled and ended gatherings. The overview in `client/src/routes/_organizer/gatherings/$gatheringId/-components/Overview.tsx` already has each row's `status` and `linkActive`, and `InviteMembers.tsx` shows how an action names its count and reports its result. The table `invitations` stores only `secret_hash`, so an existing link cannot be sent again.

## Goals / Non-Goals

**Goals:** one action that reminds every pending member with an active link, with a confirmation dialog and a reminder mail of its own.

**Non-Goals:** a deadline for responses (#92), automatic reminders, reminding a chosen subset, recording when a member was reminded.

## Decisions

Decisions taken from the issue, kept as written:

- The overview offers one action for every member who has not answered.
- A reminder carries a new personal link and the old link stops working, because no link is stored that could be sent again.
- Invitations with a revoked link are skipped. Invitations whose mail failed for good are included.
- A dialog names the number of members and asks for confirmation before anything is sent.
- No time of the last reminder is stored and a second reminder is not blocked.
- The reminder has its own text. Its subject starts with `Erinnerung:` or `Reminder:` and then names title and span. The body says that no answer has arrived yet and otherwise follows the invitation, with the privacy notice and the imprint.
- The server refuses the action for a cancelled or ended gathering.
- Every control is a shadcn component.

Decisions of this change:

- **Route.** `POST /api/gatherings/{id}/reminders`, organizers only, answers 201 with `{ count }`, 404 for an unknown gathering and 422 with the error codes of `open_gathering` for a closed one. A gathering resource keeps the action next to `POST /api/gatherings/{id}/invitations`. Not architecturally significant, because it extends the existing API in its existing style.
- **Recipients query.** Within the transaction, after `open_gathering` locks the gathering, select the invitations of the gathering with `secret_hash IS NOT NULL` and no row in `responses`, locking them with `FOR UPDATE`. Deactivated members are included, as for a resend and the cancellation mail. Not architecturally significant, because no operator sees it.
- **Shared rotation.** Extract the rotation of the secret and the enqueue from `resend_invitation` into a function both paths call, with the mail builder as the only difference. The invitation mail and the reminder mail share one builder that takes the message ids of subject and opening line, so the footer with legal links is not duplicated. Not architecturally significant.
- **Mail texts.** New Fluent messages `reminder-mail-subject` and `reminder-mail-details` in `server/locales/en.ftl` and `de.ftl`. The details line says that no answer has arrived yet and names title, location and span. Greeting and answer lines reuse the invitation messages. Not architecturally significant.
- **Page wording.** Button "Remind N members" and "N Mitglieder erinnern". Dialog title "Remind members?" and "Mitglieder erinnern?". Dialog text says that N members receive a new link and their old link stops working. Buttons "Send reminders" and "Back", in German "Erinnerungen senden" and "Zurück". Result message "N reminders sent" and "N Erinnerungen gesendet". Not architecturally significant.
- **Count on the page.** The button counts overview rows with `status === 'pending' && linkActive`, so no new field is needed. The page offers the button only while the gathering is neither cancelled nor ended, using the same check as `InvitationActions.tsx`. The result message appears where the page shows the message after invitations are sent. Not architecturally significant.
- **Dialog.** Add the shadcn component `dialog` with the shadcn command line tool. Its license is MIT like the other shadcn components. If it pulls in a new package, the commit names it and its license. Not architecturally significant.

No decision records are drafted.

## arc42

- `05-building-block-view.md`: the row of `invitations` names reminding the pending members of a gathering.
- `08-cross-cutting-concepts.md`: the paragraph on personal links says that a reminder, like a resend, stores the hash of a new secret in place of the old one.

## Risks / Trade-offs

- One click replaces the links of many members for good. The confirmation dialog states this before anything happens.
- A member who opens the old link after a reminder sees that it is no longer valid. The reminder in the inbox holds the working link.
