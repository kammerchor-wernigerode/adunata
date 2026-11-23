## 1. Database

- [x] 1.1 Add `server/migrations/0003_count_changes_of_gatherings.sql` with `version` and `changed_at` as the design describes, plus a test that an existing gathering gets `version` 0 and `changed_at` equal to `created_at`
- [x] 1.2 Raise `version` and set `changed_at` in `change` and `cancel` in `server/src/gatherings.rs`, with tests for one change, two changes and a cancellation

## 2. Calendar file

- [x] 2.1 Add the module `calendar` with SHA-1, the UUID of version 5 and the namespace constant, tested against FIPS 180-4 vectors and the UID of the delta spec
- [x] 2.2 Write the file: properties in order, escaping, CRLF and folding at 75 octets on UTF-8 boundaries, with unit tests for each and for the example of the delta spec
- [x] 2.3 Add `GET /api/invitation/calendar` to `server/src/responses.rs` with the media type, `Content-Disposition` and 404 cases, plus integration tests for accepted, tentative on a cancelled gathering, declined, pending, ended and no secret
- [x] 2.4 Add `calendar` to `InvitationPage`, the route to `PUBLIC` in `server/src/api.rs` and regenerate `client/openapi.json`

## 3. Client

- [x] 3.1 Show the shadcn `Button` "Add to calendar" and the sentence about later changes on the response page when `calendar` is true, and save the fetched file as `gathering.ics`
- [x] 3.2 Add the strings to the English and German catalogs
- [x] 3.3 Extend `client/src/routes/invitation/invitation.integration.test.tsx` for button shown, button hidden and the download request carrying the secret

## 4. Documentation

- [x] 4.1 Write `docs/decisions/0025-stable-calendar-uids-for-gatherings.md` from the draft in design.md
- [x] 4.2 Update arc42 sections 05 and 08 and add ADR 0025 to the table in 09
