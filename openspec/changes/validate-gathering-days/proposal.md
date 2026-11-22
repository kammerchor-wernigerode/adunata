# Proposal

## Why

The gathering form offers every day in both calendars. An organizer can pick a last day before the first day and learns about it only from the answer to saving. A first day in the past passes every layer, although no new gathering can start in the past. When the organizer moves the first day, the last day stays put, so a weekend that moves by a week becomes a gathering of ten days or fails validation.

## What Changes

- The server refuses a new or changed first day before today with the new error code `firstDayInPast` on the field `firstDay`. Today is the day in the time zone of the instance. An unchanged first day of a running gathering stays allowed.
- The calendar of the first day disables every day before today by the date of the browser, except the day already chosen.
- The calendar of the last day disables every day before the chosen first day.
- When the organizer changes the first day while both days are set, the last day moves by the same number of days.
- The form shows a translated message for `firstDayInPast` at the first day, in German and English.
- A first day equal to the last day stays valid. `beforeFirstDay` stays as it is.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `gatherings`: the requirement "Gathering data" gains the rule for a first day in the past, the limits of both calendars and the moving last day.

## Impact

- Server: `server/src/gatherings.rs` (validation, `create_gathering`, `change_gathering`), `server/src/validation.rs` (new `ErrorCode` variant) and the OpenAPI document in `client/openapi.json`.
- Client: `client/src/components/DatePicker.tsx`, `client/src/routes/_organizer/gatherings/-components/GatheringForm.tsx`, the generated client in `client/src/api/generated/` and the catalogs `server/locales/de.ftl` and `server/locales/en.ftl`.
- Tests: server tests that create past gatherings through the API and the client test of the gathering form, which depends on the current month.
- Documentation: `docs/arc42/08-cross-cutting-concepts.md`, section on the time zone of the instance.
- The database gets no new constraint.
