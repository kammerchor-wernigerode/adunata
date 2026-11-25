# Design

## Context

`Gathering::from_row` in `server/src/gatherings.rs` computes `begun` and `ended` from today in the time zone of the instance. `link_state` in `server/src/responses.rs` turns them into `LinkState`, and `answer` refuses every response unless the state is open. `answer_for_member` stores an organizer's response in every state. `remind_pending` in `server/src/invitations.rs` refuses a gathering that has begun after `open_gathering` returned it, which the change `refuse-reminders-once-begun` (#98, merged) introduced. `invitation_mail` builds invitation and reminder from the same template with `MailTexts`. `Localizer::date_span` already renders a single day through `date-single-day`. `GatheringForm.tsx` sends `GatheringInput`, and the client has `DatePicker`, `Input` and `ToggleGroup`. `ResponseForm.tsx` serves both the member's page and the organizer's page.

## Goals / Non-Goals

**Goals:**
- Close the member's form at an optional deadline, except for a decline, and stop reminders at the same moment.
- Keep one computation on the server for whether the deadline has passed.

**Non-Goals:**
- A time of day for the deadline, a default deadline for the instance and mails that Adunata sends on its own before or at the deadline.
- Fixed choices such as one or two weeks in the form.

## Decisions

The decisions of the issue hold as written: an optional deadline in whole days before the first day of at least 1, moving with the first day, closing when the day that lies this number of days before the first day begins, entered as days or weeks or as a day in a calendar, the API taking only days, the last day to answer between today and two days before the first day checked on client and server, a late decline until the first day, organizers answering after the deadline, reminders refused and hidden after the deadline, inviting and resending still possible, the last day to answer in both mails and on the response page, and shadcn components for every control. The decisions below settle what the issue left open.

### Names

The column is `gatherings.response_deadline_days INTEGER NULL CHECK (response_deadline_days >= 1)` in a new migration. The API field is `responseDeadlineDays` on `GatheringInput` and `Gathering`. `Gathering` also reports `lastDayToAnswer` and `deadlinePassed`, and `InvitedGathering` reports `lastDayToAnswer`. The new link state is `deadlinePassed`. The new error codes are `deadlinePassed`, for a member's answer other than a decline and for a reminder, and `deadlineInPast`, for a last day to answer before today. A number below 1 reuses `invalid`. Not architecturally significant, because it adds fields and codes to existing contracts and a nullable column that the migrations add on start.

### The server computes `deadlinePassed` and `lastDayToAnswer`

`from_row` computes `lastDayToAnswer` as `first_day - days - 1` and `deadlinePassed` as `lastDayToAnswer < today`, next to `begun`. The client reads both and so follows the time zone of the instance, as it already does for `begun`. The link state, the refusal of reminders, the overview and the mails all take the same value. Not architecturally significant, because it extends an existing response with derived fields.

### No fixed choices in the form

The form offers the number with the unit days or weeks and the calendar, nothing more. Two weeks is "2" with "weeks", so fixed choices would add a control without a new possibility. Not architecturally significant, because it is a detail of one form.

### A member without an answer may decline late, with a comment

The state `deadlinePassed` offers the decline to every member, answered or not, because an organizer wants to know who does not come. The comment stays optional, as for every decline, so the member can give a reason. The server accepts only the status declined from the personal link in that state. A member who already declined may send the decline again to change the comment. Not architecturally significant, because it is a rule of one endpoint.

### The check against today follows the stored date

`validate` refuses a last day to answer before today unless the gathering changes and its last day to answer equals the stored one, the same rule as for the first day. So an organizer can change the title after the deadline, moving the first day moves the deadline with it and only a new date in the past is refused. Removing the deadline is always valid and reopens the form. The subtraction uses checked arithmetic, and a number too large for a date counts as a day in the past, so the server answers 422 and never 500. Not architecturally significant, because it is validation of one field.

### The page of the gathering shows the deadline

The page of the gathering names the last day to answer next to its days in `Period.tsx`. Once `deadlinePassed` is true, the line says that responses closed after that day. The overview table itself stays as it is. Not architecturally significant, because it is one line on a page.

### Wording of mails after the deadline

Reminders never go out after the deadline, so only an invitation or a resend can go out then. That mail says that the deadline passed on the last day to answer and that the member can still decline through the link. Before the deadline both mails name the last day to answer with `date-single-day`. Not architecturally significant, because it changes the text of mails.

### Precedence of states

The link state checks cancelled, ended, begun, `deadlinePassed` and open in this order. `remind_pending` keeps the codes of `open_gathering` for cancelled and ended, then refuses `begun`, then `deadlinePassed`. Not architecturally significant, because it orders existing checks.

### Decline-only form for members

`ResponseForm` gets a property that limits the statuses to declined. Only `client/src/routes/invitation/index.tsx` sets it in the state `deadlinePassed`, so the organizer's page stays as it is. Not architecturally significant, because it is a detail of one component.

### Save and add another

"Save & add another" keeps the number and the unit of the deadline, like title, location and description, because the deadline is relative to a first day that the organizer enters next. Not architecturally significant, because it is a detail of one form.

### arc42

- `06-runtime-view.md`: the row "State" of the response page names the deadline, the decline-only form and the order of states.
- `08-cross-cutting-concepts.md`: the paragraph on the time zone names whether the deadline has passed next to whether a gathering has begun.

No decision is architecturally significant, so the change drafts no record.

## Risks / Trade-offs

- A browser that keeps a page open across midnight after the last day to answer still shows the old form or button until it reloads. The server refuses and the page names the failure.
- The calendar of the client uses the date of the browser, so near midnight it may offer a day that the server refuses as `deadlineInPast`. The form shows the error at the field, as it does for the first day.
