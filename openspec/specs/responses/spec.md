# responses Specification

## Purpose
Lets invited members say whether and when they attend a gathering and what they need. Gives organizers the overview they plan with.

## Requirements

### Requirement: Response status
An invitation SHALL have one of the statuses pending, accepted, tentative or declined. It SHALL start as pending. A member SHALL be able to choose accepted, tentative or declined.

#### Scenario: Accept
- **WHEN** a member opens the personal link and chooses accepted
- **THEN** Adunata stores the status accepted for that invitation

#### Scenario: Decline
- **WHEN** a member chooses declined
- **THEN** Adunata stores the status declined and clears the period and the beds of that response

### Requirement: Period of presence
A member who accepts or accepts tentatively SHALL be able to state when they arrive and when they leave, in steps of 30 minutes within the span of the gathering. Without a stated period Adunata SHALL treat the member as present for the whole gathering. Adunata SHALL show these times in the time zone of the instance, which the operator configures.

#### Scenario: Times in the time zone of the instance
- **WHEN** the instance runs in the time zone Europe/Berlin and a member arrives on 14 March 2027 at 18:00 local time
- **THEN** the form and the overview show 18:00, whatever time zone the browser or the server uses

#### Scenario: Time between the steps
- **WHEN** a member sends an arrival at 18:10
- **THEN** Adunata rejects the response and marks the arrival as invalid

#### Scenario: No period stated
- **WHEN** a member accepts without stating a period
- **THEN** organizers see the member as present from the first to the last day

#### Scenario: Period outside the gathering
- **WHEN** a member enters an arrival before the first day of the gathering
- **THEN** Adunata rejects the response and marks the arrival as invalid

#### Scenario: Leave before arrival
- **WHEN** a member enters a departure that lies before the arrival
- **THEN** Adunata rejects the response and marks the departure as invalid

### Requirement: Beds
A member who accepts or accepts tentatively SHALL state whether they need beds, offer beds or need none. A member who needs or offers beds SHALL state how many, from 1 to 20.

#### Scenario: Offer beds
- **WHEN** a member states that they offer two beds
- **THEN** Adunata stores an offer of two beds for that response

#### Scenario: Need without a count
- **WHEN** a member states that they need beds but gives no count
- **THEN** Adunata rejects the response and asks for the count

### Requirement: Comment
A member SHALL be able to add a comment of up to 2000 characters to a response with any status.

#### Scenario: Comment with a decline
- **WHEN** a member declines and adds a comment
- **THEN** Adunata stores the comment with the response

### Requirement: Change a response
A member SHALL be able to change a response through the same link until the first day of the gathering begins in the time zone of the instance. Adunata SHALL show the stored values when the member opens the link again. After the gathering has begun the link SHALL show the response without a form.

#### Scenario: Change from tentative to accepted
- **WHEN** a member who answered tentative opens the link again and chooses accepted
- **THEN** the form shows the earlier values and Adunata stores the new status

#### Scenario: Open the link after the start
- **WHEN** a member opens the personal link on the first day of the gathering
- **THEN** Adunata shows the stored response and offers no form

### Requirement: Responses by organizers
An organizer SHALL be able to enter and change the response of any invitation with the same fields as the member, also after the gathering has begun. Adunata SHALL record whether the member or an organizer made the last change and when.

#### Scenario: Answer on a member's behalf
- **WHEN** a member accepts by phone and an organizer enters the acceptance
- **THEN** Adunata stores the acceptance and the overview shows that an organizer made the last change

#### Scenario: Member opens the link afterwards
- **WHEN** a member opens the personal link after an organizer changed the response before the gathering began
- **THEN** the form shows the values the organizer entered

### Requirement: Retention
Adunata SHALL delete comments 30 days and whole responses 365 days after the last day of a gathering. The operator SHALL be able to set each period in days or to keep the data. When the period for whole responses ends, Adunata SHALL delete the invitations of the gathering together with their responses and keep the gathering.

