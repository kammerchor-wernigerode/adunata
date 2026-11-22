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
Adunata SHALL show organizers the members sorted by last name, with a filter for active and inactive members and a search by name or mail address.

#### Scenario: Search
- **WHEN** an organizer searches for part of a last name
- **THEN** the list shows only members whose name or mail address contains that text, ignoring case

### Requirement: Page action of the member list
The member list SHALL offer "Add member" as the page action in the page header. The page action SHALL sit at the top right on the line of the main heading, outside the heading. When heading and action do not fit on one line, the action SHALL wrap below the heading. The search field SHALL stay below the header.

#### Scenario: Add member in the header
- **WHEN** an organizer opens the member list
- **THEN** the link "Add member" is the first control after the main heading outside the heading. The search field follows it
