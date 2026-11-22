## Context

The option `dev: bool` of the subcommand `Serve` in `server/src/main.rs` turns on the shortcut. `serve` checks it with `runs_only_on_this_machine` and passes it on as `development_mode`, which `access.rs` reads from the state in `api.rs`. ADR 0018 records the decision with `--dev`. Issue #75 needs `--dev` for mail confinement on hosts other than localhost.

## Goals / Non-Goals

**Goals:**
- Move the shortcut to `--local` with unchanged behaviour and guard.
- Keep `--dev` accepted and without effect.
- Use the term "local mode" in every current place.

**Non-Goals:**
- Mail confinement behind `--dev`, see #75.
- Any change to the guard, the default organizer, the logged sign-in links or the limits.
- Archived OpenSpec changes and the text of ADR 0018, which keep the old term.

## Decisions

- **Name `--local`.** The name states where the shortcut runs, which is what its guard enforces. We reject `--debug`, because the shortcut also runs in release builds such as the binary of the end-to-end tests, and because `debug` is also a log level. Significant, because operators see the option.
- **`--dev` stays without effect.** It turns on nothing, applies no guard and logs no hint, because no operator outside the project uses it yet and #75 gives it a meaning soon. With `--dev` alone Adunata behaves like `serve`, also with a public URL on another host. Significant, because an operator notices which option turns on the shortcut. Recorded in the same ADR.
- **Rename throughout.** The capability `development-mode` becomes `local-mode`. Code identifiers follow, such as `development_mode` to `local_mode` and `development_organizer` to `local_organizer`. Not significant on its own, because only the names change.
- **Wording.** The warning at start reads "local mode is on, so the log contains sign-in links". The log message becomes `sign-in link for local mode`, and the script in `README.md` that selects it follows. It is the only reader of that message in the repository. Not significant, because a single term is a matter of consistency.
- **New ADR.** ADR 0001 forbids changing a record, so ADR 0023 "Local mode on localhost" carries the decision of ADR 0018 with `--local`, says why the name is not `--debug` and supersedes ADR 0018. ADR 0018 keeps its text and only gets the status `Superseded by [ADR 0023](0023-local-mode-on-localhost.md)`.
- **Anchor.** The heading "Development mode" in `docs/operating/installation.md` becomes "Local mode". Every link to `#development-mode` follows to `#local-mode`.

### arc42 sections

- `01-introduction-and-goals.md`: the row of the capability links `local-mode`.
- `05-building-block-view.md`: the module `access` names local mode.
- `08-cross-cutting-concepts.md`: the paragraph on the shortcut names `serve --local`, local mode and ADR 0023. The exception in the section on logs names local mode.
- `09-architecture-decisions.md`: lists ADR 0023 as accepted and ADR 0018 as superseded.

## Risks / Trade-offs

- [`--dev` keeps a piece of the shortcut after losing the guard, so a test system on another host logs working sign-in links] → A test starts `serve --dev` with a public URL on another host and checks that no sign-in link reaches the log.
- [Anyone used to `serve --dev` gets no shortcut and no hint] → The README and the operating docs name `--local`. Only people inside the project used `--dev`.

## Open Questions

- OpenSpec does not let a MODIFIED requirement rename a scenario. The two scenarios "Sign-in link outside development mode" and "Development mode outside localhost" in `service-operation` keep their titles, and only their steps name local mode. Renaming the titles is left to the archive of this change.
