# Design

## Context

`client/src/components/PageTitle.tsx` renders the browser title and the `h1` that the root layout focuses after a route change. Every route uses it, including sign-in, the invitation page and `RouteError.tsx`. It has no place for an action, so each of the three routes with a page action lays out its button on its own.

`OverviewTable.tsx` renders `MailNotice.tsx` inside the name cell. `MailNotice` returns nothing for a row whose mail did not fail and whose link is active, and otherwise renders the notice, the button "Resend" while the gathering is open and an element with `role="alert"` for a failed resend. Each cell starts with a label that is `sr-only` in the table and visible in the card layout below 40rem.

## Goals / Non-Goals

**Goals:**

- One header layout for every page action, owned by `PageTitle`.
- Even row heights in the overview, with the notice and "Resend" in their own column.

**Non-Goals:**

- The order and style of form actions.
- A bulk action that resends every failed invitation.
- Actions on the invitation page.
- "Invite members" and "Cancel gathering" on the gathering page. They belong to the section "Invitations" and are not page actions.
- Moving the search field of the member list, or span, location, description and the cancelled notice of the gathering page out from below the header.
- Changing the variant of a button. "Edit gathering" stays `outline`, the other two stay default.

## Decisions

### Decisions taken in the issue

1. Every page action (create, add, edit) sits at the top right, on the same line as the page title, so an organizer finds it in one fixed place.
2. Form actions stay at the bottom left of the form, because they end the form and are already consistent.
3. The overview table gets a last column "Actions" that holds "Resend", which keeps row heights even while the action stays one click away.
4. The notice about a failed mail or a revoked link moves into the "Actions" column with the button, so the name cell holds only the name.
5. In the card layout below 40rem, the actions are one more labelled line of the card, and a row without an action shows no such line, because an empty line adds noise.
6. `PageTitle` takes an optional action next to the heading, because the three pages need the same header layout.
7. Every control comes from shadcn. A missing control is added with the shadcn command line tool. This change needs no new control, because `Button` and `Table` exist.

### Wrapping title

When heading and action do not fit on one line, the action wraps below the heading. `PageTitle` wraps both in a flex container with `flex-wrap`, `items-start` and `justify-between`. The heading keeps `min-w-0` and `break-words`, so a long word still breaks. The member list and the gathering page already let their action row wrap the same way. The container takes over the bottom margin of today's heading, so a page without an action looks as it does today.

### Action as a prop beside the heading

`PageTitle` takes an optional prop `action` of type `ReactNode` and renders it as a sibling of the `h1`, never inside it. The accessible name of the heading and the browser title stay the title alone, and focus after a route change still lands on the heading. An alternative was a separate `PageHeader` component, but that would mean a second component that every page with an action must combine with `PageTitle`.

### Actions cell for every row

Every row keeps six cells, so the table stays a valid grid. `MailNotice` renders the whole content of the actions cell including its label, and returns nothing for a row without a notice. The actions cell gets `@max-[40rem]:empty:hidden`, so an empty cell shows no line in the card layout. A row with a notice but no button, as on an ended or cancelled gathering, still renders the notice and keeps its line. The alternative of computing in `OverviewTable` whether the row has a notice would duplicate the condition that `MailNotice` already holds.

### Architectural significance

None of these decisions is architecturally significant, because each only changes the layout of client pages and no operator or running instance depends on it. The change drafts no decision record.

### arc42

- Section 8, "Pages and forms" in `docs/arc42/08-cross-cutting-concepts.md`: add that a page action sits at the top right on the line of the main heading and wraps below it on a narrow screen, while form actions end the form.

No other section changes. The building blocks, interfaces and deployment stay as they are.

## Risks / Trade-offs

- [`PageTitle` is shared by every page] → The prop is optional and a page without it renders the same heading with the same spacing. The existing layout test for heading focus and tab title keeps running.
- [Top right and wrapping are layout that jsdom cannot measure] → Integration tests assert structure: the action follows the heading and sits outside it. The e2e reflow check at 390 px already covers the gathering page with its overview. Code review checks the classes.
- [The existing test expects five labelled cells per row] → It changes to expect six labels for a row with a notice and an empty last cell for a row without.
