# Design

## Context

See proposal.md for the motivation. Today the artifacts carry only Adunata's own `LICENSE`. The fonts are the single exception. `fontLicenses()` in `client/vite.config.ts` emits `assets/LICENSE-<font>.txt`. The binary embeds `client/dist` through `rust-embed` in `server/src/client.rs` and sends the MIME type of each file without a charset. An integration test there asserts that both font licenses are served.

After `license-compliance`, `server/Cargo.lock` holds 264 packages including `adunata` itself. The lock file also holds crates for targets the server is never built for, such as `r-efi`. The binary contains the Rust standard library, which is not in `Cargo.lock`. Rust ships `COPYRIGHT-library.html` with the copyright notices of the standard library and of its own third-party dependencies, such as `hashbrown`, `gimli` and `libc`. Time zone data comes from `jiff` with `tzdb-bundle-always`, so it is covered as a crate.

The client bundle holds 58 packages by Vite's count. Vite's `build.license` reads the license file of each package in the JavaScript chunks and skips assets. `tailwindcss` and `tw-animate-css` reach the browser only through the stylesheet, so it misses them. `@fluent/bundle` and `react-remove-scroll-bar` ship no license file, so they appear without text. After `license-compliance`, `client/src/shadcn.css` is a copy of `shadcn/tailwind.css` 4.23.0 under MIT. Its notice sits in a plain comment that the build drops. The components in `client/src/components/ui/` were copied from shadcn too. No lock file lists either.

The OCI image from `dockerTools.buildLayeredImage` contains the runtime closure of the server and, after `license-compliance`, `nss-cacert` through `SSL_CERT_FILE`. For glibc alone, the cache of NixOS lists `glibc`, `xgcc-libgcc`, `libidn2` and `libunistring`. With libgcc_s and `nss-cacert`, the image holds about seven packages from nixpkgs. The image has no shell. The release archive contains the binary only, because ADR 0015 loads glibc and libgcc_s from the distribution.

Copyleft code ships only in the image, through glibc, `libidn2`, `libunistring` and `nss-cacert`. No crate and no bundled npm package needs MPL or LGPL.

The flake and `nix/test.nix` name tools from nixpkgs one by one. After `license-compliance` and this change they include `cacert`, `cargo-deny`, `cargo-about` and `openssl`. `nix/test.nix` names `curl`, whose license of the same name the OSI has not approved. The base system of every NixOS node puts `curl` into `environment.systemPackages` anyway.

`license-compliance` creates `server/deny.toml` with the allowed licenses and checks the lock files against it. It defers license files, copied code and packages from nixpkgs to this change.

## Goals / Non-Goals

**Goals:**

- Each artifact meets the notice conditions of the licenses of the code it contains.
- No crate, no npm package in the client and no package from nixpkgs in the image ships without a notice. A check fails when one would.

**Non-Goals:**

- A machine-readable software bill of materials in CycloneDX or SPDX.
- Notices for tools that build or test Adunata and never ship. The license check for them is in scope, the notices are not.
- The transitive build closure of nixpkgs, such as the compilers of stdenv, and the base system of the NixOS test nodes. The license check covers the set of tools the flake takes from nixpkgs.
- A check that finds new copies of third-party code. Review catches them.
- A visible link to the notices in the user interface.

## Decisions

### `cargo-about` for the crates

`cargo-about` from nixpkgs renders the license texts of every crate from the crate sources, not from the declared field. It runs offline in a derivation over the crates that crane vendors, with a static `server/about.toml` and a plain text template. `about.toml` limits `targets` to the two Linux targets and lists its own `accepted` licenses, because `cargo-about` cannot read `deny.toml`. A check in `lint` compares both lists through `builtins.fromTOML`, so they cannot drift. The static file lets the command run without Nix too.

Alternatives: `cargo-cyclonedx` produces an SBOM without license texts. A script over `cargo metadata` would have to find and pick license files per crate, which `cargo-about` already does.

Significant: no. Which tool collects the texts is invisible to operators.

### `COPYRIGHT-library.html` for the Rust standard library

The notices of the standard library are the file `COPYRIGHT-library.html` of the Rust release, served as `licenses-rust.html` next to `licenses.txt`. The Rust project maintains it per release, and it lists the third-party dependencies of the standard library that `Cargo.lock` does not see. The `rustc` that nixpkgs builds from source generates this file offline and leaves the list of those dependencies empty, so the file comes from the prebuilt release in `rust.packages.prebuilt`, which nixpkgs uses to bootstrap. A check in `lint` fails when the version of that release differs from the `rustc` that builds the server. `licenses.txt` names the file and the version of Rust.