#### Scenario: Default retention
- **WHEN** the operator configured no retention period and a gathering ended 31 days ago
- **THEN** its responses exist without their comments

#### Scenario: Keep everything
- **WHEN** the operator set both periods to keep the data and a gathering ended two years ago
- **THEN** its responses and comments still exist

#### Scenario: Responses expire
- **WHEN** the operator configured a retention period of 90 days for responses and a gathering ended 91 days ago
- **THEN** the gathering still exists without invitations and responses

#### Scenario: Comments expire
- **WHEN** the operator configured a retention period of 30 days for comments and a gathering ended 31 days ago
- **THEN** the responses of that gathering exist without their comments

### Requirement: Confirmation
Adunata SHALL confirm a stored response on the page right away. It SHALL send no mail for a response.

#### Scenario: Save a response
- **WHEN** a member saves a response
- **THEN** the page states that the response is stored and shows its values

### Requirement: Attendees visible to members
On the response page Adunata SHALL show a member the names of everyone invited to the same gathering who accepted or accepted tentatively, each with that status. It SHALL show only counts for declined and pending invitations. It SHALL never show another member's period, beds, comment or mail address.

#### Scenario: Member looks at attendees
- **WHEN** a member opens the personal link of a gathering with three accepted, one tentative and two declined invitations
- **THEN** the page lists four names with their status and states that two declined

#### Scenario: Comment of another member
- **WHEN** another invited member has added a comment
- **THEN** the page does not show that comment

### Requirement: Notice on the response page
The response page SHALL state that other invited members see the member's name and status. It SHALL state at the comment field that only organizers read the comment.

#### Scenario: Member reads the notice
- **WHEN** a member opens the response form
- **THEN** the page states who sees the name, the status and the comment

### Requirement: Overview for organizers
Adunata SHALL show organizers every invitation of a gathering with member, status, period, beds, comment and who made the last change. It SHALL show the count per status and the total of beds needed and beds offered. It SHALL show an invitation whose mail failed for good and offer to resend it.

#### Scenario: Plan beds
- **WHEN** two members need one bed each and one member offers three beds
- **THEN** the overview shows two beds needed and three beds offered

#### Scenario: Filter by status
- **WHEN** an organizer filters the overview by pending
- **THEN** the overview lists only the members who have not answered yet

### Requirement: Actions column of the overview
The overview table SHALL end with a column "Actions". For an invitation whose mail failed or whose link is revoked, this column SHALL hold the notice and, while the gathering is neither cancelled nor ended, the action "Resend". The name cell SHALL hold only the linked name. In the card layout below a container width of 40rem, the actions SHALL be one more labelled line of the card, which a row with an empty actions cell SHALL not show.

#### Scenario: Last column names the actions
- **WHEN** an organizer opens the overview of a gathering
- **THEN** the last column heading reads "Actions" in English and "Aktionen" in German

#### Scenario: Failed mail in the actions column
- **WHEN** the invitation mail of a member failed and the gathering is neither cancelled nor ended
- **THEN** the last cell of the member's row shows "mail failed" and the button "Resend" while the name cell shows only the linked name

#### Scenario: Revoked link in the actions column
- **WHEN** the link of a member is revoked and the gathering is neither cancelled nor ended
- **THEN** the last cell of the member's row shows "link revoked" and the button "Resend"

#### Scenario: Resend accepted
- **WHEN** an organizer resends an invitation from the actions column and the server accepts it
- **THEN** the button "Resend" disappears from that row

#### Scenario: Resend refused
- **WHEN** an organizer resends an invitation from the actions column and the server refuses it
- **THEN** the button "Resend" stays and the cell names the failure

#### Scenario: Notice without action on a closed gathering
- **WHEN** the gathering has ended or is cancelled and a member's mail failed
- **THEN** the actions cell of that row shows the notice with its label "Actions" and no button "Resend"

#### Scenario: Row without actions
- **WHEN** a member's mail did not fail and the link is active
- **THEN** the actions cell of that row is empty and carries no label, so the card layout shows no actions line for it
