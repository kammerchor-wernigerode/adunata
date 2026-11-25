# member-roster Specification

## Purpose
Keeps the list of ensemble members that organizers invite, with only the data that invitations need.

## Requirements

### Requirement: Member data
A member SHALL have a first name, a last name, a mail address, a language and an active flag. The mail address SHALL be unique among members, ignoring case. Adunata SHALL store no other personal data about a member in this change.

#### Scenario: Add a member
- **WHEN** an organizer adds a member with first name, last name, mail address and language
- **THEN** Adunata stores the member as active

#### Scenario: Duplicate address
- **WHEN** an organizer adds a member whose mail address differs from an existing one only in case
- **THEN** Adunata rejects the member and names the conflicting address

### Requirement: Edit members
An organizer SHALL be able to change every field of a member.

#### Scenario: Change the mail address
- **WHEN** an organizer changes a member's mail address
- **THEN** Adunata sends future mail to the new address

### Requirement: Active and inactive members
An organizer SHALL be able to deactivate a member and to activate the member again. Adunata SHALL offer only active members for new invitations.

#### Scenario: Deactivate a member
- **WHEN** an organizer deactivates a member
- **THEN** the member stays in the list as inactive and is not invited to new gatherings

### Requirement: Delete members
An organizer SHALL be able to delete a member. Adunata SHALL then delete the member together with all of the member's invitations, responses, sessions, sign-in links and mail in the outbox.

#### Scenario: Delete a member
- **WHEN** an organizer deletes a member
- **THEN** the member, the member's invitations, responses, sessions and queued mail are gone and the member's invitation links no longer work

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

### Requirement: Page action of the member list
The member list SHALL offer "Add member" as the page action in the page header. The page action SHALL sit at the top right on the line of the main heading, outside the heading. When heading and action do not fit on one line, the action SHALL wrap below the heading. The search field SHALL stay below the header.

#### Scenario: Add member in the header
- **WHEN** an organizer opens the member list
- **THEN** the link "Add member" is the first control after the main heading outside the heading. The search field follows it
