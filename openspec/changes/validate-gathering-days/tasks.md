# Tasks

## 1. Server

- [x] 1.1 Add the variant `FirstDayInPast` to `ErrorCode` in `server/src/validation.rs` and verify `cargo test` serializes it as `firstDayInPast`.
- [x] 1.2 Let `validate` in `server/src/gatherings.rs` take today and the stored first day as `Option<Date>` and refuse a first day before today that differs from the stored one with `firstDayInPast` on `firstDay`. Verify with unit tests in the module `tests`: a past first day without a stored day is refused, today is accepted, an unchanged past first day is accepted, a changed past first day is refused, and a first day equal to the last day is accepted.
- [x] 1.3 Pass `state.time_zone.today()` and `None` from `create_gathering`. Verify with an integration test that `POST /api/gatherings` with `weekend(-1, 1)` answers 422 with `{ "errors": [{ "field": "firstDay", "code": "firstDayInPast" }] }` and with `weekend(0, 0)` answers 201.
- [x] 1.4 Rework `change_gathering` to load the gathering with `find_for_update` in a transaction and answer in the order of design.md: 404, `ended`, field errors, update. Verify with integration tests: a running gathering stored with `invitations::testing::gathering(pool, ..., -2, 2)` accepts a `PUT` with `weekend(-2, 3)`, refuses one with `weekend(-1, 3)` with `firstDayInPast` and keeps its stored days, and `PUT /api/gatherings/999` still answers 404.
- [x] 1.5 Store the past gatherings of `gathering_has_ended` and `ended_gathering_stays_as_it_was` with `invitations::testing::gathering` instead of `POST /api/gatherings` and verify both tests pass unchanged in their assertions.
- [x] 1.6 Regenerate `client/openapi.json` with `cargo run -- openapi > ../client/openapi.json` in `server/` and the client with `pnpm run generate` in `client/`. Verify `ErrorCode` in `client/src/api/generated/model/errorCode.ts` lists `firstDayInPast` and `pnpm run check:generated` passes.

## 2. Messages

- [x] 2.1 Add `error-first-day-in-past` to `server/locales/en.ftl` and `server/locales/de.ftl` with the texts of design.md, next to `error-last-day-before-first-day`. Verify the catalog tests in `client/src/l10n/` pass.
- [x] 2.2 Map `firstDayInPast` to `error-first-day-in-past` in `message` of `client/src/routes/_organizer/gatherings/-components/GatheringForm.tsx`. Verify with a test in `client/src/routes/_organizer/gatherings/gatherings.integration.test.tsx` where the mocked `POST */api/gatherings` answers 422 with `firstDayInPast` on `firstDay`: the first day is marked `aria-invalid`, has focus and its description reads "The first day cannot lie in the past.".

## 3. Calendars and moving last day

- [x] 3.1 Let `DatePicker` in `client/src/components/DatePicker.tsx` disable days before `firstDay` and after `lastDay` each on its own and never the day of `value`. Verify the existing tests of `ResponseForm` still pass.
- [x] 3.2 In `GatheringForm`, pass today by the browser's date as `firstDay` to the calendar of the first day and the chosen first day as `firstDay` to the calendar of the last day. Verify with tests in `gatherings.integration.test.tsx` under a pinned date: in the calendar of the first day the day before today is disabled and today is enabled, the stored past first day of a running gathering opened at `/gatherings/1/edit` is enabled, and in the calendar of the last day every day before the chosen first day is disabled and the first day itself is enabled.
- [x] 3.3 In `GatheringForm`, move the last day by the difference in calendar days when the first day changes while both days are set, using `differenceInCalendarDays` and `addDays` from `date-fns`. Verify with tests under a pinned date: first day 12 March and last day 14 March, changing the first day to 15 March shows 17 March as the last day, and choosing a first day while it is empty leaves a set last day as it was.
- [x] 3.4 Pin the date in the test "marks a last day before the first day" and pick the last day before the first day, so it still reaches `beforeFirstDay` on every run date. Verify the test passes.

## 4. Documentation

- [x] 4.1 Add to the paragraph on the time zone of the instance in `docs/arc42/08-cross-cutting-concepts.md` that the calendars of the client may use the browser's date to disable days while the server alone decides whether a day lies in the past. Verify the paragraph stays one line and its links resolve.
