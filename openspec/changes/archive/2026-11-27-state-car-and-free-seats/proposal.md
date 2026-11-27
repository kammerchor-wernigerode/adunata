## Why

An organizer plans rides to shared activities of a gathering. Adunata does not record who arrives by car or how many seats a car has free, so the organizer has to ask each member.

## What Changes

- A member who accepts or accepts tentatively answers whether they arrive by car. The answer is required, like the question about beds.
- A member who arrives by car states the free seats, not counting the driver, as a whole number from 0 to 8. 0 still counts as a car.
- A decline clears car and free seats, like period and beds. The database enforces it.
- A response stored before this change has no statement about a car. Its form opens with the question unanswered, and saving it as accepted or tentative requires an answer.
- An organizer enters car and free seats for a member with the same form.
- The confirmation and the read-only view of a member's own response show the car and the free seats.
- Other members never see car or free seats. The response page says at the question that only organizers see them.
- The overview gets a column for the car with the free seats, plus the number of cars and the total of free seats.
- The CSV file gets the column free seats right before the comment. **BREAKING** for anyone who reads the file by column position.

Out of scope: assigning passengers to drivers, a statement that a member needs a seat, showing cars or seats to other members, and vehicles with more than 8 free seats.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `responses`: new requirement "Car and free seats", and changes to "Response status", "Confirmation", "Attendees visible to members", "Notice on the response page", "Overview for organizers", "Columns of the CSV file" and "Language of the CSV file".

## Impact

- Database: migration `server/migrations/0006_*.sql` adds two columns and their check constraints to the table `responses`.
- Server: `server/src/responses.rs` (values, validation, storage, overview rows, totals, CSV), `server/locales/en.ftl` and `server/locales/de.ftl`.
- API: the schemas `ResponseValues`, `OverviewRow` and `Totals` gain fields, so `client/openapi.json` and `client/src/api/generated/` are regenerated.
- Client: `client/src/components/ResponseForm.tsx`, `client/src/routes/invitation/index.tsx`, `OverviewTable.tsx` and `Overview.tsx` under `client/src/routes/_organizer/gatherings/$gatheringId/-components/`.
- Documentation: `docs/arc42/12-glossary.md`, `docs/arc42/06-runtime-view.md` and `docs/arc42/01-introduction-and-goals.md`.
