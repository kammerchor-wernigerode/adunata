# Runtime view

## Sign-in

An organizer requests a sign-in link on a laptop and opens the mail on a phone or on the same laptop. The sequence shows the building blocks that take part, see [ADR 0010](../decisions/0010-sign-in-by-mail-link.md) for the reasons.

```mermaid
sequenceDiagram
  actor Organizer
  participant Client
  participant access
  participant members
  participant mail
  participant Mailserver as Mail server
  Organizer->>Client: enters mail address
  Client->>access: POST /api/sign-in-requests
  access->>access: checks limits per network address
  access->>members: finds active organizer by address
  access->>access: checks limit per address, stores hash of new secret
  access->>mail: adds sign-in mail to the outbox
  access-->>Client: 202, the same answer for every address
  mail->>Mailserver: delivers in the background, then clears the content
  Organizer->>Client: opens /sign-in#secret from the mail and presses the button
  Client->>access: POST /api/sessions with Authorization Bearer secret
  access->>access: redeems the link in one UPDATE, stores hash of session secret
  access-->>Client: 201 with the session cookie
  Client->>access: GET /api/session with the cookie
  access->>members: loads the active organizer
  access-->>Client: 200 with the organizer
```

|Step|Building block|What happens|
|-|-|-|
|Request|`access`|Counts the request against the network address and then the links of the address in the last hour. Only for an active organizer within both limits it stores a link and queues its mail. The answer is always 202.|
|Delivery|`mail`|Sends the mail from the outbox with the organizer's name and clears its content once the mail server accepts it.|
|Confirmation|Client|Reads the secret from the fragment, which the browser never sends, and sends it only in the `Authorization` header when the organizer presses the button.|
|Redemption|`access`|Marks the link as used in the statement that finds it, so a second request fails. It starts a session for 14 days.|
|Every later request|`access`, `members`|The extractor for organizer routes loads the session with its member and rejects the request unless the member is still an active organizer.|

## Inviting members

An organizer invites the active members to a gathering. Mail leaves through the outbox, see [ADR 0006](../decisions/0006-outbox-for-mail.md), and loses its content once its delivery ends, see [ADR 0013](../decisions/0013-mail-loses-its-content-once-its-delivery-ends.md).

```mermaid
sequenceDiagram
  actor Organizer
  participant Client
  participant invitations
  participant gatherings
  participant members
  participant mail
  participant Mailserver as Mail server
  Organizer->>Client: invites all members not invited yet
  Client->>invitations: POST /api/gatherings/{id}/invitations
  invitations->>gatherings: finds the gathering, refuses if cancelled or ended
  invitations->>members: lists the active members
  loop each member not invited yet
    invitations->>invitations: stores the invitation with the hash of a new secret
    invitations->>mail: adds the mail with the personal link in the member's language
  end
  invitations-->>Client: 201 with the number of invitations
  mail->>Mailserver: delivers each mail, then clears its content
```

|Step|Building block|What happens|
|-|-|-|
|Check|`invitations`, `gatherings`|A cancelled or ended gathering takes no invitations.|
|Invite|`invitations`|One transaction stores every new invitation and its mail, so a failure leaves neither behind. A unique key per gathering and member skips members who are invited already.|
|Deliver|`mail`|Each mail goes out with the member's name and loses recipient, subject and bodies once the mail server accepts it or it fails for good. The list of invitations shows where the newest mail of each stands.|
