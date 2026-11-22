# Design

## Context

See proposal.md for the motivation. `validate` in `server/src/gatherings.rs` checks the fields of `GatheringInput` alone. It knows neither today nor the stored gathering. `change_gathering` updates with `WHERE last_day >= today` and reads the gathering only afterwards to tell `ended` from 404. `find_for_update` in the same module already loads a gathering and locks its row. `state.time_zone.today()` gives today in the time zone of the instance and already decides `ended`.

On the client, `DatePicker` in `client/src/components/DatePicker.tsx` disables days only when it gets both `firstDay` and `lastDay`. `ResponseForm` passes both. `GatheringForm` passes neither and keeps both days as ISO strings in react-hook-form.

## Goals / Non-Goals

**Goals:**
- One home for the rule on the first day in the past, the validation of the gathering module on the server.
- Calendars that offer only days that can be valid, without a second copy of the server rule.

**Non-Goals:**
- No database constraint, because the rule depends on the stored value and on today.
- No rule for a last day in the past and no upper limit for the length of a gathering.
- No change to responses or invitations when the span changes.

## Decisions

Each decision below is not architecturally significant, because neither an operator nor a running instance notices it beyond the behaviour that the spec `gatherings` already records. The change drafts no decision record.

### Decisions of the issue

These are kept as the issue states them.

- A first day equal to the last day stays valid, because one-day gatherings such as a concert exist.
- The server refuses a first day in the past only when it is new or differs from the stored one, because organizers must keep editing a running gathering.
- The server decides what today is, with the time zone of the instance, because it already does so for `ended`. The form uses the browser's date only to disable days in the calendar.
- The last day moves by the difference only when it is already set, because an empty last day has nothing to keep. Likewise nothing moves while the first day is still empty, because there is no difference to apply.

### Error code `firstDayInPast`

The new variant `FirstDayInPast` of `ErrorCode` in `server/src/validation.rs` serializes as `firstDayInPast` and is reported on the field `firstDay`. The catalogs get the key `error-first-day-in-past` with "The first day cannot lie in the past." and "Der erste Tag kann nicht in der Vergangenheit liegen.", which follow the wording of `error-last-day-before-first-day`. A first day equal to today is allowed.

### Validation takes today and the stored first day

`validate` gets today and the stored first day as an `Option<Date>`. It refuses the first day when it lies before today and differs from the stored first day. `create_gathering` passes `None`, so every past first day is refused. Field errors keep being collected together, so a request with an empty title and a past first day gets both errors. Alternative considered: a separate check in each handler. Rejected, because the rule would then have two homes.

### Order of answers when a gathering changes

`change_gathering` opens a transaction and loads the gathering with `find_for_update`. It answers in this order and stops at the first that applies.

1. No such gathering: status 404.
2. The gathering has ended: status 422 with `ended` on the field `gathering`, as today.
3. Field errors from `validate`, including `firstDayInPast`: status 422.
4. Otherwise it updates the row and answers status 200.

An unknown id with an invalid body therefore answers 404 instead of 422. Alternative considered: keep validating before loading. Rejected, because `validate` now needs the stored first day, and loading once under a lock avoids a race between reading the stored day and writing the new one.

### Calendar limits in `DatePicker`

`DatePicker` disables days before `firstDay` when given and days after `lastDay` when given, each on its own. The day of `value` is never disabled, so the calendar always shows the chosen day as selectable. `ResponseForm` passes both limits and behaves as before.

`GatheringForm` passes today by the browser's date as `firstDay` to the calendar of the first day, and the chosen first day as `firstDay` to the calendar of the last day. The calendar of the last day has no lower limit while the first day is empty.

The stored first day of a running gathering stays enabled because it is the chosen value. Once the organizer moves the first day to another day, the stored past day is disabled like every other past day. Choosing it again needs the button to cancel the edit. This keeps the picker free of a special case for the stored value, and the server still accepts the stored day if the form sends it.

### Moving the last day

`GatheringForm` handles the change of the first day itself. When the previous first day and the last day are both set, it sets the last day to the last day plus the difference in calendar days between the new and the previous first day. It uses `differenceInCalendarDays` and `addDays` from `date-fns`, which the client already depends on. The move also applies when the first day moves earlier.

### Tests that need a fixed date

Client tests of the gathering form pin the browser's date with `vi.useFakeTimers({ toFake: ['Date'] })` and `vi.setSystemTime`, so they do not depend on the day they run. Server tests that need a gathering in the past store it with `invitations::testing::gathering`, which `members.rs` already uses, because the API now refuses past first days.

## arc42

- `docs/arc42/08-cross-cutting-concepts.md`, the paragraph on the time zone of the instance: it says every comparison with the present goes through the function of the module `configuration`. It gains one sentence that the calendars of the client may use the browser's date to disable days, while the server alone decides whether a day lies in the past.

## Risks / Trade-offs

- [The browser's date and the instance's date differ around midnight or across time zones] → The calendar may offer yesterday or hide today for a short time. The server decides, and the form shows `firstDayInPast` at the first day.
- [Existing tests create past gatherings through the API] → They switch to `invitations::testing::gathering`. Tasks name each test.
- [The client test of the form picks days of the current month] → It pins the date and picks the last day before the first day, so the case `beforeFirstDay` stays reachable and testable.
