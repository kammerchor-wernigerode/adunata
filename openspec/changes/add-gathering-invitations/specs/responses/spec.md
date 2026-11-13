# Spec Delta

## Purpose

Lets invited members say whether and when they attend a gathering and what they need. Gives organizers the overview they plan with.

## ADDED Requirements

### Requirement: Response status
An invitation SHALL have one of the statuses pending, accepted, tentative or declined. It SHALL start as pending. A member SHALL be able to choose accepted, tentative or declined.

#### Scenario: Accept
- **WHEN** a member opens the personal link and chooses accepted
- **THEN** Adunata stores the status accepted for that invitation

#### Scenario: Decline
- **WHEN** a member chooses declined
- **THEN** Adunata stores the status declined and clears the period and the beds of that response

### Requirement: Period of presence
A member who accepts or accepts tentatively SHALL be able to state when they arrive and when they leave, in steps of 30 minutes within the span of the gathering. Without a stated period Adunata SHALL treat the member as present for the whole gathering.

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
A member who accepts or accepts tentatively SHALL state whether they need beds, offer beds or need none. A member who needs or offers beds SHALL state how many, at least one.

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
Adunata SHALL keep responses and comments after a gathering ends unless the operator configures a retention period. The operator SHALL be able to set one period for comments and one for whole responses. Adunata SHALL delete them that many days after the last day of the gathering.

#### Scenario: Default retention
- **WHEN** a gathering ended a year ago and the operator configured no retention period
- **THEN** its responses and comments still exist

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

### Requirement: Overview for organizers
Adunata SHALL show organizers every invitation of a gathering with member, status, period, beds, comment and who made the last change. It SHALL show the count per status and the total of beds needed and beds offered.

#### Scenario: Plan beds
- **WHEN** two members need one bed each and one member offers three beds
- **THEN** the overview shows two beds needed and three beds offered

#### Scenario: Filter by status
- **WHEN** an organizer filters the overview by pending
- **THEN** the overview lists only the members who have not answered yet
