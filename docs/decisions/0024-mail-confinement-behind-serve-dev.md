# ADR 0024: Mail confinement behind serve --dev

## Context

Adunata sends invitations and sign-in links to the addresses of its members. An operator who wants to see these mails without reaching their recipients has no way to stop delivery. [ADR 0023](0023-local-mode-on-localhost.md) left `serve --dev` without effect.

Such an instance may run on any host, so a guard on the public URL would block it. A configuration that confines mail may be copied to an instance that should send mail. That instance must not start with confinement settings it was never meant to have.

## Decision

We will let `serve --dev` allow two settings, `mail.confine` and `mail.confine-to`. With `confine = true`, Adunata delivers every mail to the redirect address in `confine-to`, with the original recipient in the header `X-Original-To`. Subject and bodies stay unchanged. The redirect happens at delivery, so mail that waited in the outbox before the start is redirected too.

Without `--dev`, `serve` refuses to start with status 78 when either setting is present, even with `confine = false`. With `confine = true` and no valid redirect address, `serve --dev` refuses to start with status 78. `serve` logs a warning at start while confinement is on.

`--dev` alone changes nothing. It applies no guard to the public URL or the listen address. `--dev` and `--local` are independent. `healthcheck` and `grant-organizer` accept both settings.

We rejected an allowlist of recipients or domains, because one mailbox is enough to see every mail an instance sends.

## Status

Accepted

## Consequences

An operator who wants confined mail sets `--dev` and both settings. A configuration with the settings but without `--dev` refuses to start instead of silently ignoring them.

An instance without the settings sends mail to its recipients, with or without `--dev`. Confinement takes effect only when an operator turns it on.

The redirect mailbox receives working invitation and sign-in links.

The outbox keeps the original recipient. Turning confinement off later delivers the mail still waiting to its recipients.

`healthcheck` and `grant-organizer` accept the settings, so a container healthcheck keeps working on a confined instance.
