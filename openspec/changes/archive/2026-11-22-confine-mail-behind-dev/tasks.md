## 1. Configuration

- [x] 1.1 Add `mail.confine` and `mail.confine-to` to `SETTINGS` in `server/src/configuration.rs` and accept a TOML boolean for `mail.confine` only
- [x] 1.2 Load both settings into `Configuration`, keeping whether either is present, and report a malformed `confine` and, with `confine = true`, a missing or invalid `confine-to` as problems that name the variable and not the value
- [x] 1.3 Unit tests for the scenarios of `instance-configuration` that the loader decides: malformed switch, missing and invalid redirect address, boolean in the file, other values rejected as before

## 2. Serve

- [x] 2.1 Give `--dev` of `serve` a help text that describes confinement, and pass it to `serve`
- [x] 2.2 In `serve`, exit with 78 before touching the database when either setting is present without `--dev`, naming the setting and not its value
- [x] 2.3 Log a warning at start while confinement is on, without the redirect address
- [x] 2.4 Leave `healthcheck` and `grant-organizer` accepting both settings
- [x] 2.5 Tests for the refusal without `--dev`, for `--dev` on a public URL other than localhost and for the warning

## 3. Delivery

- [x] 3.1 Carry the redirect address in `Delivery` and, when set, address the mail to it in `send()` with the header `X-Original-To` holding the original address
- [x] 3.2 Integration tests against Mailpit: a confined mail arrives only at the redirect address with `X-Original-To` and unchanged subject and bodies, a mail enqueued before the start is redirected, and an unconfined mail has no `X-Original-To`

## 4. Documentation

- [x] 4.1 Describe `mail.confine` and `mail.confine-to` in the table of `docs/operating/configuration.md`, noting that they need `serve --dev`
- [x] 4.2 Describe `serve --dev` and mail confinement in `docs/operating/installation.md` next to local mode
- [x] 4.3 Write `docs/decisions/0024-mail-confinement-behind-serve-dev.md` from the draft in design.md and list it in `docs/arc42/09-architecture-decisions.md`
- [x] 4.4 Update `docs/arc42/05-building-block-view.md` and `docs/arc42/08-cross-cutting-concepts.md` as design.md names
