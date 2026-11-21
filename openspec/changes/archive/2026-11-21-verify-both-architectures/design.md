# Design

## Context

- `ci.yaml` (workflow name `CI`) runs `verify`, `e2e` and `nixos` only for `x86_64-linux`. `verify` uploads the bare release binary, which loses its executable bit, and `e2e` tests it.
- `nixos` restores the store from the cache with `save: false`. In 4 of the last 20 runs it rebuilt client and server derivations, so the VM test then checked a binary it built again instead of the one `verify` uploaded.
- The Nix cache key holds `runner.os` but not the architecture and hashes no sources, so an exact hit on `main` never saves a newer store. The Playwright cache key holds no architecture either. The caches of Playwright and pnpm are saved by pull requests too. The repository uses 9.52 of its 10 GB of cache. `release.yaml` restores the Nix prefix `nix-Linux-`, which would also match a key for arm64.
- On `main`, one `concurrency` group keeps only the newest pending run, so some commits on `main` get no run.
- The ruleset of `main` requires the check `Verify`. A matrix renames that check. `e2e` and `nixos` are not required at all.
- Every workflow runs for every path. The check `editorconfig` covers `openspec/` and is exposed on its own as `checks.<system>.editorconfig`. It needs only `editorconfig-checker` from the binary cache. `AGENTS.md` has OpenSpec changes archived directly on `main` with `[skip ci]`.
- `ci.yaml` evaluates every flake output with `nix flake check --no-build --option abort-on-warn true`.
- GitHub offers `ubuntu-24.04-arm` as a hosted arm64 runner for public repositories at no cost. It exposes no `/dev/kvm`. The issue that asked for KVM there was closed as not planned [1].
- The flake already lists `aarch64-linux`. `nix build .#packages.aarch64-linux.image .#checks.aarch64-linux.imageClosure` succeeds unchanged.

[1]: https://github.com/actions/runner-images/issues/14062

## Goals / Non-Goals

**Goals:**

- Each architecture is built once per run, and every check of that run looks at those bytes.
- A later workflow can publish the artifacts of a run without building them again.

**Non-Goals:**

- Publishing. The change `publish-nightly-and-releases` covers it.
- The NixOS VM test on `aarch64`. It needs KVM.
- Moving the end-to-end tests into the flake.

## Decisions

### 1. One job per architecture builds and checks

`build` is a matrix over `x86_64-linux` on `ubuntu-24.04` and `aarch64-linux` on `ubuntu-24.04-arm`, with `fail-fast: false`. Each leg first runs `nix flake check --no-build --option abort-on-warn true`, then one `nix build --keep-going --option abort-on-warn true` of `checks.<system>.integration`, `packages.<system>.release` and `packages.<system>.image`. The x86 leg adds `checks.x86_64-linux.nixosVmTest` to the same call and replaces the job `nixos`.

Each leg uploads two artifacts. Other workflows rely on their names:

|Artifact|Content|Retention|
|-|-|-|
|`release-<system>`|the release binary in a tarball, which keeps its executable bit|30 days for a push to `main`, one day otherwise|
|`image-<system>`|the image archive|one day|

The workflow keeps the name `CI`, because `publish.yaml` refers to it by name.

Separate jobs for checks, VM test and artifacts would each restore a store of several GB and could rebuild what another job built. QEMU on an x86 runner would build Rust many times slower, and `pkgsCross` would need a second build path in the flake.

Not architecturally significant, because how CI splits its jobs is tooling.

### 2. End-to-end tests and an image start on both architectures

`e2e` is the same matrix with `fail-fast: false`. It unpacks the binary of its architecture and runs Playwright against it. It then loads the image with Docker, starts it against PostgreSQL from `compose.yaml` and waits for `/api/health`. On `aarch64` this start is the only test of the image, because the VM test needs KVM.

Not architecturally significant, because where tests run is tooling.

### 3. Changes to `openspec/` alone run only `editorconfig`

A job `changes` compares, for a pull request, its base with its head and, for a push, `github.event.before` with the pushed commit, using `git diff --name-only --no-renames`, so a push of several commits counts all of them and a file moved into `openspec/` still counts as a change at its old path. If `github.event.before` is missing, consists of zeros or is not an ancestor of the pushed commit, `changes` runs everything. If every changed path lies under `openspec/`, it runs `nix build .#checks.x86_64-linux.editorconfig`, and every job but `Verdict` skips. Such a commit on `main` uploads no artifacts and gets no image. A run started by `workflow_dispatch` never skips. A filter on the workflow itself would leave the required check without a report and block such a pull request for good.

