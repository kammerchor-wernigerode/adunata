# Tasks

## 1. Build and check per architecture

- [x] 1.1 Open a draft pull request for this branch, as design.md decision 7 allows. Verify that its run starts.
- [x] 1.2 Replace `verify` and `nixos` in `.github/workflows/ci.yaml` with the matrix job `build` and its artifacts as design.md decision 1 describes. Verify with `zizmor` through the lint target and by the run of the draft pull request on both architectures.
- [x] 1.3 Turn `e2e` into the same matrix with the image start of design.md decision 2. Verify by the run of the draft pull request on both architectures.
- [x] 1.4 Add the concurrency groups and the cache rules of design.md decision 5, including the Playwright key, the save only on `main`, `actions: write` on the saving job and the prefix in `.github/workflows/release.yaml`. Verify with `zizmor` and by `grep -n -A6 'cache' .github/workflows/*.yaml`.

## 2. Skip and aggregate

- [x] 2.1 Add the job `changes` of design.md decision 3 with its matrix output and the draft rule. Verify with a throwaway branch that changes only a file under `openspec/`, whose run builds only `editorconfig`, and with the draft pull request, whose run has only the x86 leg.
- [x] 2.2 Add the aggregate job `Verdict` of design.md decision 4. Verify that `Verdict` passes on the draft and on a ready pull request and fails on a throwaway branch with a failing job.
- [x] 2.3 Replace the job `verify` and the check `Verify` in `README.md` with `build` and `Verdict`. Verify with `grep -n -i verify README.md`.

## 3. Documentation and rules

- [x] 3.1 Replace the rule on archiving in `AGENTS.md` with the rule of design.md decision 6, and update the section "Pull requests" to the jobs `build`, `e2e` and `Verdict`. Verify that `AGENTS.md` no longer mentions `[skip ci]`, `verify` or `nixos` as jobs.
- [x] 3.2 Update "Testing" in `docs/arc42/08-cross-cutting-concepts.md` as design.md lists. Verify that every link resolves.

## 4. Hand-over

- [x] 4.1 Run `openspec-verify-change` for `verify-both-architectures` and fix what it finds.
- [x] 4.2 Run a code review of the whole branch with the review command of the agent and fix what it finds.
- [x] 4.3 Run `nix build .#checks.<system>.integration` for the local system and fix every failure.
- [x] 4.4 Rebase onto `main`, push and mark the pull request ready for review. The description asks a maintainer to replace `Verify` with `Verdict` in the ruleset of `main` before the merge and to rebase open pull requests afterwards.
