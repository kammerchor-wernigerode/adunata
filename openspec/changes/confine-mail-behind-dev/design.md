# Design

## Context

`serve` takes `--dev` and discards it since ADR 0023. `send()` in `server/src/mail.rs` is the only place that sets the recipient of a mail, and `deliver_due` resolves the address with `COALESCE(members.mail, outbox.recipient)` only at delivery. `Configuration::load` in `server/src/configuration.rs` reads every setting from the layered sources, lists keys in `SETTINGS` and reports every problem at once, which `main.rs` turns into exit status 78. The configuration file takes strings, integers and string arrays, and rejects a boolean.

## Goals / Non-Goals

**Goals:** keep the mail of a test system away from real members, with a forgotten option failing closed.

**Non-Goals:** an allowlist of recipients or domains, other features behind `--dev`, any change to `--local`.

## Decisions

1. **Settings `mail.confine` and `mail.confine-to`.** Environment variables `ADNT_MAIL_CONFINE` and `ADNT_MAIL_CONFINE_TO`. They sit in the table `mail` next to `sender` and `max-attempts`. Significant, because operators set them.
2. **Redirect in `send()`.** With confinement on, `send()` builds the `To` from the redirect address and adds `X-Original-To` with the original address. The address comes from the configuration at delivery, so mail enqueued before the start is redirected too. The outbox keeps its original recipient. Significant, because a running instance delivers differently.
3. **Original recipient only in `X-Original-To`.** The subject and bodies stay as an unconfined mail would be, so a tester sees what a member would see. Not significant on its own, it is part of decision 2.
4. **No allowlist.** One mailbox is enough to check the mail of a test system. Part of decision 2.
5. **Refusal belongs to `serve`.** `Configuration::load` reads both settings for every command. `serve` without `--dev` refuses with 78 when either setting is present, even `confine = false`, because a forgotten option must never send mail to real members. `healthcheck` and `grant-organizer` have no `--dev` and accept the settings, so the container healthcheck of a test system keeps working. `grant-organizer` sends no mail, so it needs no confinement. Significant, because an operator sees the refusal.
6. **Missing or invalid redirect address refuses the start.** With `confine = true`, `serve --dev` exits with 78 when `confine-to` is missing or no valid `lettre::Address`. Safer than a silent fallback. Part of decision 5.
7. **Warning at start.** `serve` logs a warning while confinement is on, without the address, matching the warning of local mode. Part of decision 2.
8. **Boolean in the configuration file.** `confine` takes `true` or `false` in TOML and the strings `true` and `false` in the environment. The file reader turns a TOML boolean into the string form only for this key. Not significant beyond decision 1.
9. **`--dev` alone changes nothing and has no guard.** A test system runs on another host, so a guard on the public URL would block it. `--dev` and `--local` stay independent. Significant, because it defines what an operator gets from the option.

The significant decisions 1, 2, 5 and 9 become one record, ADR 0024 "Mail confinement behind serve --dev". It supersedes no record. ADR 0023 stays accepted, because it already left `--dev` free for this meaning.

Draft of ADR 0024:

- Context: an operator who wants to see the mail of an instance without reaching its recipients has no way to stop delivery. ADR 0023 left `serve --dev` without effect. A guard on the public URL would block an instance on another host. A configuration that confines mail must not start on an instance meant to send mail.
- Decision: `serve --dev` allows two settings, `mail.confine` and `mail.confine-to`. With `confine = true`, every mail goes to the redirect address, with the original recipient in `X-Original-To`. Without `--dev`, `serve` refuses to start with 78 when either setting is present. With `confine = true` and no valid redirect address, `serve` refuses with 78. `--dev` alone changes nothing and has no guard on the public URL or the listen address. `--dev` and `--local` are independent. We rejected an allowlist, because one mailbox is enough to see every mail an instance sends.
- Consequences: an operator who wants confined mail sets `--dev` and both settings. A configuration with the settings but without `--dev` refuses to start instead of ignoring them. The redirect mailbox receives working links. `healthcheck` and `grant-organizer` accept the settings.

## arc42

- `05-building-block-view.md`: the module `mail` redirects mail while confinement is on.
- `08-cross-cutting-concepts.md`: the configuration concept names the boolean switch and the binding of the confinement settings to `serve --dev`.
- `09-architecture-decisions.md`: list ADR 0024.

## Risks / Trade-offs

- A test system without `--dev` and without the settings still sends to members. Confinement protects only when an operator turns it on. The documentation says so.
- The outbox keeps the original recipient, so turning confinement off later delivers pending mail to members. That matches the behaviour of every other setting at delivery.
