# Tasks

Each group is one commit that passes `nix build .#checks.<system>.fast`.

## 1. Drop test databases after each test

- [x] 1.1 Give `empty_database` and `fresh_database` in `server/src/database.rs` a return type that owns the database name and drops the database with `WITH (FORCE)` in its `Drop`, as design.md describes. Update the 19 call sites in `database.rs`, `health.rs`, `mail.rs` and `spike.rs` so each binds the value for the whole test. Verify that `cargo test` passes and that `psql -tAc "select count(*) from pg_database where datname like 'adunata\_test\_%'"` returns the same number before and after the run.
- [x] 1.2 Update the server slice row in [section 08](../../../docs/arc42/08-cross-cutting-concepts.md#testing) to say that each test drops its database afterwards. Verify that the relative links resolve.
- [x] 1.3 Extend the README sentence on `cargo test` in "Tests and lints" to say that each test drops its database. Add the command that drops leftover `adunata_test_*` databases while no tests run. Verify that the command runs as written.

## 2. Share compiler output across worktrees

- [x] 2.1 Add `pkgs.sccache` to the development shell in `flake.nix` and set `RUSTC_WRAPPER` to it. The commit message says what sccache does and why the project needs it. Verify that `sccache --show-stats` counts compile requests after `cargo build` in the shell.

## 3. Load the development shell through direnv

- [x] 3.1 Add `.envrc` with `use flake` followed by `source_env_if_exists .envrc.local`. Add `.envrc.local` and `/.direnv` to `.gitignore`. Verify that `direnv allow` in the main checkout sets `ADUNATA_DATABASE_URL` to the database `adunata`.
- [x] 3.2 Let `client/vite.config.ts` read `ADUNATA_VITE_PORT` with fallback 5173, set `strictPort` and build the proxy target from `ADUNATA_LISTEN_ADDRESS` with fallback `127.0.0.1:8080`. Verify that `npm run dev` without the variables serves on 5173 and proxies `/api` to 8080. Verify that `npm run lint` passes.
- [x] 3.3 Describe direnv with nix-direnv as the way into the development shell in the README section "With Nix". Mention `direnv exec . <command>` for shells without the direnv hook. In "Run Adunata" say that `ADUNATA_VITE_PORT` and `ADUNATA_LISTEN_ADDRESS` set the Vite port and its proxy target. Verify that each documented command runs as written.

## 4. Plugin for git-worktree-init

- [ ] 4.1 Add a `[*.py]` section with `indent_size = 4` to `.editorconfig`. Verify that the check `editorconfig` passes.
- [ ] 4.2 Write `dev/worktree_plugin.py` with the options `port_base` (default 18000) and `clone_db` (default off), `SANITIZE_ENV`, `slot_of`, `apply`, `next_steps` and `teardown` as design.md describes. Verify with `python3 -m py_compile dev/worktree_plugin.py`.
- [ ] 4.3 Write the decision record `docs/decisions/0009-optional-worktree-tooling.md` from the draft in design.md. The record carries its own definition of reference `[1]`. Add its row to [section 09](../../../docs/arc42/09-architecture-decisions.md). Verify that the relative links resolve.
- [ ] 4.4 Add a section on parallel worktrees to `README.md`. It covers installing git-worktree-init, `git config worktree-init.plugin dev/worktree_plugin.py`, the options and the port layout. It explains how to include Claude Code worktrees by clearing `worktree-init.ignorePaths` and that their databases go away at the next `git wt prune`. It says that `git wt remove` fails while PostgreSQL is down. Verify that each documented command runs as written.

## 5. Acceptance

- [ ] 5.1 Create two worktrees with `git wt add`. Verify that each gets `.envrc.local`, its own database and a built client. Verify that server and Vite of both run at the same time next to the main checkout.
- [ ] 5.2 Create a worktree with `--clone-db` while the main checkout's server runs. Verify that the clone holds the main checkout's rows.
- [ ] 5.3 Run `git wt refresh` on a worktree. Verify that it keeps its slot and its data.
- [ ] 5.4 Remove one worktree with `git wt remove` and the other with `git worktree remove` followed by `git wt prune`. Verify that both databases are gone.
- [ ] 5.5 Create a worktree from a branch older than this change. Verify that creation succeeds without `direnv allow`.
- [ ] 5.6 Create a Claude Code worktree after clearing `ignorePaths`. Verify that the hook prepares it. Run `env | grep ADUNATA_DATABASE_URL` through the Bash tool of Claude Code in that worktree and record whether it shows the slot's database or needs `direnv exec .`.
- [ ] 5.7 Build the server in the second worktree. Verify that `sccache --show-stats` reports cache hits.
