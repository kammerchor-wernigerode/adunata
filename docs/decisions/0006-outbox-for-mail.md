# ADR 0006: Outbox for mail

## Context

Invitations reach members only by mail. A lost invitation means a member never answers. Organizers notice it late or not at all. Mail servers of ensembles are often shared hosting accounts that are unreachable for minutes at a time.

An action such as storing a response and the mail it causes have to succeed or fail together. Sending during the request ties the action to the mail server. Either the action fails when the server is down. Or the mail is lost when the action succeeds but sending fails.

Adunata runs as one process next to PostgreSQL, see [ADR 0007](0007-modular-monolith.md). Operators should not run an extra queue.

## Decision

We will write every mail to an outbox table in the same transaction as the action that causes it. A background task in the same process sends due mail from the outbox. It retries a failed delivery after 10 seconds and then doubles the delay up to one hour. After a configurable number of attempts it marks the mail as failed. It marks a mail as sent as soon as the mail server accepts it and never sends it again.

We considered sending during the request, which loses mail or fails actions during an outage. We considered an external message queue, which adds a service to operate for a few hundred mails per gathering.

## Status

Accepted

## Consequences

An action never fails because the mail server is down. A rolled back action never leaves a mail behind.

Mail leaves with a delay of up to the polling interval of the background task, currently two seconds.

If the process stops between handing a mail to the server and marking it as sent, the mail goes out a second time after the next start. Members may then receive the same mail twice, which is less harmful than receiving none.

The outbox keeps every sent mail with its content. How long it keeps them is not yet decided.
