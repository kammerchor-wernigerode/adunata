## 1. Server

- [ ] 1.1 In `server/src/main.rs`, add the option `--local` to `serve` with the help "Start in local mode for a debug run on localhost" and turn the shortcut on with it instead of `--dev`
- [ ] 1.2 Keep `--dev` on `serve`, described in the help as an option without effect, and make it change nothing
- [ ] 1.3 Rename `development_mode`, `development_organizer`, `development_sign_in_link` and the module `development` of the tests to local mode in `main.rs`, `access.rs` and `api.rs`, together with their doc comments and the error message of the guard
- [ ] 1.4 Change the warning at start to "local mode is on, so the log contains sign-in links" and both log messages to `sign-in link for local mode`

## 2. Tests

- [ ] 2.1 In `server/tests/startup.rs`, check that the help of `serve` lists `--local` and `--dev`, and start the guard tests with `serve --local`
- [ ] 2.2 Add a test that starts `serve --dev` with a public URL on another host and checks that Adunata starts and that no sign-in link and no warning of local mode reach the log
- [ ] 2.3 Rename the tests in `main.rs` and `access.rs` that name development mode

## 3. Documentation

- [ ] 3.1 In `docs/operating/installation.md`, rename the section "Development mode" to "Local mode", name `serve --local` and the new log message
- [ ] 3.2 In `README.md`, name `serve --local`, link `installation.md#local-mode` and select `sign-in link for local mode` in the script
- [ ] 3.3 In `dev-data/participate/README.md` and the comment in `dev-data/participate/import.sh`, name `serve --local` and local mode
- [ ] 3.4 Update `docs/arc42/01-introduction-and-goals.md`, `05-building-block-view.md` and `08-cross-cutting-concepts.md` as design.md lists
- [ ] 3.5 Check with `grep -rn -i "development.mode\|serve --dev\|#development-mode"` outside `openspec/changes/archive/` and ADR 0018 that no current place keeps the old term

## 4. Decision records

- [ ] 4.1 Write `docs/decisions/0023-local-mode-on-localhost.md` from the template, with `--local`, the reason against `--debug`, `--dev` without effect and the note that it supersedes ADR 0018
- [ ] 4.2 Set the status of ADR 0018 to `Superseded by [ADR 0023](0023-local-mode-on-localhost.md)` without changing its other text
- [ ] 4.3 List ADR 0023 as accepted and ADR 0018 as superseded in `docs/arc42/09-architecture-decisions.md`
