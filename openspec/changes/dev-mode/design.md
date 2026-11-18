# Design

## Context

`serve` creates the first organizer through `access::ensure_first_organizer` from the optional bootstrap settings in `Configuration::first_organizer`. A sign-in link exists only after `POST /api/sign-in-requests`. `send_sign_in_link` in `server/src/access.rs` stores the hash of a fresh secret in `sign_in_links`, builds the link from the public URL as `/sign-in#<secret>` and puts the mail in the outbox. It counts every row in `sign_in_links` of the member from the past hour and sends nothing from the fourth on. The sign-in page redeems the link with `POST /api/sessions` and the secret as `Authorization: Bearer <secret>`.

`--config` is the precedent for an option of the command that is not a setting of a table. `PublicUrl::localhost_port()` already tells whether the public URL is on `localhost`, for the name of the session cookie.

## Goals / Non-Goals

**Goals:**

- `cargo run -- serve --dev` in `server/` gives a signed-in browser or agent within seconds, also on an empty database or on a database copied from another checkout.
- An agent finds the link in one JSON line on standard output with a fixed message and a field `link`.

**Non-Goals:**

- No change to the client, the API or the database schema.
- No seed data besides the organizer, such as members or gatherings.
- No development mode on a host other than `localhost`, such as a remote staging server.
- No replacement for Mailpit. Mail still goes out, and invitation mail still needs it.

## Decisions

### Option on `serve` without a variable

`Command::Serve` gets the field `dev: bool` with `#[arg(long)]`. `main` passes it to `serve` and leaves `Configuration` untouched. No environment variable turns the mode on, so a variable copied into a Compose file or an env file of a service cannot turn it on by accident. The flag is also not read from `server/.env`, which would make it look like a setting.

A subcommand `serve-dev` would repeat every option of `serve` and drift from it. A setting in the configuration file such as `[development] mode` would follow the naming rule of the tables. Development mode is a way to run the command and not a property of the instance, so it belongs next to `--config`.

This decision is architecturally significant, because an operator sees the option in `adunata help serve` of every build. ADR 0018 below records it.

### Every build, only on `localhost`

The option does not depend on `cfg(debug_assertions)`, unlike the reading of `.env`. A person can debug a release build from `nix build` or from the end-to-end setup the same way, and an agent never needs to know which build runs.

`serve` refuses to start with `--dev` when `configuration.public_url.localhost_port()` is `None`. It logs an error that names `ADNT_HTTP_PUBLIC_URL` and exits with status 1, before it connects to the database, as for an invalid configuration. A production instance needs HTTPS on its own host, so it cannot turn on development mode. Every local run, every worktree and the end-to-end setup use `localhost`.

The guard is specific to this mode, because the mode puts links that grant access into the log. It sets no rule for other features of development.

Development mode in every build without a guard would leave only a warning between a stray option and organizer links in the log of a production instance. Development mode only in debug builds would rule out debugging a release build.

This decision is architecturally significant, because the shipped binary carries the mode and checks the public URL for it. It is part of ADR 0018.

### Development steps in one function

`main.rs` gets `async fn prepare_organizers(pool, first_organizer, language, public_url, development_mode) -> Result<Option<String>, sqlx::Error>`. Without development mode it creates the configured first organizer as before and returns `None`. In development mode it logs the warning, picks the configured or default first organizer, calls `ensure_first_organizer`, creates the start-up link, logs it and returns it. `serve` calls it after `connect` and before it binds the listener and exits with status 1 on an error. The `integration` module in `main.rs` tests the function with a test database and `test_logs::captured_logs()`, so the branch between both paths has automatic tests with and without the option.

A test of the binary in `server/tests/startup.rs` that spawns `serve --dev` would come closer to real use. It needs a database and process handling that the file does not have, because all its tests stop before the database.

This decision is not architecturally significant, because it only arranges code and tests.

### Default first organizer

In development mode `prepare_organizers` uses `configuration.first_organizer` or, if it is unset, the member `organizer@example.org` named Ada Lovelace. `access` owns the default as a public function `development_organizer()`, because `access` owns the first organizer. `ensure_first_organizer` stays unchanged and creates the member only when no active organizer exists. The address and name match the example in the [configuration reference](../../../docs/operating/configuration.md) and in the end-to-end tests. `example.org` is a reserved domain, so no real person receives mail.

Creating or reactivating the default organizer on every start, like `grant-organizer` does, would change the roles of a database that someone set up on purpose.

This decision is not architecturally significant, because only a development run notices which member it creates.

### Sign-in link at start-up for the oldest active organizer

`access` gets a public function `development_sign_in_link(pool, public_url) -> Result<String, sqlx::Error>`. It takes the active organizer with the lowest id, stores a link for that member like `send_sign_in_link` does and returns the link. It puts no mail in the outbox and skips the limit per address, so frequent restarts keep working. The insert of the link moves into a private function that both callers use. After `ensure_first_organizer` in development mode an active organizer always exists, so the query fails only if the database changes in between, which `serve` reports as an error.

