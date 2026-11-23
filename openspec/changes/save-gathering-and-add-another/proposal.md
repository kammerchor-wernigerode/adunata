## Why

An organizer plans the gatherings of a year in one sitting. After each save, the form opens the page of the new gathering, so the organizer goes back to the list, opens the form again and types title, location and description anew. Gatherings repeat. One instance holds 112 past gatherings that share 13 titles.

## What Changes

- The form for a new gathering offers a second button "Save & add another" ("Speichern & Weiter") next to "Save". The form that changes a gathering does not show it.
- The second button stores the gathering and keeps the form open. Title, location and description stay. First day and last day become empty, and focus moves to the title.
- After such a save, the form shows a line that names the title and the span of the stored gathering, links to its page and is announced to screen readers. It stays until the next save or refusal replaces it.
- "Save" stays the first button and the one Enter triggers, and opens the page of the gathering as today.
- Both buttons are disabled while a save runs. A refused save behaves as today with either button.
- The client gains the shadcn component `alert` for the line.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `gatherings`: the form for a new gathering can store a gathering and stay open for the next one.

## Impact

- `client/src/routes/_organizer/gatherings/-components/GatheringForm.tsx` and its tests in `client/src/routes/_organizer/gatherings/gatherings.integration.test.tsx`.
- New `client/src/components/ui/alert.tsx`, added by the command line tool of shadcn.
- Two new labels in `server/locales/de.ftl` and `server/locales/en.ftl`. The API and the server code do not change.
