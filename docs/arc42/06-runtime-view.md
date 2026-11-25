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

## Answering through a personal link

A member opens the personal link from the invitation mail and answers. See [ADR 0011](../decisions/0011-personal-links-in-the-url-fragment.md) for why the secret stays in the fragment and [ADR 0012](../decisions/0012-wall-clock-times-in-the-instance-time-zone.md) for the times.

```mermaid
sequenceDiagram
  actor Member
  participant Client
  participant responses
  participant invitations
  participant gatherings
  Member->>Client: opens /invitation#secret
  Client->>responses: GET /api/invitation with Authorization Bearer secret
  responses->>invitations: extractor PersonalLink resolves the hash of the secret
  invitations->>gatherings: loads the gathering
  responses->>responses: decides from the date of the instance whether the member can answer
  responses-->>Client: gathering, state, stored response and attendees
  Member->>Client: chooses status, period, beds and comment
  Client->>responses: PUT /api/invitation/response
  responses->>responses: validates against the days of the gathering and stores
  responses-->>Client: 200 with the stored values, announced on the page
```

|Step|Building block|What happens|
|-|-|-|
|Open|`responses`, `invitations`|An unknown or revoked secret ends in status 404 that names nothing. Otherwise the page shows the gathering, the member's stored response and who comes.|
|State|`responses`, `gatherings`|`responses` derives the state of the page from the gathering and the date of the instance. The state decides whether the page offers a form. The [responses specification](../../openspec/specs/responses/spec.md) defines the states.|
|Store|`responses`|Validates the answer against the days of the gathering and stores it with a note that the member changed it last.|

## Start-up

The operator or the service manager runs `adunata serve`. The order matters, because each step fails before the next one causes an effect. The [installation guide](../operating/installation.md#commands) lists the exit statuses, see [ADR 0019](../decisions/0019-exit-statuses-from-sysexits-for-a-failed-start.md).

```mermaid
sequenceDiagram
  participant Manager as Service manager
  participant main
  participant configuration
  participant database
  participant mail
  participant responses
  participant access
  participant serving
  Manager->>main: adunata serve
  main->>configuration: reads and checks every setting
  configuration-->>main: typed settings or every problem at once
  main->>database: opens the pool
  database->>database: compares the schema with the binary, applies pending migrations
  main->>mail: starts the delivery task
  main->>responses: starts the retention task
  main->>main: opens the listen address
  main->>access: creates the first organizer if the instance has none
  main->>serving: serves the router
  serving-->>Manager: accepts requests until SIGTERM
```

|Step|Building block|What happens|
|-|-|-|
|Configuration|`main`, `configuration`|An invalid configuration ends the start before Adunata touches the database or opens a port.|
|Database|`database`|A database that is unreachable or newer than the binary ends the start. Otherwise the pending migrations run before any other module uses the pool.|
|Background tasks|`mail`, `responses`|Both tasks run inside the process of the server, so an instance is one process.|
|Organizers|`access`|The first organizer exists before the first request arrives, so someone can sign in.|
|Serving|`serving`|On SIGTERM it stops accepting connections and lets requests in progress finish. Mail that waits stays in the outbox.|

## Delivering mail from the outbox

The delivery task of `mail` runs for as long as the process runs. The scenario shows a mail server that rejects the first attempt. See [ADR 0006](../decisions/0006-outbox-for-mail.md) for the outbox and [ADR 0013](../decisions/0013-mail-loses-its-content-once-its-delivery-ends.md) for the content.

```mermaid
sequenceDiagram
  participant mail
  participant Database as PostgreSQL
  participant Mailserver as Mail server
  mail->>Database: locks the oldest mail that is due
  mail->>Mailserver: sends the mail
  Mailserver-->>mail: rejects it or does not answer
  mail->>Database: counts the attempt and sets a later time for the next one
  mail->>Database: locks the mail again once it is due
  mail->>Mailserver: sends the mail
  Mailserver-->>mail: accepts it
  mail->>Database: marks the mail as sent and clears its content
```

|Step|Building block|What happens|
|-|-|-|
|Pick|`mail`|One transaction locks a due mail and skips mail that another transaction holds, so two deliveries never send the same mail.|
|Retry|`mail`|Each failed attempt moves the next one further away. After the configured number of attempts the mail fails for good and loses its content too.|
|Result|`mail`|The outbox keeps the row with its time of delivery or failure. `invitations` reads it to show where the mail of an invitation stands.|

The [mail delivery specification](../../openspec/specs/mail-delivery/spec.md) holds the requirements for retries. The [configuration reference](../operating/configuration.md#settings) names the setting for the number of attempts.
