# Proposal

## Why

A member can answer through the personal link until the first day of the gathering begins. An organizer who books beds or orders meals needs final numbers earlier and cannot close the responses before that day.

## What Changes

- A gathering gets an optional deadline for responses, stored as a whole number of days before the first day, at least 1. No value means no deadline, so existing gatherings behave as today. The deadline moves with the first day.
- The form closes for members when the day that lies this number of days before the first day begins. With a first day of 14 March and 7 days, 6 March is the last day to answer.
- The form of the gathering offers a number with the unit days or weeks and a calendar for the last day to answer. The client converts both to days. The API takes only days. The last day to answer lies between today and two days before the first day, and client and server both check this.
- After the deadline a member can still decline through the personal link until the first day begins, with an optional comment. Accepting or accepting tentatively is refused.
- Organizers still record and change responses after the deadline.
- Adunata refuses reminders once the deadline has passed, and the overview hides the action. Inviting and resending stay possible.
- The invitation mail, the reminder mail and the response page name the last day to answer. The page of the gathering shows it to organizers and whether it has passed.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `gatherings`: new requirement "Deadline for responses" for the stored value, its checks and the form.
- `responses`: "Change a response" ends at the deadline. New requirements for the late decline, the last day on the response page and on the page of the gathering, and the remind action after the deadline.
- `invitations`: new requirements for refusing reminders after the deadline and for the last day to answer in both mails.

## Impact

- `server/migrations/`: a new migration adds the nullable column `gatherings.response_deadline_days`.
- `server/src/gatherings.rs`: `Gathering` and `GatheringInput` carry the deadline, `Gathering` reports `deadlinePassed` and `lastDayToAnswer`, `validate` checks the deadline.
- `server/src/responses.rs`: `LinkState` gets `deadlinePassed`, `answer` lets a member only decline in that state, `InvitedGathering` names the last day to answer.
- `server/src/invitations.rs`: `remind_pending` refuses after the deadline, `invitation_mail` names the last day to answer.
- `server/src/validation.rs`: new error codes `deadlinePassed` and `deadlineInPast`.
- `server/locales/*.ftl`: texts for mails, the response page, the form and the errors.
- `client/src/api/generated/`: regenerated from the OpenAPI document.
- `client/src/routes/_organizer/gatherings/-components/GatheringForm.tsx`, `client/src/components/ResponseForm.tsx`, `client/src/routes/invitation/index.tsx`, `client/src/routes/_organizer/gatherings/$gatheringId/-components/Overview.tsx` and `Period.tsx`.
- `docs/arc42/06-runtime-view.md` and `docs/arc42/08-cross-cutting-concepts.md`.
- No new dependency.
