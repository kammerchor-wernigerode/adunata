# Tasks

## 1. Option and guard

- [x] 1.1 Add `dev: bool` with `#[arg(long)]` to `Command::Serve` in `server/src/main.rs` and pass it to `serve`. Verify with a test in `server/tests/startup.rs` that `adunata help serve` names `--dev`.
- [x] 1.2 Make `serve --dev` log an error naming `ADNT_HTTP_PUBLIC_URL` and exit with status 1 before connecting to the database when `configuration.public_url.localhost_port()` is `None`. Verify with a test in `server/tests/startup.rs` that `serve --dev` with `ADNT_HTTP_PUBLIC_URL=https://adunata.example.org` and the unreachable database exits with status 1 and names the public URL, not the database.

## 2. Default organizer and link at start-up

- [x] 2.1 Add `access::development_organizer()` that returns `organizer@example.org`, Ada Lovelace. Verify with a unit test of its values.
- [x] 2.2 Move the insert of a sign-in link in `server/src/access.rs` into a private function and keep `send_sign_in_link` on it. Verify that the existing sign-in tests in `access` still pass.
- [x] 2.3 Add `access::development_sign_in_link(pool, public_url)` that creates a link for the active organizer with the lowest id, without mail and without the limit per address. Verify with integration tests that its secret starts a session through `POST /api/sessions`, that it picks the older of two active organizers, that it still returns a link after the limit per address is reached and that it puts no mail in the outbox.
- [x] 2.4 Add `prepare_organizers` in `server/src/main.rs` that creates the first organizer as before without development mode and, in development mode, picks the configured or default first organizer, calls `ensure_first_organizer` and returns the start-up link. Call it from `serve` before the listener binds, log the warning that development mode is on and the link at level `info` with the message `sign-in link for development` and the field `link`, and exit with status 1 on an error. Verify with integration tests in the `integration` module of `main.rs` that on an empty database it creates `organizer@example.org`, that bootstrap settings win, that the returned link starts a session and that its secret appears in `test_logs::captured_logs()` only on the development path.

## 3. Link of each request in the log

- [x] 3.1 Add `development_mode: bool` to `api::AppState`, set it from the option in `serve` and to `false` in `api::testing::state`. Let `send_sign_in_link` log the link with the same message and field after the commit when it is set. Verify with integration tests using `test_logs::captured_logs()` that the secret of a requested link appears in the log with development mode and does not appear without it.
- [x] 3.2 Skip the count of recent links per member in `send_sign_in_link` when `development_mode` is set. Verify with an integration test that a fourth request within an hour puts a mail in the outbox with development mode, and with the existing test that it puts none without.

## 4. Documentation and decision record

- [ ] 4.1 Write `docs/decisions/0018-development-mode-on-localhost.md` from the draft in design.md and add its row to `docs/arc42/09-architecture-decisions.md`. Verify that every link resolves.
- [ ] 4.2 Update `docs/arc42/05-building-block-view.md` with the new interfaces of `access`, and `docs/arc42/08-cross-cutting-concepts.md` with a paragraph on development mode under Authentication and authorization and the exception for sign-in links under Logging, both linking to ADR 0018. Verify that the text matches the specs of `development-mode`, `organizer-access` and `service-operation`.
- [ ] 4.3 Describe `adunata serve --dev`, its `localhost` guard and the link in the log under Commands in `docs/operating/installation.md`. Verify that the links resolve.
- [ ] 4.4 Change the command under Run Adunata in `README.md` to `cargo run -- serve --dev` and show how an agent gets a session: wait for the line `accepting requests`, take `.fields.link` of the line whose `.fields.message` is `sign-in link for development` with `jq`, and send the part after `#` to `POST /api/sessions` as `Authorization: Bearer <secret>` with `curl` and a cookie jar. Verify by running these commands against a fresh database and getting status 200 from `GET /api/session` with the cookie.

## 5. Checks

- [ ] 5.1 Run `openspec validate dev-mode --strict` and the checks up to integration, `nix build .#checks.<system>.integration` or the commands under Checks before a push in the README, and verify that both pass.

## Workflow follow-up

- Run `openspec-verify-change` and `/code-review high` on the branch before the first push.
- Archive the change on `main` after the pull request is merged.
