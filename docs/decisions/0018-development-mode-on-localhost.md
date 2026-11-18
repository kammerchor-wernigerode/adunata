# ADR 0018: Development mode on localhost

## Context

A person or a coding agent who starts Adunata for a manual debug run needs an organizer and a sign-in link before any work can begin. On a fresh database that means bootstrap settings or `grant-organizer` and then a mail in Mailpit. An agent cannot open Mailpit easily.

No log carries a secret in plain form, see [ADR 0011](0011-personal-links-in-the-url-fragment.md). Adunata limits sign-in links to three per address and hour, see [ADR 0010](0010-sign-in-by-mail-link.md). A production instance needs a public URL with HTTPS, and only a public URL on `localhost` may use HTTP.

Debug runs happen with debug builds and with release builds, such as the binary of the end-to-end tests.

## Decision

We will offer `adunata serve --dev` in every build and refuse it unless the public URL is on `localhost` and Adunata listens on a loopback address. No environment variable turns it on.

In development mode Adunata uses `organizer@example.org`, Ada Lovelace, as the first organizer unless the bootstrap settings name one. At start-up it logs a sign-in link for the oldest active organizer. It logs the link of every sign-in link it sends by mail and drops the limit of links per address. These are exceptions to ADR 0011 and ADR 0010 for development mode only. It logs a warning that development mode is on.

We considered a separate subcommand, a setting in the configuration file, an environment variable, development mode in every build without a guard, development mode only in debug builds and showing the link on the sign-in page.

## Status

Accepted

## Consequences

A person or an agent starts a debug run with one command and signs in with the link from the log. An agent can redeem the secret of the link through the API without a browser.

A production instance cannot turn on development mode, because its public URL is not on `localhost`. Another machine on the network cannot reach an instance in development mode. A staging server on another host and a server in a container that listens on every interface cannot use it either.

On `localhost` anyone who reads the log can sign in as an organizer within one hour.
