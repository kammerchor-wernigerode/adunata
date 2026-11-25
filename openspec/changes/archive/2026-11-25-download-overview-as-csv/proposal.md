# Proposal

## Why

Organizers plan rooms and meals in a spreadsheet and copy each value of the overview by hand. The overview shows sentences such as "needs 2 beds" and a dash for an empty value, which a spreadsheet cannot sort or sum.

## What Changes

- A new endpoint of the module `responses` returns the overview of one gathering as a CSV file, behind an organizer session.
- The file holds every invitation of the gathering, one plain value per cell, with headings and statuses in the language of the organizer's interface.
- The file guards against CSV injection and keeps line breaks, commas and double quotes of names and comments intact.
- The overview page gets a download link with a notice that the file holds personal data and that Adunata does not delete it.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `responses`: adds the download of the overview as a CSV file and the notice at its link.

## Impact

- `server/src/responses.rs`: a new route `GET /api/gatherings/{id}/overview.csv` next to `overview`, reusing `build_overview`.
- `server/locales/en.ftl` and `server/locales/de.ftl`: column headings, the link and its notice.
- `client/src/routes/_organizer/gatherings/$gatheringId/-components/`: a new component for the link and its notice, placed in `Overview.tsx`.
- `client/src/api/generated/`: regenerated from the OpenAPI document.
- No new dependency and no change to the database.
- arc42 sections 03, 05 and 08 name the new output.
