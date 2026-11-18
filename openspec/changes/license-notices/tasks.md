# Tasks

Requires the change `license-compliance` on `main`, because it creates `server/deny.toml`, `client/src/shadcn.css` and puts `nss-cacert` into the image.

## 1. Notices of the client

- [x] 1.1 Set `build.license` in `client/vite.config.ts` with `fileName: 'licenses.txt'`. Add `client/notices.json` and `client/licenses/` with entries for `src/shadcn.css` and `src/components/ui/` (shadcn 4.23.0, MIT), `tailwindcss` and `tw-animate-css` from `node_modules`, and texts for `@fluent/bundle` (Apache-2.0) and `react-remove-scroll-bar` (MIT). Add a Vite plugin that appends one section per entry and fails the build on a bundled package without license text and on an entry whose path or package does not exist. Add `<link rel="license" href="/licenses.txt">` to `client/index.html`. Name in the README where copied code goes and that it needs an entry. Verify that `npm run build` writes `dist/licenses.txt` with texts for `react`, `tailwindcss`, `@fluent/bundle` and shadcn, that the build fails when the entry for `@fluent/bundle` is removed and when an entry names a missing path
- [x] 1.2 Send `text/plain; charset=utf-8` for `.txt` assets in `server/src/client.rs`. Add an integration test that requests `/licenses.txt` and asserts that content type, `react`, `shadcn` and the MIT text. Verify that it passes with `cargo test` after `npm run build` without Nix and in `nix build .#checks.<system>.integration`

## 2. Notices of the crates and the standard library

- [x] 2.1 Add a static `server/about.toml` with `targets` for the two Linux targets and its own `accepted` list, a plain text template and a check in `lint` that compares `accepted` with `server/deny.toml` through `builtins.fromTOML`. Add a derivation in `flake.nix` that runs `cargo-about` offline over the crates that crane vendors and appends its output to the `licenses.txt` of the client before the server embeds it. Add the command without Nix to the README. Name the comparison in the row `lint` of the testing table in `docs/arc42/08-cross-cutting-concepts.md` with the skill `arc42`. The commit names `cargo-about`, its license and why the project needs it. Verify that `licenses.txt` of the built server names `axum` and `jiff` with license texts, that the build fails when a crate's license is outside the list and that the check fails when `about.toml` and `deny.toml` differ
- [x] 2.2 Take `COPYRIGHT-library.html` from the prebuilt `rustc` in nixpkgs, add a check in `lint` that its version equals the `rustc` of the server, add it to `dist` as `licenses-rust.html` and name it with the version of Rust in `licenses.txt`. Copy both files next to `LICENSE` in the release archive. Verify that the built server serves `/licenses-rust.html` with `hashbrown` and that the release archive contains both files

## 3. Packages from nixpkgs in the image

- [x] 3.1 Confirm the runtime closure of the image with `nix path-info -r`. Add the closure list to `flake.nix` with license files and source of each package, and the directory `/share/licenses` in the image with both notice files, the texts and a source reference per package with the nixpkgs revision of `flake.lock`, upstream URL and hash. Add the closure check to `buildChecks` on Linux only. Name it in the row `build` of the testing table and update `docs/arc42/07-deployment-view.md` with the skill `arc42`. Name `/licenses.txt`, `/licenses-rust.html` and how to copy `/share/licenses` out of the image with `podman create` and `podman cp` in `docs/operating/installation.md`. Verify that the check passes, fails when a package is added to the image and fails when an entry no longer matches the closure, and that the documented commands run as written
- [x] 3.2 Assert in `nix/test.nix` on the node `container` that `/share/licenses` holds `licenses.txt` and an entry for glibc. Assert on every node that `/licenses.txt` contains `axum` and `react` and that `/licenses-rust.html` responds, and on the node `binary` that the served `/licenses.txt` equals the one in the release. Verify that `nix build .#checks.x86_64-linux.nixosVmTest` passes

## 4. License check for tools from nixpkgs

- [x] 4.1 Define one attribute set of every tool the flake takes from nixpkgs and let the development shell, the checks and `nix/test.nix` take their packages from it. Drop `pkgs.curl` from `nix/test.nix`, so the nodes use `curl` from the NixOS base system. Verify that `nix build .#checks.<system>.integration` and `nix build .#checks.x86_64-linux.nixosVmTest` pass
- [x] 4.2 Add the check to the target `lint` that reads `meta.license` of every package in the set and of the closure list, the latter from the Linux package set, and compares it with `server/deny.toml` plus a list in `flake.nix` for licenses allowed only for packages from nixpkgs. Start that list from the licenses the packages use today, each approved by the OSI. Name the check in the row `lint` of the testing table with the skill `arc42`. Verify that the check passes on Linux and macOS, fails by package name for a package with a license on neither list and fails for a package without `spdxId`

## 5. Decision record and risks

- [x] 5.1 Write `docs/decisions/0017-third-party-notices-in-every-artifact.md` from the draft in design.md and add it to `docs/arc42/09-architecture-decisions.md`. Remove the row on missing notices from `docs/arc42/11-risks-and-technical-debt.md`. Verify that every relative link resolves and that `nix build .#checks.<system>.integration` passes

## Workflow follow-up

- Archive the change on `main` after the pull request is merged.
