# Proposal

## Why

Page actions sit in a different place on each organizer page. "Create gathering" sits on its own line below the title of the gathering list. "Add member" sits next to the search field of the member list. "Edit gathering" sits to the right of span and location on the gathering page. An organizer has to look for the action on each page. The overview table of a gathering also shows the mail notice and "Resend" inside the name cell of some rows only, so row heights vary and the table looks restless.

## What Changes

- The main heading of a page can carry one page action, which sits at the top right on the line of the heading. When both do not fit on one line, the action wraps below the heading.
- "Create gathering", "Add member" and "Edit gathering" move into that place. "Edit gathering" stays hidden for a gathering that is cancelled or has ended.
- The overview table of a gathering gets a last column "Actions". The notice about a failed mail or a revoked link moves there together with "Resend". The name cell holds only the linked name.
- In the card layout of the overview below a container width of 40rem, the actions are one more labelled line of the card. A row with an empty actions cell shows no such line.
- A new catalog message names the column "Actions" in every language.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `gatherings`: the actions to create a gathering and to edit a gathering sit in the page header.
- `member-roster`: the action to add a member sits in the page header.
- `responses`: the overview for organizers shows the mail notice and "Resend" in a last column "Actions".

## Impact

- Client: `client/src/components/PageTitle.tsx`, the routes `client/src/routes/_organizer/gatherings/index.tsx`, `client/src/routes/_organizer/members/index.tsx` and `client/src/routes/_organizer/gatherings/$gatheringId/index.tsx`, and `OverviewTable.tsx` and `MailNotice.tsx` in `client/src/routes/_organizer/gatherings/$gatheringId/-components/`.
- Catalogs: `server/locales/en.ftl` and `server/locales/de.ftl` get the message `overview-column-actions`.
- Tests: the client integration tests of the gathering list, the member list, the gathering page and the overview.
- Documentation: section 8 of arc42, "Pages and forms".
- No change to the API, the server logic, the database or dependencies.
