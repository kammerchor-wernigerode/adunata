## 1. Server

- [ ] 1.1 Add migration `server/migrations/0006_state_car_and_free_seats.sql` with the columns `car` and `free_seats` and their check constraints, and test that the database refuses a declined response with a car and a car of false with a count
- [ ] 1.2 Add `car` and `freeSeats` to `ResponseValues` in `server/src/responses.rs`, with normalization, validation, storage and reading, and integration tests for each scenario of the requirement "Car and free seats" and for the decline that clears the car
- [ ] 1.3 Add `car` and `freeSeats` to `OverviewRow` and `cars` and `freeSeats` to `Totals`, with a test of the scenario "Plan rides"
- [ ] 1.4 Add the CSV column free seats before the comment and update the CSV tests to the new rows and headings
- [ ] 1.5 Add the server keys `overview-csv-free-seats` to `server/locales/en.ftl` and `server/locales/de.ftl`
- [ ] 1.6 Test that the page data of a personal link holds no car or free seats of another member
- [ ] 1.7 Regenerate `client/openapi.json` and `client/src/api/generated/` as `CONTRIBUTING.md` describes

## 2. Client

- [ ] 2.1 Add the car question, the free seats field, the hint and the validation to `client/src/components/ResponseForm.tsx`, and show the car in the confirmation
- [ ] 2.2 Show the car in the read-only view `yours` in `client/src/routes/invitation/index.tsx`
- [ ] 2.3 Add the column car to `OverviewTable.tsx` and the totals cars and free seats to `Overview.tsx`
- [ ] 2.4 Add the client keys of design.md in English and German
- [ ] 2.5 Extend `client/src/routes/invitation/invitation.integration.test.tsx` and `overview.integration.test.tsx` with the form, the hint, the unanswered question of an old response, the confirmation and the overview cells "no car" and "–"

## 3. Documentation

- [ ] 3.1 Update `docs/arc42/12-glossary.md`, `docs/arc42/06-runtime-view.md` and `docs/arc42/01-introduction-and-goals.md` as design.md describes
