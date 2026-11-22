## Why

`adunata serve --dev` turns on a sign-in shortcut whose guard allows it only on the own machine. Issue #75 makes `--dev` the option for mail confinement on test systems, which run on other hosts. The shortcut therefore needs an option of its own, named after where it runs.

## What Changes

- **BREAKING**: The sign-in shortcut moves from `serve --dev` to `serve --local`. Its behaviour and its guard stay as they are.
- `serve --dev` stays accepted and has no effect until #75 gives it one. It logs no hint to `--local`.
- The capability `development-mode` becomes `local-mode`, and the term "development mode" becomes "local mode" in specs, code, help text, log messages and documentation.
- The warning at start reads "local mode is on, so the log contains sign-in links". The log message of a sign-in link becomes `sign-in link for local mode`.
- A new ADR "Local mode on localhost" supersedes ADR 0018.

## Capabilities

### New Capabilities
- `local-mode`: The sign-in shortcut of `serve --local` on localhost, renamed from `development-mode`, and the option `--dev` without effect.

### Modified Capabilities
- `development-mode`: Removed, its requirements move to `local-mode`.
- `organizer-access`: References name local mode and the capability `local-mode`.
- `service-operation`: The log exception and the scenario of the exit status name local mode and `serve --local`.

## Impact

- `server/src/main.rs`, `server/src/access.rs`, `server/src/api.rs` and `server/tests/startup.rs`.
- `README.md`, `dev-data/participate/README.md`, `dev-data/participate/import.sh`, `docs/operating/installation.md`, `docs/arc42/` and `docs/decisions/`.
- Anyone who runs `serve --dev` for a debug run switches to `serve --local`.
