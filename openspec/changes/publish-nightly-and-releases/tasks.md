# Tasks

## 1. Version and archive

- [ ] 1.1 Let `flake.nix` read the version from `server/Cargo.toml`, set `server/Cargo.toml`, `server/Cargo.lock` and `client/package.json` to `0.0.0` and add the version check of design.md decision 2 to the target `lint`. Verify by changing one version locally, then adding `+1` to all three, and seeing `nix build .#checks.<system>.lint` fail both times and pass again after the revert.
- [ ] 1.2 Add `packages.<system>.releaseArchive` with its `.sha256` file as design.md decision 3 describes. Verify that `nix build --rebuild` gives the same hash, on a Linux builder or by the run of the pull request.
- [ ] 1.3 Let `build` in `.github/workflows/ci.yaml` build and upload `releaseArchive` as `release-<system>` and `e2e` unpack it. Verify by the run of the pull request on both architectures.

## 2. Publish and release

- [ ] 2.1 Add `.github/workflows/publish.yaml` as design.md decision 4 describes. Verify with `zizmor` through the lint target and by reading it against the scenarios of "Nightly image".
- [ ] 2.2 Rewrite `.github/workflows/release.yaml` as design.md decision 5 lists, with the dry run. Verify with `zizmor` and by dry runs after the merge for a tag on a commit that does not raise the version and a tag outside `main`, which both stop with their message.

## 3. Documentation

- [ ] 3.1 Describe both architectures, the tags `nightly`, `sha-<commit>`, `latest` and the version, the annotations and the archive per architecture with `SHA256SUMS` in "Container" and "Release binary" of `docs/operating/installation.md`, and fix its introduction that a NixOS test runs every installation. Say that `nightly` follows `main` and suits test instances. Verify that the archive names match `releaseArchive`.
- [ ] 3.2 Describe in `README.md` how a maintainer cuts a release, including the dry run, and add the public package to the settings that no file holds. Verify that the steps match `release.yaml`.
- [ ] 3.3 Write the record "Published artifacts, their tags and the project version" from the draft in design.md with the next free number in `docs/decisions/`, add it to `docs/arc42/09-architecture-decisions.md` and update `07-deployment-view.md`, `08-cross-cutting-concepts.md` and `11-risks-and-technical-debt.md` as design.md lists. Check the number again after the last rebase onto `main`. Verify that every link resolves.

## 4. Hand-over

- [ ] 4.1 Run `openspec-verify-change` for `publish-nightly-and-releases` and fix what it finds.
- [ ] 4.2 Run a code review of the whole branch with the review command of the agent and fix what it finds.
- [ ] 4.3 Run `nix build .#checks.<system>.integration` for the local system and fix every failure.
- [ ] 4.4 Rebase onto `main`, push the branch and open the pull request ready for review. The description names the manual step to make the package public after the first publish and asks a maintainer to watch the first run of `publish.yaml`.
