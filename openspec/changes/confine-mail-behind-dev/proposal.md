# Proposal

## Why

An instance that holds real member addresses sends every invitation or sign-in link to those members, and Adunata has no way to keep that mail away from them. Since local mode moved to `serve --local`, the option `serve --dev` is free and accepted without effect, so it can carry this protection.

## What Changes

- Two new settings in the table `mail`: `confine` (`ADNT_MAIL_CONFINE`) turns mail confinement on, and `confine-to` (`ADNT_MAIL_CONFINE_TO`) holds the redirect address.
- While confinement is on, `serve` delivers every mail to the redirect address instead of its recipient, including mail that waited in the outbox before the start. The mail carries its original recipient in the header `X-Original-To`. Subject and both body parts stay unchanged.
- `serve` without `--dev` refuses to start with exit status 78 when either setting is present, even when `confine` is `false`.
- `serve --dev` refuses to start with exit status 78 when `confine` is `true` and `confine-to` is missing or no valid mail address.
- `serve` logs a warning at start while confinement is on.
- `--dev` alone changes nothing. It has no guard on the public URL or the listen address and is independent of `--local`.
- `healthcheck` and `grant-organizer` accept both settings without `--dev`.
- The configuration file accepts a boolean for `confine`.
- A new ADR records `--dev` and mail confinement. ADR 0023 stays in force.

## Capabilities

### New Capabilities

### Modified Capabilities
- `mail-delivery`: new requirement for mail confinement.
- `instance-configuration`: the configuration file accepts a boolean for a switch, and a new requirement binds the confinement settings to `serve --dev`.
- `service-operation`: the log warns while confinement is on.

## Impact

- `server/src/main.rs`: the option `--dev` of `serve`, its help, the check before start and the warning.
- `server/src/configuration.rs`: the two settings, the boolean value and their validation.
- `server/src/mail.rs`: `send()` redirects and adds `X-Original-To`.
- `docs/operating/configuration.md`, `docs/operating/installation.md`, `docs/decisions/`, `docs/arc42/`.
