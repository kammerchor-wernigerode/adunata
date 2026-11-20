# Tasks

## 1. Command

- [x] 1.1 Add the request with its deadline parameter and the mapping of unspecified addresses to loopback in `server/src/health.rs`, as design.md decision 1 describes. Verify with the unit and integration tests of design.md decision 3 in that module.
- [x] 1.2 Add the command `healthcheck` to `server/src/main.rs` with a configuration path that returns its problems, and the one-line output on failure. Verify with the process tests of design.md decision 3 in `server/tests/startup.rs`.
- [ ] 1.3 Let `nix/test.nix` run the container with `--health-cmd` in the exec form of design.md decision 2 and check `podman healthcheck run` before and after PostgreSQL stops. Verify by running `nixosVmTest` with its output in a log file.
- [ ] 1.4 Let the image start in `e2e` of `.github/workflows/ci.yaml` wait for `docker exec` of `adunata healthcheck`. Verify by the run of the pull request on both architectures.

## 2. Documentation

- [ ] 2.1 Document the command under "Commands" and "Health" in `docs/operating/installation.md`, add `healthcheck:` to the Compose example and show `--health-cmd` for Podman, both in the exec form, and name the start period, the tools that act on health and `ADNT_CONFIG_FILE` for containers with `--config`. Verify that `adunata help healthcheck` matches the text.
- [ ] 2.2 Write the record "Healthcheck as a command of the binary" from the draft in design.md with the next free number in `docs/decisions/`, add it to `docs/arc42/09-architecture-decisions.md` and update `03-context-and-scope.md`, `05-building-block-view.md` and "Health" in `08-cross-cutting-concepts.md` as design.md lists. Check the number again after the last rebase onto `main`. Verify that every link resolves.

## 3. Hand-over

- [ ] 3.1 Run `openspec-verify-change` for `container-healthcheck` and fix what it finds.
- [ ] 3.2 Run a code review of the whole branch with the review command of the agent and fix what it finds.
- [ ] 3.3 Run `nix build .#checks.<system>.integration` for the local system and fix every failure.
- [ ] 3.4 Rebase onto `main`, push the branch and open the pull request ready for review.
