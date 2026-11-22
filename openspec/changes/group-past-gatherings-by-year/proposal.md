# Proposal

## Why

The gathering list shows every past gathering in one flat list. An instance with 112 past gatherings from ten years renders a page about 8,150 pixels high, and the year appears only inside the grey date line of each row. An organizer looks for a past gathering by its year but has to scroll through every later year to reach it.

## What Changes

- The section "Past" of the gathering list groups its gatherings by the year of their first day, latest year first.
- Each year is a section with the year as a heading of level 3 and the number of its gatherings. The organizer opens and closes each year on its own, and several years can be open at once.
- The client remembers on the device which years the organizer opened or closed, so a return to the list shows the same years open. A year without a remembered choice starts open only if it is the latest year with past gatherings, so on a first visit only that year is open.
- A row of a gathering stays as it is, with its link, its date line including the year, its location and the badge for a cancelled gathering.
- The upcoming gatherings, the empty message of "Past" and the API stay unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `gatherings`: the requirement "Gathering list" groups past gatherings by year in sections the organizer can open and close, and the client remembers them on the device.

## Impact

- Client: `client/src/routes/_organizer/gatherings/index.tsx` and the components in `client/src/routes/_organizer/gatherings/-components/`.
- Catalogs: one new message for the count of gatherings in `server/locales/en.ftl` and `server/locales/de.ftl`.
- Tests: the client integration test of the gathering list in `client/src/routes/_organizer/gatherings/gatherings.integration.test.tsx`.
- Browser: one new entry in `localStorage` that holds the remembered years.
- No change to the server, the API, the database or dependencies. The shadcn component `Collapsible` is already installed.
