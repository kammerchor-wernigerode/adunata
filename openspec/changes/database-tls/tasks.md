# Tasks

## 1. TLS and the check of the URL

- [x] 1.1 Add the sqlx feature `tls-rustls-ring-native-roots` in `server/Cargo.toml` and update `server/Cargo.lock`. Verify that `cargo tree -i webpki-roots` finds nothing, that `cargo deny check licenses` passes and that the notices still build. The commit says that the feature adds no crate.
- [x] 1.2 Add the check of design.md decision 2 to `server/src/configuration.rs`, with `url` as a direct dependency. Verify with the unit tests of design.md decision 3. The commit says what `url` does and why the check needs it, and names its license.
- [ ] 1.3 Extend `nix/test.nix` as design.md decision 3 lists. Verify by running `nixosVmTest` with its output in a log file.

## 2. Documentation

- [ ] 2.1 Document `sslmode` and `sslrootcert` in the row of `database.url` in `docs/operating/configuration.md`, including the default `prefer`, that `allow` behaves like `disable`, that `sslrootcert` takes effect only with `verify-ca` or `verify-full`, that `sslmode=disable` restores plain text, which parameters Adunata rejects and that it refuses every variable whose name starts with `PGSSL`. Extend "Certificate authorities" and the introduction of `docs/operating/installation.md` from the mail server to the database, and show how the file of the authority gets into the container. Verify that the example URL matches the VM test.
- [ ] 2.2 Write the record "TLS to the database configured in its URL" from the draft in design.md with the next free number in `docs/decisions/`, add it to `docs/arc42/09-architecture-decisions.md` and update `03-context-and-scope.md`, `07-deployment-view.md` and `08-cross-cutting-concepts.md` as design.md lists. Check the number again after the last rebase onto `main`. Verify that every link resolves.

## 3. Hand-over

- [ ] 3.1 Run `openspec-verify-change` for `database-tls` and fix what it finds.
- [ ] 3.2 Run a code review of the whole branch with the review command of the agent and fix what it finds.
- [ ] 3.3 Run `nix build .#checks.<system>.integration` for the local system and fix every failure.
- [ ] 3.4 Rebase onto `main`, push the branch and open the pull request ready for review.
