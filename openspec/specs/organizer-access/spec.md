# organizer-access Specification

## Purpose
Lets organizers sign in without a password and keeps every organizer function closed to anyone without the organizer role.

## Requirements

### Requirement: First organizer
The operator SHALL name the first organizer by mail address and name when setting up an instance. Adunata SHALL create that person as an active member with the organizer role if no organizer exists yet.

#### Scenario: Set up a new instance
- **WHEN** an operator starts a new instance with the mail address and name of the first organizer
- **THEN** Adunata creates that member with the organizer role

#### Scenario: Instance without any organizer
- **WHEN** an instance without an organizer starts without a first organizer configured
- **THEN** Adunata starts and logs a warning that nobody can sign in

#### Scenario: Instance already has an organizer
- **WHEN** an instance that already has an organizer starts with a first organizer configured
- **THEN** Adunata leaves members and roles unchanged

### Requirement: Recovery of organizer access
The binary SHALL offer the operator a subcommand that makes the member with a given mail address an active organizer and creates the member if needed. It SHALL use the configuration of the instance.

#### Scenario: Every organizer mailbox lost
- **WHEN** an operator runs the subcommand with a mail address, a first name and a last name
- **THEN** that member is an active organizer and can request a sign-in link

### Requirement: Sign-in by mail link
An organizer SHALL sign in by entering a mail address and opening the link that Adunata then sends to it. The link SHALL work once and expire one hour after Adunata sent it.

#### Scenario: Successful sign-in
- **WHEN** an active organizer requests a sign-in link, opens it within one hour and confirms with the button
- **THEN** Adunata starts a session for that organizer and the link no longer works

#### Scenario: Link opened by a mail scanner
- **WHEN** a program that checks mail for malicious content opens the sign-in link
- **THEN** Adunata shows a page with a button to sign in, starts no session and keeps the link usable

#### Scenario: Link used twice
- **WHEN** someone opens a sign-in link that was already used
- **THEN** Adunata starts no session and offers to send a new link

#### Scenario: Expired link
- **WHEN** someone opens a sign-in link after it expired
- **THEN** Adunata starts no session and offers to send a new link

### Requirement: No disclosure of accounts
The response to a sign-in request SHALL be the same whether or not the mail address belongs to an active organizer. Adunata SHALL send a link only to active organizers.

#### Scenario: Unknown address
- **WHEN** someone requests a sign-in link for an address that belongs to no organizer
- **THEN** Adunata shows the same confirmation as for an organizer and sends no mail

### Requirement: Limited sign-in requests
Adunata SHALL limit how many sign-in links it sends to one address and accepts from one network address within a period.

#### Scenario: Many requests from one network address
- **WHEN** one network address requests more sign-in links for different addresses than the limit allows
- **THEN** Adunata sends no further link to any address from that network address until the period ends and shows the same confirmation

#### Scenario: Request through a trusted proxy
- **WHEN** requests from two clients reach Adunata through a reverse proxy that the operator named as trusted
- **THEN** Adunata counts each client against its own limit

#### Scenario: Repeated requests
- **WHEN** someone requests sign-in links for the same address more often than the limit allows
- **THEN** Adunata sends no further link until the period ends and shows the same confirmation

### Requirement: Session and sign-out
A session SHALL end when the organizer signs out, 14 days after sign-in or when the organizer loses the role or is deactivated.

#### Scenario: Sign out
- **WHEN** an organizer signs out
- **THEN** Adunata ends the session and further requests with it are rejected

#### Scenario: Session expires
- **WHEN** an organizer sends a request 14 days and one minute after signing in
- **THEN** Adunata rejects it and the client shows the sign-in page

#### Scenario: Role revoked
- **WHEN** an organizer revokes the role of another organizer who has an active session
- **THEN** the next request in that session is rejected

### Requirement: Organizer role
Only organizers SHALL be able to use organizer functions. An organizer SHALL be able to grant the organizer role to an active member and to revoke it. Adunata SHALL keep at least one active organizer.

#### Scenario: Request without a session
- **WHEN** a request for an organizer function arrives without a valid session
- **THEN** Adunata rejects it and the client shows the sign-in page

#### Scenario: Request from another origin
- **WHEN** a page on another origin makes the browser of a signed-in organizer send a request that changes data
- **THEN** Adunata rejects the request and changes nothing

#### Scenario: Deactivate or delete the last organizer
- **WHEN** an organizer tries to deactivate or delete the only active organizer
- **THEN** Adunata refuses and explains why

#### Scenario: Revoke the last organizer
- **WHEN** an organizer tries to revoke the role of the only active organizer
- **THEN** Adunata refuses and explains why
