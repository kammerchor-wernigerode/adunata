# Tasks

## 1. Server

- [ ] 1.1 Add a migration with the nullable column `gatherings.response_deadline_days` and its check `>= 1`
- [ ] 1.2 Add `responseDeadlineDays` to `GatheringInput` and `Gathering`, and compute `lastDayToAnswer` and `deadlinePassed` in `Gathering::from_row` with checked arithmetic
- [ ] 1.3 Add the error codes `deadlinePassed` and `deadlineInPast` to `ErrorCode`
- [ ] 1.4 Validate the deadline in `validate` of `gatherings.rs`: `invalid` below 1, `deadlineInPast` for a new last day to answer before today or one that cannot be computed, accepted when it equals the stored one
- [ ] 1.5 Add `LinkState::DeadlinePassed` after `Begun` in `link_state`, let `answer` store only a decline in that state and refuse other statuses with `deadlinePassed`, and show attendees in that state
- [ ] 1.6 Add `lastDayToAnswer` to `InvitedGathering`
- [ ] 1.7 Refuse `remind_pending` with `deadlinePassed` after the check for `begun`, and name the refusal in the OpenAPI description of the route
- [ ] 1.8 Name the last day to answer in the invitation and reminder mail, with the wording for a deadline that has passed, in `server/locales/en.ftl` and `de.ftl`
- [ ] 1.9 Add tests for each scenario of the delta specs on the server, including the boundaries of 6 and 7 March, the huge number, the precedence of states and an organizer's answer after the deadline

## 2. Client

- [ ] 2.1 Regenerate `client/src/api/generated/` from the OpenAPI document
- [ ] 2.2 Add the deadline to `GatheringForm.tsx` with `Input`, `ToggleGroup` for days and weeks and `DatePicker` for the last day to answer, converting both to days, limiting the calendar to today up to two days before the first day, keeping it on "Save & add another" and translating the error codes
- [ ] 2.3 Add a property to `ResponseForm.tsx` that offers only the status declined, and set it on the response page in the state `deadlinePassed` with the notice of that state
- [ ] 2.4 Name the last day to answer on the response page while the link is open
- [ ] 2.5 Show the last day to answer and whether responses closed in `Period.tsx` on the page of the gathering
- [ ] 2.6 Hide the remind button in `Overview.tsx` when `deadlinePassed` is true
- [ ] 2.7 Add the texts in English and German and tests for the form conversion, the calendar limits, the decline-only form, the page line and the hidden remind button

## 3. Documentation

- [ ] 3.1 Update the row "State" in `docs/arc42/06-runtime-view.md` with the deadline, the decline-only form and the order of states
- [ ] 3.2 Name the deadline next to whether a gathering has begun in the paragraph on the time zone in `docs/arc42/08-cross-cutting-concepts.md`
