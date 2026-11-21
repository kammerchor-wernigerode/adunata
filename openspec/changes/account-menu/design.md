# Design

## Context

`RootLayout` renders the header band with the product name, `OrganizerNavigation` and `LanguageSwitcher` side by side. `OrganizerNavigation` returns nothing without a session and otherwise renders a `<nav aria-label="Main navigation">` with the links Gatherings and Members and a link-styled sign-out button. Its `leave` function signs out through `useSubmission`, navigates to `/sign-in`, resets the session query and removes inactive queries. A failure shows `error-unexpected` in a `role="alert"` span next to the button. `LanguageSwitcher` is a shadcn `NativeSelect` with a label and calls `setLocale` from `useLocale`, which stores the choice on the device. `useOrganizer` returns the signed-in `Member` with `id`, `firstName` and `lastName`, `null` without a session and `undefined` while the session loads. The client depends on `radix-ui` and `lucide-react` but has no `dropdown-menu` in `client/src/components/ui/`. See proposal.md for the motivation.

## Goals / Non-Goals

**Goals:**
- One account menu for the signed-in organizer that holds the own member entry, the language and sign-out.
- Sign-out keeps its current behavior and only moves.

**Non-Goals:**
- Any change to the API, the session, the server or the stored language of mail.
- Other places that switch a language, and page actions, which issue #61 covers.
- A confirmation dialog, a toast component or initials or an avatar in the trigger.

## Decisions

### Decisions taken in the issue

These come from the issue and stay as written.

- Build the menu from the shadcn component `dropdown-menu`, added with `pnpm exec shadcn add dropdown-menu` in `client/` as the README describes, and use its radio group items for the language, because the project forbids styling native elements. It builds on `radix-ui`, which the client already depends on (MIT).
- Put sign-out last and set it apart with a separator, because that prevents the accidental click.
- Add no confirmation dialog, because the second click in the menu is enough.
- Keep the language select in the header for people without a session, because they have no account menu and expect a language switch there.
- Record no architecture decision, because an operator or a running instance does not notice this change.

### Answers to the open questions of the issue

- A failed sign-out names its failure in a `role="alert"` region next to the trigger in the header, outside the menu. The menu content unmounts when it closes, so the failure state and the always-mounted alert element live in the component that renders the trigger. The client has no toast component, and an alert region matches how the client reports failures elsewhere. Alternative considered: a toast, rejected because it adds a component and a dependency for one message.
- The trigger shows first and last name on every width. On narrow screens it truncates with an ellipsis (`truncate` with a `max-w`) instead of wrapping, and the full name stays the accessible name. Alternative considered: only the first name on narrow screens, rejected because it needs a second rendering for one breakpoint and loses nothing the ellipsis does not already solve.

### Further decisions

- A new component `AccountMenu` in `client/src/components/AccountMenu.tsx` renders the trigger, the menu and the failure alert. It takes over `leave` and `useSubmission` from `OrganizerNavigation`, which keeps only the two links. Alternative considered: putting the menu inside `OrganizerNavigation`, rejected because a file holds one component.
- The account menu sits outside `<nav aria-label="Main navigation">`, at the end of the header row. The navigation landmark then holds only navigation links, and the menu is an account control, not a page link.
- `RootLayout` decides by `useOrganizer()`. A `Member` renders `OrganizerNavigation` and `AccountMenu`. `null` renders `LanguageSwitcher`. `undefined`, while the session loads, renders neither the menu nor the select, so the header does not show a select that disappears a moment later.
- The own member entry is a `DropdownMenuItem` with `asChild` around a TanStack Router `Link` to `/members/$memberId` with the organizer's `id`. Its text is a new catalog entry `account-own-entry` ("My member entry", "Mein Mitgliedseintrag").
- The language group is a `DropdownMenuLabel` with the existing `language-label` and a `DropdownMenuRadioGroup` bound to `useLocale`. The item texts come from the existing catalog entries `language-name-de` and `language-name-en`, each item with its `lang` attribute. `LanguageSwitcher` switches to the same entries and drops its local `languageNames` map, so each name has one home. Choosing a language closes the menu like any other item. `LocalizationProvider` renders nothing while the catalog of a newly chosen language loads, so the menu would unmount anyway, and keeping it open would need a change to the provider for no gain.
- Sign-out is a `DropdownMenuItem` that calls `leave` on select, disabled while the request runs, after a `DropdownMenuSeparator`. The existing entry `nav-sign-out` keeps its text.
- The trigger is a shadcn `Button` with the organizer's name and a `ChevronDown` icon from `lucide-react` that is `aria-hidden`. Its accessible name comes from a new entry `account-menu-label` with a placeholder, "Account of { $name }" and "Konto von { $name }", through `aria-label`, so a screen reader announces what the button opens.

### Architecture significance

None of these decisions is architecturally significant, because an operator or a running instance does not notice how the header arranges its controls. The change therefore drafts no record.

### Affected arc42 sections

- `docs/arc42/08-cross-cutting-concepts.md`, section Localization: "A switch next to the title changes it" becomes a switch in the account menu for a signed-in organizer and in the header otherwise.
- `docs/arc42/08-cross-cutting-concepts.md`, section Pages and forms: the header description adds that organizer pages end the band with an account menu that holds the own member entry, the language and sign-out last.

## Risks / Trade-offs

- [Every page renders the header, so a broken menu breaks every organizer page] → The layout and sign-in integration tests cover the menu with and without a session, and axe runs with the menu open.
- [jsdom computes no layout, so the ellipsis is not testable there] → The end-to-end test of the organizer checks the header at 390 px through the existing reflow helper in `client/e2e/invitation.e2e.ts`.
- [The shadcn CLI may change `pnpm-lock.yaml`] → The builder updates `client/pnpm-deps.hash` as the README describes under the hash of client dependencies.
- [Two clicks instead of one for every sign-out] → Accepted, it is the purpose of the change.