Alternatives: converting the HTML to text keeps one file but adds a converter and about 400 KB. `LICENSE-MIT`, `LICENSE-APACHE` and `COPYRIGHT` alone miss the third-party parts of the standard library.

Significant: no on its own. The record below covers what operators see.

### Vite `build.license` and a manifest for the client

Vite's `build.license` writes `licenses.txt` at the root of `dist` with the texts of the packages in the JavaScript bundle. `client/notices.json` supplies what Vite cannot see. Each entry is one of three kinds.

- Copied code, with path or directory, source, version, SPDX license and copyright notice. The first entries are `src/shadcn.css` and `src/components/ui/`, both shadcn 4.23.0 under MIT.
- An npm package that reaches the browser outside the JavaScript bundle, whose license file the plugin reads from `node_modules`. The first entries are `tailwindcss` and `tw-animate-css`.
- A license text under `client/licenses/` for a package that ships none. The first entries are `@fluent/bundle` under Apache-2.0 and `react-remove-scroll-bar` under MIT, with the copyright from their repositories.

A Vite plugin in `client/vite.config.ts` appends one section per entry to `licenses.txt`. It fails the build when a package in Vite's output has no license text and no entry supplies one, and when an entry names a path or package that does not exist. The header of `shadcn.css` stays for readers of the source. `fontLicenses()` stays as it is. The fonts may move into the manifest later.

`client/index.html` gets `<link rel="license" href="/licenses.txt">`, so the document a browser receives points to the notices without visible interface or catalog texts.

Alternatives: an own plugin over all `dependencies` in `package.json` includes unused packages such as `zod` and still needs texts for packages without a file. A second fixed list in `vite.config.ts` next to the fonts adds a third mechanism. A header with `/*! */` per copied file scatters the notices across the bundle.

Significant: no on its own. The record below covers what operators see.

### Notices served at fixed paths

A derivation in `flake.nix` appends the output of `cargo-about` to the `licenses.txt` of the client build and adds `licenses-rust.html`, before the server embeds `dist`. `rust-embed` serves both like the font licenses. The rule in `client.rs` already serves a path with a dot as an asset. `client.rs` sends `text/plain; charset=utf-8` for `.txt` files, because the notices contain characters such as `©`. The release archive copies both files next to `LICENSE`.

Alternatives: a subcommand `adunata licenses` reaches operators but not browsers, which receive the client. A visible link in the client footer needs texts in every catalog.

Significant: yes. Operators and browsers see new public paths, and every installation method now ships notices.

### Closure of the image against a maintained list

`flake.nix` keeps a list of the packages from nixpkgs that the image may contain. Each entry names the package, any license exception that its declared license leaves out, such as the runtime exception of GCC, and, where nixpkgs exposes none, its source. The license texts are files in `nix/licenses/`, copied once from the sources of glibc, GCC and NSS. The seven packages need only six texts and the notices of glibc, and the texts do not change between releases. The SPDX list in nixpkgs would avoid the copies, but it is under CC0-1.0, which the OSI has not approved. A check in the target `build`, on Linux only like `releaseLinkage`, computes the runtime closure of the image with `closureInfo`, maps each store path to its package name and version and fails on a path the list does not name. It also fails on an entry that is no longer in the closure, so the list stays exact. The image gets a directory `/share/licenses` with both notice files and, per package from nixpkgs, its license texts and a file that names the nixpkgs revision of `flake.lock`, the URL and hash of the upstream source and the store path.

The image has no shell, so the operating guide names a command that copies `/share/licenses` out of the image, such as `podman create` followed by `podman cp`.

The source reference is the answer to the source obligations of LGPL and MPL-2.0 in the image. Both sources are public and fixed by hash, and the nixpkgs revision names the patches too. ADR 0017 records this as a position. Attaching source archives to each release would add well over 100 MB. A written offer would bind the project to answer requests for years.

The check needs the built image, so it cannot run in `lint`, which builds on nothing and runs on macOS too.

Alternatives: generating entries from `meta.license` scales to large closures but has no copyright lines, and the closure has about seven packages. `sbomnix` lists the closure but carries no texts.

Significant: yes, part of the record below. An operator of the image finds the license texts and the source of every system package in it.

### One set of tools from nixpkgs and its license check

