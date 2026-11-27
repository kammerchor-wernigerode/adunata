## Context

A response holds status, period, beds and comment. The table `responses` in `server/migrations/0002_invite_members_to_gatherings.sql` stores beds as `beds` and `bed_count` with check constraints. `server/src/responses.rs` holds `ResponseValues`, their normalization and validation, the upsert, the overview rows, the totals and the CSV columns. `client/src/components/ResponseForm.tsx` is the form for members and organizers. `OverviewTable.tsx` and `Overview.tsx` under `client/src/routes/_organizer/gatherings/$gatheringId/-components/` show rows and totals. `client/src/routes/invitation/index.tsx` shows the read-only view through its function `yours`. The car follows the path of beds through every one of these places.

## Goals / Non-Goals

**Goals:** record car and free seats per response, show them only to organizers, total them for ride planning.

**Non-Goals:** assigning passengers to drivers, a statement that a member needs a seat, showing cars or seats to other members, more than 8 free seats.

## Decisions

Decisions of the issue, kept as they are:
- The question applies to accepted and tentative responses and is required, like beds.
- The count is the free seats without the driver, a whole number from 0 to 8. 0 means a full car.
- A member cannot state that they need a seat.
- A declined response has no car and no seats.
- A response stored before this change has no statement until someone changes it.
- Only organizers see car and seats.
- The overview shows car and seats per invitation, the number of cars and the total of free seats.
- The CSV file gets one column for the free seats before the comment, empty without a car or a statement.
- Organizers use the same form.

Decisions of intake, kept as they are:
- The response page states at the car question that only organizers see car and seats.
- The overview shows "no car" for a member without a car and a dash for a response without a statement. Neither adds to a total.
- The confirmation and the read-only view show car and seats.

Decisions made here:
- **Storage.** Migration `0006_state_car_and_free_seats.sql` adds `car BOOLEAN` and `free_seats SMALLINT CHECK (free_seats BETWEEN 0 AND 8)` to `responses`. A check keeps both null for a declined response, and a check ties `free_seats IS NOT NULL` to `car IS TRUE`. Existing rows get null, which means no statement. A boolean fits a yes-or-no question better than a text enum like `beds`. Not significant, because an operator only sees a migration that runs on start as every migration does.
- **API.** `ResponseValues` and `OverviewRow` gain `car: boolean | null` and `freeSeats: number | null`. `Totals` gains `cars` and `freeSeats`. Missing answer gives `required` on `car`, missing count gives `required` on `freeSeats`, and a count outside 0 to 8 gives `invalid` on `freeSeats`. `invalid` fits a negative count, which `tooLarge` would misname. Not significant, because the API serves only the embedded client.
- **Normalization.** Like beds, a decline clears car and seats, and a response without a car drops any sent count instead of rejecting it.
- **Form.** A radio group "Do you come by car?" with "Yes, by car" and "No car", below beds, and a number field for the free seats when yes is chosen. The hint "Only organizers see your car and free seats." sits under the legend. The form keeps the radio group unselected when the stored `car` is null.
- **Labels.**

|Key|English|German|
|-|-|-|
|`response-car`|Do you come by car?|Kommst du mit dem Auto?|
|`response-car-yes`|Yes, by car|Ja, mit dem Auto|
|`response-car-no`|No car|Kein Auto|
|`response-free-seats`|How many seats are free, not counting you?|Wie viele Plätze sind frei, ohne dich?|
|`response-car-hint`|Only organizers see your car and free seats.|Nur Organisatoren sehen dein Auto und die freien Plätze.|
|`response-error-car`|Choose whether you come by car.|Wähle, ob du mit dem Auto kommst.|
|`response-error-free-seats`|State the free seats, from 0 to 8.|Gib die freien Plätze an, von 0 bis 8.|
|`overview-column-car`|Car|Auto|
|`overview-car-none`|no car|kein Auto|
|`overview-free-seats`|{ $count } free seats, "one free seat" for 1|{ $count } freie Plätze, "ein freier Platz" für 1|
|`overview-cars`|Cars|Autos|
|`overview-free-seats-total`|Free seats|Freie Plätze|
|`overview-csv-free-seats`|Free seats|Freie Plätze|

  The confirmation and the read-only view reuse `overview-free-seats` and `overview-car-none`, as the confirmation reuses `overview-beds-*` today.
- **Read-only view.** `yours` in `client/src/routes/invitation/index.tsx` adds the car after the status when the stored response holds a statement. It shows no beds today, and this change does not add them.

No decision is architecturally significant, so the change drafts no record.

## arc42

- `docs/arc42/12-glossary.md`: the entry "Response" names the car, and a new entry "Car" explains the statement and its free seats.
- `docs/arc42/06-runtime-view.md`: the step where the member chooses the values names the car.
- `docs/arc42/01-introduction-and-goals.md`: the sentence about what members reply names whether they come by car.

## Risks / Trade-offs

- [Readers of the CSV by column position break] → The heading row names every column, and the change notes it as breaking.
- [Responses from before the change show a dash until changed] → Intended by the issue. The organizer can tell them from "no car".
