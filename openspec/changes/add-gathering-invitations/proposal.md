# Proposal

## Why

Ensembles meet every four to six weeks to rehearse and give concerts. Such a gathering spans one or more days, often a weekend and sometimes days during the week. Organizers need to know early who comes and when each member arrives and leaves. They also need to know who needs or offers a bed. This change covers one complete round, from creating a gathering to seeing every member's response. It is the smallest change that replaces the core of the predecessor application.

## What Changes

- Organizers sign in with a single-use link sent by mail. Adunata stores no passwords. The operator names the first organizer when setting up the instance.
- Organizers keep a list of members with name, mail address, language and whether the member is active.
- Organizers create, edit and cancel gatherings. A gathering has a title, a location, a description and a continuous span of days. Cancelling a gathering informs every invited member by mail.
- Organizers invite the active members to a gathering by mail. Each invitation carries a personal link that is secret, expires and can be revoked.
- A member opens the link and answers without an account until the gathering starts. The member accepts, accepts tentatively or declines. A member who comes states when they are present and whether they need or offer beds and how many. Every member may add a comment.
- Organizers enter and change responses on a member's behalf, also after the gathering has started.
- Members who open their link see who has accepted or accepted tentatively. They see no other member's period, beds or comment.
- Organizers see every response and the totals per status, along with the beds needed and offered.
- Times of arrival and departure appear in the time zone of the instance, which operators set with a new setting.
- Adunata deletes comments 30 days and responses 365 days after a gathering ends. Operators can change both periods or keep the data. Deleting a member deletes the member's invitations, responses, sessions and queued mail.
- Mail in the outbox loses its content once its delivery ends, so personal links do not stay in the database.
- Operators can link a privacy notice and an imprint. Every page and every invitation mail shows the links.
- Operators name the reverse proxies that Adunata trusts for the network address of a request.
- An operator who has lost every organizer mailbox restores access with a subcommand of the binary.

Out of scope for this change are shared activities such as a dinner, reminders, response deadlines, travel by car, mail that covers several gatherings, notifying members when the days of a gathering change, statistics about past gatherings, exports, calendar links, voices or sections and sign-in through an external identity provider. Shared activities belong to the core and follow in a later change. Taking over data from the predecessor application also follows in its own change. Adunata works without it and starts empty.

This change depends on `establish-foundation`.

## Capabilities

### New Capabilities

- `organizer-access`: signing in and out as an organizer and granting the organizer role
- `member-roster`: the list of members that organizers keep
- `gatherings`: creating, editing and cancelling gatherings
- `invitations`: inviting members to a gathering and managing their personal links
- `responses`: how members answer an invitation, what they see of others and what organizers see

### Modified Capabilities

- `instance-configuration`: the required time zone, trusted proxies and links to a privacy notice and an imprint
- `mail-delivery`: mail loses its content once its delivery ends and belongs to a member

## Impact

- The server gains modules for access, members, gatherings, invitations and responses. The browser client gains views for organizers and the response form for members.
- New mails are the sign-in link, the invitation and the cancellation of a gathering, all in German and English.
- The database gains tables for members, sessions, sign-in links, gatherings, invitations and responses. The outbox gains references to the member and the invitation.
- Most arc42 sections change, as listed in the design. New decision records cover sign-in by mail link, personal links in the fragment of the URL, wall-clock times in the time zone of the instance and the content of delivered mail.
