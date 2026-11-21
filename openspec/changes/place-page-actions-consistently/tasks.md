# Tasks

## 1. Page header

- [x] 1.1 Give `client/src/components/PageTitle.tsx` an optional prop `action` that renders beside the `h1` in a wrapping flex container, as design.md describes. Verify that the test "names the tab and moves focus to the main heading after a route change" in `client/src/routes/layout.integration.test.tsx` still passes
- [x] 1.2 Move "Create gathering" in `client/src/routes/_organizer/gatherings/index.tsx` into the `action` of `PageTitle`. Verify with a test in `client/src/routes/_organizer/gatherings/gatherings.integration.test.tsx` that the link follows the main heading and is not inside it
- [x] 1.3 Move "Add member" in `client/src/routes/_organizer/members/index.tsx` into the `action` of `PageTitle` and leave the search field below. Verify with a test in `client/src/routes/_organizer/members/members.integration.test.tsx` that the link follows the main heading, is not inside it and comes before the search field
- [x] 1.4 Move "Edit gathering" in `client/src/routes/_organizer/gatherings/$gatheringId/index.tsx` into the `action` of `PageTitle`, only while the gathering is neither cancelled nor ended. Keep span, location, description and the cancelled notice below. Extend the test "opens the gathering with its span and the action to edit it" in `client/src/routes/_organizer/gatherings/gatherings.integration.test.tsx` to check that the link follows the main heading and is not inside it. Add a test there that the ended gathering of the fixture shows no such link
- [x] 1.5 Add to "Pages and forms" in `docs/arc42/08-cross-cutting-concepts.md` that a page action sits at the top right on the line of the main heading and wraps below it on a narrow screen. Verify that the section states it once and that its links resolve

## 2. Actions column of the overview

- [ ] 2.1 Add `overview-column-actions = Actions` to `server/locales/en.ftl` and `overview-column-actions = Aktionen` to `server/locales/de.ftl`. Verify that the tests in `server/src/localization.rs` pass
- [ ] 2.2 Add `actions` as the last column of `OverviewTable.tsx` and render `MailNotice` in its cell instead of the name cell. `MailNotice` renders the label "Actions" itself and the cell gets `@max-[40rem]:empty:hidden`, as design.md describes. Verify that the name cell holds only the link
- [ ] 2.3 Update the tests in `client/src/routes/_organizer/gatherings/$gatheringId/overview.integration.test.tsx`. The resend tests find the notice and "Resend" in the last cell of the row. The column test expects the labels Name, Status, Period, Beds, Comment and Actions for a row with a notice and an empty last cell for a row without. New tests show a revoked link in the last cell and the notice without "Resend" on an ended gathering. Verify that all of them pass
