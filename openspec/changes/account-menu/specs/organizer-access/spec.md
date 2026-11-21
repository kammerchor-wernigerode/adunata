## ADDED Requirements

### Requirement: Account menu
With a session, the client SHALL show one account menu at the end of the header, after the navigation links. Its trigger SHALL show the organizer's first and last name and stay on one line, cutting a long name off with an ellipsis. The menu SHALL hold, in this order, a link to the organizer's own member entry, the language choice, a separator and sign-out. It SHALL work with the keyboard alone. Without a session, the header SHALL show no account menu.

#### Scenario: Open the menu
- **WHEN** a signed-in organizer named Ada Lovelace opens the account menu
- **THEN** the trigger reads "Ada Lovelace" and the menu shows, in this order, the own member entry, the languages Deutsch and English, a separator and sign-out

#### Scenario: Own member entry
- **WHEN** an organizer chooses the own member entry in the account menu
- **THEN** the client shows the page of that organizer's member entry

#### Scenario: Keyboard only
- **WHEN** an organizer focuses the trigger, opens the menu with Enter, moves to sign-out with the arrow keys and presses Enter
- **THEN** the client signs the organizer out

#### Scenario: Header on a phone
- **WHEN** a signed-in organizer views a page 390 px wide with the text at 200 %
- **THEN** the trigger stays on one line and the page does not scroll sideways

#### Scenario: No session
- **WHEN** a person without a session views the sign-in page
- **THEN** the header shows no account menu and no sign-out

### Requirement: Sign-out in the account menu
The client SHALL offer sign-out only as the last entry of the account menu, after a separator, and SHALL not ask for confirmation. A sign-out SHALL end the session, show the sign-in page and clear the data the client cached. When sign-out fails, the client SHALL name the failure in the header next to the trigger, so the message stays visible after the menu closes.

#### Scenario: Sign out from the menu
- **WHEN** a signed-in organizer opens the account menu and chooses sign-out
- **THEN** Adunata ends the session, the client shows the sign-in page and the header shows neither navigation nor account menu

#### Scenario: Sign-out not in the navigation row
- **WHEN** a signed-in organizer views any page with the account menu closed
- **THEN** no sign-out control is visible in the header

#### Scenario: Sign-out fails
- **WHEN** an organizer chooses sign-out and the server answers with an error
- **THEN** the menu closes, the organizer stays on the page and the header shows "Something went wrong. Try again." next to the trigger
