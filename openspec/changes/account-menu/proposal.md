# Proposal

## Why

An organizer signs out with one click on a link button that sits in the header row between the navigation links and the language select. Organizers have signed out by accident more than once. Moving sign-out into an account menu, last and set apart, makes it take two deliberate clicks.

## What Changes

- With a session, the header shows one account menu at the top right, after the links Gatherings and Members. Its trigger shows the organizer's first and last name and stays on one line on narrow screens.
- The menu holds, in this order, a link to the organizer's own member entry, the language as a single choice between Deutsch and English, a separator and sign-out.
- Choosing a language in the menu works like the language select does today. It changes only the language of the client, not the language stored for the member's mail.
- Sign-out leaves the navigation row and moves into the menu. It behaves as before and asks for no confirmation.
- A failed sign-out shows its message next to the trigger in the header, so it stays visible after the menu closes.
- With a session, the header no longer shows the separate language select. Without a session, it shows the select as today.
- New texts come from the German and English catalogs.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `organizer-access`: adds the account menu with the own member entry and sign-out, and where a failed sign-out is reported.
- `localization`: the language of the client is switched in the account menu with a session and in the header select without one.

## Impact

- Client: `client/src/components/RootLayout.tsx`, `client/src/components/OrganizerNavigation.tsx`, `client/src/l10n/LanguageSwitcher.tsx`, a new account menu component and the shadcn component `dropdown-menu` under `client/src/components/ui/`.
- Catalogs: `server/locales/en.ftl` and `server/locales/de.ftl`.
- Tests: client integration tests of the navigation, sign-in and localization, and the end-to-end test of the organizer.
- Documentation: `docs/arc42/08-cross-cutting-concepts.md`.
- No change to the API, the session or the server code.