`flake.nix` defines one attribute set with every tool it takes from nixpkgs. The development shell, the checks and `nix/test.nix` take their packages from it. Builders such as `runCommand` and `dockerTools` are no tools in this sense. A check in the target `lint` maps over the set and over the entries of the closure list and reads `meta.license` of each. It compares each `spdxId` with the list from `server/deny.toml`, read with `builtins.fromTOML`, plus a short list in `flake.nix` of licenses allowed only for packages from nixpkgs. That list starts with what the packages use today, such as `LGPL-2.0-or-later`, `LGPL-3.0-or-later`, `GPL-2.0-or-later`, `GPL-3.0-or-later` and `PostgreSQL`, each checked against the list of the OSI. When `meta.license` lists several licenses, each must be allowed. A package without an `spdxId`, or with a license on neither list, fails the check by name. The closure entries are evaluated from the Linux package set, as `builtForLinux` does, because libgcc does not exist on macOS. The check is pure evaluation and runs on every system.

`nix/test.nix` drops `pkgs.curl` without a replacement. The test nodes keep `curl` from the NixOS base system, which is outside the check like stdenv.

The extra list keeps crates and npm packages restricted to the licenses they use today, so a copyleft crate still fails the gate of `license-compliance` and needs a decision.

Alternatives: one shared list in `server/deny.toml` lets a GPL crate pass without anyone deciding on it. Replacing `curl` with `xh` adds a dependency while `curl` stays in the base system. A check that greps every `pkgs.<name>` needs exceptions for every builder.

Significant: no. It is a check in CI.

### Affected arc42 sections

- `07-deployment-view.md`: each artifact carries the notices, and the image carries license texts and sources of its packages from nixpkgs.
- `08-cross-cutting-concepts.md`: the row `lint` of the testing table names the license check for tools and the comparison of `about.toml` with `deny.toml`. The row `build` names the closure check.
- `09-architecture-decisions.md`: the new record.
- `11-risks-and-technical-debt.md`: the row on missing notices goes.

### Draft decision record: ADR 0017 Third-party notices in every artifact

Supersedes none. The number follows ADR 0016 of `license-compliance`.

**Context.** The binary, the OCI image and the release archive contain third-party code under MIT, Apache-2.0, BSD and similar licenses, which require that their texts and copyright notices travel with the code. The image also contains glibc, `libidn2`, `libunistring` and a CA bundle from nixpkgs under LGPL and MPL-2.0, which also require access to the source. Browsers receive the client, so they are recipients too. The client contains code copied from shadcn, which no lock file lists. The image has no shell.

**Decision.** We will build the notices from the license files of every crate compiled into the server, the copyright file of the Rust standard library, every package of the client and every entry of a manifest for what no lock file shows. Every instance serves them at `/licenses.txt` and `/licenses-rust.html`, and the client links them. The release archive carries both files next to `LICENSE`. The OCI image carries them in `/share/licenses` together with the license texts and a reference to the exact source of each package from nixpkgs in the image, and a check fails when a package enters the image without such an entry. We will answer the source obligations of the image with that reference, because the sources are public and fixed by hash.

**Status.** Accepted.

**Consequences.**
- Every artifact carries the notices of its dependencies and copied code.
- Anyone can read the notices of a running instance without signing in.
- A new dependency of the server or the client appears in the notices with the next build, and one without license text fails the build.
- Copying third-party code into the repository needs an entry in the manifest. Review catches a copy without one.
- A new package in the image fails the build until someone adds its license texts and source.
- The release archive carries no texts for glibc and libgcc_s, because the distribution provides them.
- If a public source behind a reference disappears, the project has to provide it on request. The reference names the hash, so any copy serves.

## Risks / Trade-offs

- [`cargo-about` finds no license file for a crate that declares a license only in `Cargo.toml`] → It falls back to the canonical SPDX text, and `about.toml` names any crate that needs a manual clarification.
- [A package from the manifest moves into the JavaScript bundle or a new package reaches the browser only through the stylesheet] → The first case lists it twice, which is harmless. The second case is not caught by the build. Review of changes to `index.css` catches it.
- [A `flake.lock` update changes the closure, for example a new layout of libgcc] → The closure check fails with the unexpected path. This is the intended trigger to update the list.
- [A tool is used outside the set] → The check does not see it. The set is the single place the Nix files take tools from, so review catches a `pkgs.<name>` outside it.

## Migration Plan

Nothing to migrate. The next release carries the notices. Rollback is the previous release.
