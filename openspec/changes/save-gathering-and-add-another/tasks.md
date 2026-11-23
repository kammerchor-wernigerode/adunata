## 1. Labels and component

- [ ] 1.1 Add the labels for the second button ("Save & add another", "Speichern & Weiter") and for the line with the variables title and span to `server/locales/en.ftl` and `server/locales/de.ftl`
- [ ] 1.2 Add the shadcn component `alert` to `client/src/components/ui/` with the command line tool of shadcn

## 2. Form

- [ ] 2.1 In `GatheringForm.tsx`, show the second button after "Save" only without the prop `gathering`, disabled while a save runs like "Save"
- [ ] 2.2 After a successful create through the second button, invalidate the queries as today, reset the form to the stored title, location and description with empty days, focus the title and show the line with `role="status"`, the title linked to the page of the gathering and the span through `formatSpan`
- [ ] 2.3 Clear the line on every refusal or unexpected error

## 3. Tests

- [ ] 3.1 Add tests under `gathering form` in `client/src/routes/_organizer/gatherings/gatherings.integration.test.tsx` for each scenario of the requirement "Save and add another gathering", including axe on the form with the line shown
