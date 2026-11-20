# Tasks

## 1. Exit statuses

- [ ] 1.1 Add the pure function that tells whether an error is temporary and the method on `StartupError` that returns 65, 75 or 1 in `server/src/database.rs`, as the table in design.md decision 1 lists. Verify with unit tests for every row, including `InvalidData`, `57P01`, `42501` and a SQLSTATE of class `23`.
- [ ] 1.2 Make `main.rs` exit with the classified status where design.md decision 1 lists, exit with 78 for every invalid configuration and move binding into a function that returns its error. Verify with the tests of design.md decision 3 in `database::integration`, in the module of the bind function and in `server/tests/startup.rs`.
- [ ] 1.3 Add a table of 0, 1, 2, 65, 75 and 78 under "Commands" in `docs/operating/installation.md`, say there that 75 also covers a socket path that does not exist, give 78 instead of 1 in the introduction of `docs/operating/configuration.md` and in "Development mode" of `installation.md`, say in "Updates" that a downgrade exits with 65 and add `restart: on-failure` with a sentence on Docker to the Compose example. Verify with `grep -rn 'status 1' docs` that no sentence gives 1 for an invalid configuration.

## 2. Restart policy

- [ ] 2.1 Add `RestartPreventExitStatus=65 78` to `nix/module.nix` and `deploy/systemd/adunata.service`, and let the module order and require `postgresql.target`. Let `nix/test.nix` wait for `postgresql.target` and check that a service that exits with 78 stays stopped. Verify by running `nixosVmTest` with its output in a log file.
- [ ] 2.2 Show in the parts on NixOS and on the release binary of `docs/operating/installation.md` how an operator overrides the restart policy. Verify that the NixOS snippet evaluates with `nix eval` against a configuration of the VM test.

## 3. Record and architecture

- [ ] 3.1 Write the record "Exit statuses from sysexits for a failed start" from the draft in design.md with the next free number in `docs/decisions/`, add it to `docs/arc42/09-architecture-decisions.md` and update `03-context-and-scope.md`, `07-deployment-view.md`, `08-cross-cutting-concepts.md` and `10-quality-requirements.md` as design.md lists. Check the number again after the last rebase onto `main`. Verify that every link resolves.

## 4. Hand-over

- [ ] 4.1 Run `openspec-verify-change` for `startup-exit-statuses` and fix what it finds.
- [ ] 4.2 Run a code review of the whole branch with the review command of the agent and fix what it finds.
- [ ] 4.3 Run `nix build .#checks.<system>.integration` for the local system and fix every failure.
- [ ] 4.4 Rebase onto `main`, push the branch and open the pull request ready for review. Its title names the breaking change of status 78.
