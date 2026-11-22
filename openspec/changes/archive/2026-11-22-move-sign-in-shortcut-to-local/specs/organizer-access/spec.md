## MODIFIED Requirements

### Requirement: First organizer
The operator SHALL name the first organizer by mail address and name when setting up an instance. Adunata SHALL create that person as an active member with the organizer role if no organizer exists yet. In local mode a default member SHALL take the place of a first organizer that the operator did not name, as the capability local-mode describes.

#### Scenario: Set up a new instance
- **WHEN** an operator starts a new instance with the mail address and name of the first organizer
- **THEN** Adunata creates that member with the organizer role

#### Scenario: Instance without any organizer
- **WHEN** an instance without an organizer starts without a first organizer configured and without local mode
- **THEN** Adunata starts and logs a warning that nobody can sign in

#### Scenario: Instance already has an organizer
- **WHEN** an instance that already has an organizer starts with a first organizer configured
- **THEN** Adunata leaves members and roles unchanged

### Requirement: Sign-in by mail link
An organizer SHALL sign in by entering a mail address and opening the link that Adunata then sends to it. The link SHALL work once and expire one hour after Adunata sent it. In local mode Adunata SHALL also create one link at start-up without mail, as the capability local-mode describes.

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

### Requirement: Limited sign-in requests
Adunata SHALL limit how many sign-in links it sends to one address and accepts from one network address within a period. In local mode the limit per address SHALL not apply, as the capability local-mode describes.

#### Scenario: Many requests from one network address
- **WHEN** one network address requests more sign-in links for different addresses than the limit allows
- **THEN** Adunata sends no further link to any address from that network address until the period ends and shows the same confirmation

#### Scenario: Request through a trusted proxy
- **WHEN** requests from two clients reach Adunata through a reverse proxy that the operator named as trusted
- **THEN** Adunata counts each client against its own limit

#### Scenario: Repeated requests
- **WHEN** someone requests sign-in links for the same address more often than the limit allows while local mode is off
- **THEN** Adunata sends no further link until the period ends and shows the same confirmation

