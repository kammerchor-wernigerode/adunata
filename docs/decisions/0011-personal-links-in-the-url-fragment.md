# ADR 0011: Personal links in the URL fragment

## Context

Members answer an invitation without an account. Each invitation therefore carries a link that grants access to it, and sign-in links grant access to an organizer session. Operators run Adunata behind a reverse proxy whose access log records the path and query of every request. Adunata cannot control that log. Pages linked from Adunata learn its address through the referrer unless the page forbids it.

The database, its statement log and every backup outlive the mail that carried a link. A link must stop working when an organizer revokes or replaces it.

## Decision

We will give every personal link and every sign-in link a random secret of 256 bits from the generator of the operating system, written as 64 hexadecimal characters. We will store only its SHA-256 hash, computed by the server, so the secret never reaches the database or its statement log.

We will put the secret into the fragment of the URL, such as `https://adunata.example.org/invitation#3f9a…`. Browsers never send the fragment. The client reads it and sends it to the API only in the header `Authorization: Bearer <secret>`. Adunata never logs headers or requests.

We will set `Referrer-Policy: no-referrer` and a `Content-Security-Policy` that admits only scripts of Adunata and forbids framing on every response. An unknown and a revoked secret lead to the same answer, which names neither gathering nor member. Resending stores the hash of a new secret in place of the old one.

We considered the secret in the path or in a query parameter, which the proxy would log, a signed token, which would need a list of revoked tokens, and hashing in PostgreSQL, which would send the secret to the database.

## Status

Accepted

## Consequences

No request line, log, referrer or database row carries a secret in plain form.

The response page works only with JavaScript, which the whole client needs already, see [ADR 0003](0003-single-page-client-with-api.md).

Revoking or replacing a link takes effect in the same transaction. A member who loses the mail needs a resend, because Adunata cannot show the old link again.

The server depends on the crates `getrandom` and `sha2`, which the lock file held already.
