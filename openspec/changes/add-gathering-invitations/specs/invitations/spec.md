# Spec Delta

## Purpose

Lets organizers invite members to a gathering by mail and gives each invited member a personal link to answer without an account.

## ADDED Requirements

### Requirement: Invite active members
An organizer SHALL be able to invite all active members who are not yet invited to a gathering. An organizer SHALL also be able to invite a chosen subset of them. Adunata SHALL invite each member at most once per gathering.

#### Scenario: Invite everyone
- **WHEN** an organizer invites all active members to a gathering
- **THEN** Adunata creates one invitation per active member and queues one mail per invitation

#### Scenario: Member added later
- **WHEN** an organizer adds a member after the first round of invitations and invites all members again
- **THEN** Adunata invites only the new member

### Requirement: Invitation mail
The invitation mail SHALL name the gathering, its location and its span and SHALL contain the member's personal link. It SHALL be written in the member's language.

#### Scenario: Mail content
- **WHEN** Adunata sends an invitation to a member whose language is German
- **THEN** the mail is in German and contains the title, location, span and personal link of the gathering

### Requirement: Personal link
Each invitation SHALL have its own link that identifies it through a random secret of at least 128 bits. Adunata SHALL store only a hash of the secret. The link SHALL grant access to that one invitation and nothing else.

#### Scenario: Open a valid link
- **WHEN** a member opens the personal link of an invitation
- **THEN** Adunata shows the response form for that invitation without asking for a sign-in

#### Scenario: Guessed link
- **WHEN** someone opens a link with a secret that matches no invitation
- **THEN** Adunata shows a page that says the link is invalid and reveals no gathering or member

### Requirement: Link expiry
A personal link SHALL expire when its gathering has ended. An expired link SHALL show the gathering but accept no response.

#### Scenario: Open after the gathering
- **WHEN** a member opens the personal link after the last day of the gathering
- **THEN** Adunata shows that the gathering is over and offers no form

### Requirement: Resend and revoke
An organizer SHALL be able to resend an invitation. Resending SHALL create a new link and revoke the old one. An organizer SHALL be able to revoke a link without sending a new one.

#### Scenario: Resend
- **WHEN** an organizer resends an invitation
- **THEN** the member receives a mail with a new link and the old link no longer works

#### Scenario: Old link after resend
- **WHEN** a member opens a link that was revoked
- **THEN** Adunata shows that the link is no longer valid and accepts no response

### Requirement: Links stay out of logs and referrers
Adunata SHALL keep personal links out of its logs. Pages reached through a personal link SHALL not send the link to other sites as a referrer.

#### Scenario: Request with a personal link
- **WHEN** a member opens a personal link
- **THEN** the log entry for that request does not contain the secret
