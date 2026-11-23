## MODIFIED Requirements

### Requirement: Remind action on the overview
While the first day of a gathering has not begun in the time zone of the instance, the gathering is not cancelled and at least one invitation is pending with an active link, the overview SHALL offer a button next to the status filter that names the number of these members, "Remind N members" in English and "N Mitglieder erinnern" in German. The overview SHALL take whether the gathering has begun from the server, not from the clock of the browser. A click SHALL open a dialog that names the number of members, says that each of them receives a new link and that the old link stops working, and offers to confirm or to go back. Adunata SHALL send nothing before the organizer confirms. After the server accepts, the dialog SHALL close, the overview SHALL reload and the page SHALL show the number of reminders queued. If the server refuses or fails, the dialog SHALL stay open and name the failure. The button and the dialog SHALL be shadcn components.

#### Scenario: Button names the count
- **WHEN** an organizer opens the overview of an open gathering with three pending invitations, one of them with a revoked link
- **THEN** the overview shows the button "Remind 2 members"

#### Scenario: Go back
- **WHEN** an organizer clicks the button and then goes back in the dialog
- **THEN** the dialog closes and Adunata queues no mail

#### Scenario: Confirm
- **WHEN** an organizer clicks the button and confirms in the dialog
- **THEN** the dialog closes, the overview reloads and the page shows that 2 reminders were sent

#### Scenario: Server refuses
- **WHEN** an organizer confirms and the server refuses or fails
- **THEN** the dialog stays open and names the failure

#### Scenario: Nobody to remind
- **WHEN** no invitation of the gathering is pending with an active link
- **THEN** the overview shows no remind button

#### Scenario: Closed gathering
- **WHEN** the gathering is cancelled or has ended
- **THEN** the overview shows no remind button

#### Scenario: Gathering has begun
- **WHEN** the server reports that the gathering has begun and one invitation is pending with an active link
- **THEN** the overview shows no remind button