On a fresh database the oldest active organizer is the default or bootstrap organizer. On a database copied from the main checkout it is whoever has been an active organizer there the longest by id. An agent that has no session can thus always sign in, without knowing an address.

Taking the first organizer of the run fails on a copied database whose organizers differ from the settings. The person or agent then gets no link and cannot learn an address without a session.

`serve` logs the link at level `info` with the message `sign-in link for development` and the field `link`. Printing it with `println!` would give a second format on standard output. The JSON log is what the operator and an agent already read.

This decision is not architecturally significant on its own. It is the behavior that ADR 0018 accepts for development mode.

### Link of each request in the log

`api::AppState` gets the field `development_mode: bool`, and `api::testing::state` sets it to `false`. When it is set, `send_sign_in_link` logs the link with the same message and field after the transaction commits. The mail still goes out.

Returning the link in the response of `POST /api/sign-in-requests` and showing it on the sign-in page would change client and API. It would also tell anyone which addresses belong to organizers, which the requirement "No disclosure of accounts" forbids.

This decision is not architecturally significant on its own, as above.

### No limit per address in development mode

When `development_mode` is set, `send_sign_in_link` skips the count of recent links per member. The limit per network address in `SignInLimiter` stays. Without this, three restarts within an hour use up the quota of the organizer. The sign-in page then drops every request without mail and without a log line. Telling start-up links apart in the count would need a new column, which the non-goals rule out.

This decision is not architecturally significant on its own. It is part of the behavior that ADR 0018 accepts.

### Warning at start

`serve` logs at level `warn` that development mode is on and that the log contains sign-in links.

This decision is not architecturally significant, because it only reports the mode that ADR 0018 introduces.

## Risks / Trade-offs

- [Someone who reads the log of a development instance can sign in as an organizer within one hour.] → The guard keeps the mode on `localhost`, where the log belongs to the person who started the process.
- [In development mode nothing limits the links per address.] → The limit of 20 requests per network address and hour still applies, and the instance runs on `localhost`.
- [A reverse proxy or tunnel could expose an instance whose public URL says `localhost`.] → The session cookie of such an instance has no `Secure` flag and links point to `localhost`, so it does not work as a public instance anyway. The operating documentation says the option is for local runs only.
- [A test of the log output shares the global subscriber with every other test of the binary.] → Each test asserts on its own random secret, which no other test produces.

## Architecture documentation

- `05-building-block-view.md` adds the default first organizer and the sign-in link for development mode to the interfaces of `access`.
- `08-cross-cutting-concepts.md`, section Authentication and authorization, gets one paragraph on development mode with the `localhost` guard, the default first organizer, the link at start-up and the missing limit per address. It links to ADR 0018.
- `08-cross-cutting-concepts.md`, section Logging, names the exception for sign-in links in development mode.
- `09-architecture-decisions.md` gets a row for ADR 0018.

Section 6 stays as it is, because the sign-in flow keeps its steps.

## Draft decision record

The draft supersedes no record. It makes exceptions to [ADR 0010](../../../docs/decisions/0010-sign-in-by-mail-link.md) and [ADR 0011](../../../docs/decisions/0011-personal-links-in-the-url-fragment.md) without changing them.

```markdown
# ADR 0018: Development mode on localhost

## Context

A person or a coding agent who starts Adunata for a manual debug run needs an organizer and a sign-in link before any work can begin. On a fresh database that means bootstrap settings or `grant-organizer` and then a mail in Mailpit. An agent cannot open Mailpit easily.

No log carries a secret in plain form, see [ADR 0011](0011-personal-links-in-the-url-fragment.md). Adunata limits sign-in links to three per address and hour, see [ADR 0010](0010-sign-in-by-mail-link.md). A production instance needs a public URL with HTTPS, and only a public URL on `localhost` may use HTTP.

Debug runs happen with debug builds and with release builds, such as the binary of the end-to-end tests.

## Decision

We will offer `adunata serve --dev` in every build and refuse it unless the public URL is on `localhost`. No environment variable turns it on.

In development mode Adunata uses `organizer@example.org`, Ada Lovelace, as the first organizer unless the bootstrap settings name one. At start-up it logs a sign-in link for the oldest active organizer. It logs the link of every sign-in link it sends by mail and drops the limit of links per address. These are exceptions to ADR 0011 and ADR 0010 for development mode only. It logs a warning that development mode is on.

We considered a separate subcommand, a setting in the configuration file, an environment variable, development mode in every build without a guard, development mode only in debug builds and showing the link on the sign-in page.

## Status

Accepted

## Consequences

A person or an agent starts a debug run with one command and signs in with the link from the log. An agent can redeem the secret of the link through the API without a browser.

A production instance cannot turn on development mode, because its public URL is not on `localhost`. A staging server on another host cannot use it either.

On `localhost` anyone who reads the log can sign in as an organizer within one hour.
```
