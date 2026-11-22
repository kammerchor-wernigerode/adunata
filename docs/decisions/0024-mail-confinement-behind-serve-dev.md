# ADR 0024: Mail confinement behind serve --dev

## Context

A test system often holds a copy of real member data. Every invitation or sign-in link it sends then reaches real members. [ADR 0023](0023-local-mode-on-localhost.md) left `serve --dev` without effect for a meaning on test systems.

A test system runs on another host than `localhost`, so a guard on the public URL would block it. An operator who copies the configuration of a test system to production must not send mail to members by mistake.

## Decision

We will let `serve --dev` allow two settings, `mail.confine` and `mail.confine-to`. With `confine = true`, Adunata delivers every mail to the redirect address in `confine-to`, with the original recipient in the header `X-Original-To`. Subject and bodies stay unchanged. The redirect happens at delivery, so mail that waited in the outbox before the start is redirected too.

Without `--dev`, `serve` refuses to start with status 78 when either setting is present, even with `confine = false`. With `confine = true` and no valid redirect address, `serve --dev` refuses to start with status 78. `serve` logs a warning at start while confinement is on.

`--dev` alone changes nothing. It applies no guard to the public URL or the listen address. `--dev` and `--local` are independent. `healthcheck` and `grant-organizer` accept both settings.

We rejected an allowlist of recipients or domains, because one mailbox is enough to check the mail of a test system.

## Status

Accepted

## Consequences

A test system sets `--dev` and both settings. A configuration copied to production without `--dev` refuses to start instead of sending mail to members.

A test system without the settings still sends to its members. Confinement protects only when an operator turns it on.

The outbox keeps the original recipient. Turning confinement off later delivers the mail still waiting to its members.

`healthcheck` and `grant-organizer` accept the settings, so the container healthcheck of a test system keeps working.