`changes` also writes the matrix of `build` as an output, because a job condition cannot see the matrix. A draft pull request gets only the x86 leg, and the steps for image and VM test skip there.

Not architecturally significant, because it changes only what CI runs.

### 4. One aggregate check

A job `Verdict` needs every other job and runs always, also when `changes` skipped the others. It fails if one failed or was cancelled, including `changes` with its check `editorconfig`, and passes when the others were skipped by `changes` or by the rule for drafts. The ruleset of `main` requires only `Verdict`, so the matrix can change without touching the ruleset. A draft run reports `Verdict (draft)` instead, so a passing draft run on the same commit cannot stand in for the full run after the pull request turns ready. The name is new on purpose. A pull request still open on the old workflow reports the old job `Verify`, which must not satisfy the new requirement. `Verdict` names the result of the run and differs from the workflow `CI`. Everything beyond the x86 leg of `build` starts once a pull request is ready for review.

Not architecturally significant, because it concerns CI and repository settings.

### 5. Concurrency and caches

Pull requests cancel older runs of the same pull request. On `main`, each pushed commit has its own `concurrency` group, so every push gets a run and a later workflow can find the artifacts of every pushed commit. Merges are the only pushes to `main`, so every merge commit has its artifacts. A commit inside a merged branch has none, and a release can only use a merge commit.

The Nix cache key holds `runner.os`, `runner.arch` and a hash of the sources, so `release.yaml` with its prefix `nix-Linux-` matches it no more. That prefix moves to `nix-Linux-X64-` in this change. The Playwright key gains `runner.arch`. Every cache restores everywhere and saves only on a push to `main`. The Nix cache then purges older entries of its prefix, which needs `actions: write` on the saving job. Each architecture caps its store at about 4 GB. Each `cache-nix-action` step carries `zizmor: ignore[cache-poisoning]`, because no run on `main` restores a cache that a pull request saved.

Not architecturally significant, because it is CI tooling.

### 6. Archive merged changes together

`AGENTS.md` no longer archives each OpenSpec change on `main`. A maintainer archives merged changes together in a pull request of its own, when all changes of a piece of work are merged or whenever they choose. That pull request touches only `openspec/`, so decision 3 runs only `editorconfig` for it. Nobody pushes to `main` directly, and `[skip ci]` is no longer needed.

Not architecturally significant, because it is a rule of the development process.

### 7. This change iterates on a draft pull request

The workflow can be checked only by running it. This change therefore deviates from `AGENTS.md` and pushes early to a draft pull request. Throwaway branches that test `changes` and a failing job target its branch. The pull request turns ready for review once every check holds.

Not architecturally significant, because it concerns the workflow of one change.

## Affected arc42 sections

- `08-cross-cutting-concepts.md`: "Testing" describes the matrix, the VM test in the x86 job, the image start on both architectures, `changes`, the aggregate check and that the VM test runs only on `x86_64` because the arm64 runner has no KVM.

No decision is architecturally significant, so the change drafts no record.

## Risks / Trade-offs

- [The NixOS module and the example unit stay untested on `aarch64`.] → The image start and the end-to-end tests cover the binary and the image. "Testing" names the gap.
- [Two stores and the caches of Playwright and pnpm share 10 GB.] → Only `main` saves, each store is capped and older entries are purged. Measure the stores on the first runs and adjust the cap.
- [A merge commit on `main` differs from the head of its pull request, and caches do not cross refs.] → `main` builds once more, incrementally.
- [`e2e` waits for both `build` legs, including the VM test, because `needs` cannot name one leg.] → The end-to-end tests start later than today. Wall time costs nothing.
- [The pull request of this change reports only `Verdict`, so the required check `Verify` never reports for it.] → A maintainer replaces `Verify` with `Verdict` in the ruleset before merging this change. Every pull request still open on the old `ci.yaml` then needs a rebase onto `main`.
- [Changes planned in parallel touch `ci.yaml` or rely on the new checks.] → Branch them after this change is merged, or rebase them onto it.
- [Playwright or the images in `compose.yaml` may not run on arm64.] → The draft pull request shows it before the merge.
