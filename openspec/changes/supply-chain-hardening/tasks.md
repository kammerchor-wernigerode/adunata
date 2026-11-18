# Tasks

## 1. Prerequisite

- [ ] 1.1 Once `license-compliance` is merged, pull `main` and rebase this branch onto it. Verify that `server/deny.toml` exists, that `flake.nix` has the check `rustLicenses` in the target `lint`, that `npm ls shadcn` in `client/` finds nothing and that `server/Cargo.toml` and `server/Cargo.lock` hold `lettre` 0.11.24 from the separate fix

## 2. npm configuration

- [ ] 2.1 Before you add any file, verify in `client/` that `npm ci --ignore-scripts`, `npm run build`, `npm test` and `npm run e2e` pass and that the Playwright tests start their browsers. Stop and report if one fails
- [ ] 2.2 Add `client/.npmrc` with `min-release-age=7` and `ignore-scripts=true` and stage it with `git add`, because the fileset of the client takes only tracked files. If the Nix build of the client fails on `min-release-age`, set `npm_config_min_release_age = "0"` in `clientPackage` in `flake.nix`, as design.md describes. Verify that `npm ci` runs no install script, that `npm install some-package@<version>` fails for a version younger than 7 days, that `npm run e2e` passes and that `nix build .#checks.<system>.integration` passes

## 3. cargo-deny bans and sources

- [ ] 3.1 Extend `server/deny.toml` with `[bans]`, `[sources]` and `[advisories]` as design.md describes. Rename the check `rustLicenses` to `rustDependencies` in `flake.nix` and wherever `git grep rustLicenses` finds it and drop its attribute `cargoDenyChecks`, so crane's default applies. Update the row `lint` in `docs/arc42/08-cross-cutting-concepts.md` with the skill `arc42` and the command without Nix in the README to `cargo deny check bans licenses sources`. Verify that the check passes, that it fails when a dependency of `server/Cargo.toml` points to a git repository or has the version `*`, that `npm run lint:licenses` still reads the list of `[licenses]` and that `nix build .#checks.<system>.integration` passes

## 4. zizmor

- [ ] 4.1 Set `persist-credentials: false` on every `actions/checkout` in `ci.yaml` and `release.yaml`. Change both `nix run nixpkgs#skopeo` in `release.yaml` to `nix run --inputs-from . nixpkgs#skopeo`. Add `# zizmor: ignore[cache-poisoning]` with the reason from design.md to the step `cache-nix-action` in `release.yaml`. Verify that `zizmor --offline .github/workflows` from the nixpkgs of the flake reports no finding
- [ ] 4.2 Add the check `workflows` to the target `lint` in `flake.nix`. It runs zizmor from nixpkgs with `--offline` over a source of its own that holds only `.github`. The commit names zizmor, its MIT license and why the project needs it. Name zizmor in the row `lint` of `docs/arc42/08-cross-cutting-concepts.md` with the skill `arc42` and in the README sentence on checks that need Nix. Verify that the check passes, that it fails when a checkout loses `persist-credentials: false`, that an edit of a workflow does not rebuild `server` and that `nix build .#checks.<system>.integration` passes

## 5. Advisory workflow

- [ ] 5.1 Add `client/scripts/check-advisories.mjs` and `client/audit-exceptions.json` with the exception for `GHSA-fc5q-39xc-vjpf`, its reason and an expiry date three months ahead. The script reads the output of `npm audit --json` from a file and behaves as design.md describes. It has no npm dependencies. Verify with `npm audit --json > audit.json` and `node scripts/check-advisories.mjs audit.json` that it passes, that it fails on an output with an `error` key, that `vite` and other entries without an advisory of their own do not count, that it fails when the exception is removed or its date lies in the past, that an exception naming an ID that no advisory has fails with `GITHUB_EVENT_NAME=schedule` and only warns with `GITHUB_EVENT_NAME=pull_request` and that it passes when only an advisory of severity moderate remains
- [ ] 5.2 Add `.github/workflows/advisories.yaml` as design.md describes, with actions pinned to SHAs and `permissions: contents: read`. In `docs/arc42/08-cross-cutting-concepts.md`, name the workflow as the second check outside the flake and in the paragraph on CI jobs, with the skill `arc42`. Verify that the check `workflows` passes, that the run the pull request triggers passes, because it changes the workflow itself, and that `nix build .#checks.<system>.integration` passes. After the merge, start one run through `workflow_dispatch` and verify that it passes

## 6. Dependabot

- [ ] 6.1 Add `.github/dependabot.yml` with the four ecosystems, cooldowns and groups from design.md. Verify that the check `workflows` audits the file and passes, that each cooldown of `npm` is at least `min-release-age` in `client/.npmrc` and, after the merge, that the Dependabot page of the repository shows no configuration error
- [ ] 6.2 Add a section on dependency updates to the README. It covers how to handle Dependabot pull requests, how to set the pins of Rust, `editorconfig-checker` and `cargo-deny` after a `flake.lock` update, that the refs of `nixpkgs` and `crane` in `flake.nix` move only by hand, how to install a security fix younger than `min-release-age`, how to add an advisory exception with an expiry date, the daily advisory run and who it notifies as well as the repository settings the project expects. Add a sentence under pull requests in `AGENTS.md` that exempts pull requests opened by Dependabot from the rules on drafts and red checks. Add the row on scanning only the lock files to `docs/arc42/11-risks-and-technical-debt.md` with the skill `arc42`. Verify that every relative link resolves and that `nix build .#checks.<system>.lint` passes

## 7. Repository settings

- [ ] 7.1 A maintainer enables Dependabot security updates, secret scanning and push protection under Settings, Advanced Security. Verify that the settings page shows all three as enabled and that they match the list in the README
- [ ] 7.2 A maintainer adds the two rulesets from design.md, one for `main` that requires the check `Verify` and one that restricts tags `v*`, both with a bypass for repository admins. Verify with `gh api repos/{owner}/{repo}/rulesets` that both exist and are active, that GitHub marks a test pull request with a failing `Verify` as blocked and that the list in the README matches

## Workflow follow-up

- Archive the change on `main` after the pull request is merged.
