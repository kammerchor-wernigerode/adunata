# ADR 0010: Sign-in by mail link

## Context

Organizers are volunteers who sign in a few times per month, often on a laptop, and read their mail on a phone. Passwords would need hashes, a reset by mail and a defense against guessing. The reset mail would be a sign-in link anyway. Most ensembles run no identity provider.

Programs that check mail for malicious content open every link in a mail. A link that starts a session when it is opened would be used up by such a program or would sign it in.

Revoking the organizer role or deactivating a member has to end every session of that member. A page on a sibling subdomain counts as the same site, so a `SameSite` cookie alone does not stop it from sending requests that change data.

## Decision

We will sign organizers in with a link that Adunata sends by mail. An organizer enters a mail address. The answer is the same whether or not the address belongs to an active organizer. The link carries a secret of 256 bits in its fragment and works once within one hour. It opens a page with a button, and only the button's request redeems the link in one `UPDATE` statement. The link works in any browser.

We will keep a session as a random secret in a cookie and store only its SHA-256 hash for 14 days. Outside `localhost` the cookie is named `__Host-adunata_session` with `Secure`, `HttpOnly`, `SameSite=Strict` and `Path=/`. On `localhost` it drops prefix and `Secure` and its name ends with the port of the public URL. Every organizer request loads the session together with its member and fails unless the member is active and holds the organizer role.

We will reject every API request other than `GET` or `HEAD` whose `Origin` differs from the public URL. We will limit sign-in requests to three links per hour per address, counted in the database, and to 20 requests per hour per network address, counted in memory, with the network address taken from `X-Forwarded-For` only behind a trusted proxy.

We will create the first organizer from settings at start-up and offer the subcommand `grant-organizer` for an operator who lost every organizer mailbox.

We considered passwords, an external identity provider, a link that signs in when opened, a link bound to the requesting browser, signed session tokens without server state and a token against cross-site request forgery.

## Status

Accepted

## Consequences

Adunata stores no password and no secret in plain form. A mail scanner neither uses up a link nor signs in.

A forwarded link signs in whoever opens it first within the hour. Binding it to a browser would break the common switch from laptop to phone.

Revoking the role ends a session at its next request without extra bookkeeping. Every organizer request costs one more query.

Signing in depends on a working mail server. An instance without a reachable mail server cannot let new organizers in.

The limit per network address lives in memory and resets with every restart. One process is enough for an ensemble.
