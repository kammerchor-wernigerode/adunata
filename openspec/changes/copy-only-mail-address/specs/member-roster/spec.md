## MODIFIED Requirements

### Requirement: Member list
Adunata SHALL show organizers the members sorted by last name, with a filter for active and inactive members and a search by name or mail address. Each row SHALL offer a button that copies only the member's mail address to the clipboard, without name and without angle brackets. The page SHALL announce the result of a copy in a status region of its own, separate from the member count.

#### Scenario: Search
- **WHEN** an organizer searches for part of a last name
- **THEN** the list shows only members whose name or mail address contains that text, ignoring case

#### Scenario: Copy a mail address
- **WHEN** an organizer clicks the button "Copy mail address" in the row of Ada Lovelace with the mail address `ada@example.org`
- **THEN** the clipboard holds exactly `ada@example.org`, the button's accessible name is "Mail address copied" and the copy status region reads "Mail address copied"

#### Scenario: Confirmation ends
- **WHEN** two seconds have passed since a successful copy
- **THEN** the button shows the copy icon again, its accessible name is "Copy mail address" and the copy status region is empty

#### Scenario: Name with a comma
- **WHEN** an organizer copies the mail address of a member with first name `Ada`, last name `Lovelace, Countess` and mail address `ada@example.org`
- **THEN** the clipboard holds exactly `ada@example.org`

#### Scenario: Name with a double quote
- **WHEN** an organizer copies the mail address of a member with first name `Ada "Bird"`, last name `Lovelace` and mail address `ada@example.org`
- **THEN** the clipboard holds exactly `ada@example.org`

#### Scenario: Inactive member
- **WHEN** an organizer opens the member list and it shows an inactive member
- **THEN** the row of the inactive member offers the button "Copy mail address" too

#### Scenario: Copy fails
- **WHEN** an organizer clicks "Copy mail address" and the browser offers no clipboard or rejects the write, for example outside a secure context
- **THEN** the copy status region reads "Could not copy the mail address. Select and copy it by hand." and the button keeps the copy icon

#### Scenario: Member count stays
- **WHEN** an organizer copies a mail address while the list shows three members
- **THEN** the status region of the member count still reads "3 members"
