# Risks and technical debt

The risks are ordered by priority.

|Risk|Mitigation|
|-|-|
|Only `server/Cargo.lock` and `client/pnpm-lock.yaml` are scanned for advisories. The OCI image has no package database of an operating system, the release binary carries no list of its crates and the packages from nixpkgs in image and VM test are outside every scan.|Dependabot updates `flake.lock` weekly within the NixOS release, so fixes in nixpkgs arrive with the next release of Adunata. The workflow `advisories` covers the crates and client packages that the binary embeds.|
|Coding agents write Rust that compiles but is hard to change. The maintainer is new to Rust and reviews structure rather than individual lines.|The rules for plain Rust in [`AGENTS.md`](../../AGENTS.md), strict lints that fail on any warning and checked module boundaries.|
|Fluent, the localization format on server and client, is maintained slowly. Its React binding saw no release since 2023.|Adunata uses only the core libraries of Fluent and its own small React binding. The catalogs stay plain Fluent files, so another library that reads Fluent could replace them.|
|Fluent in Rust has no date formatting. The catalogs spell out date ranges in mail with one message each for a single day, a range within a month, a range across months and a range across years.|Each language needs all four messages, and tests cover them in German and English. If more languages or formats are needed, ICU4X can take over date formatting on the server.|
|The NixOS module and the example unit are not tested on `aarch64`, because the hosted arm64 runner of GitHub has no KVM for the NixOS VM test.|CI builds and runs the unit tests, integration tests and end-to-end tests on `aarch64`, including the release binary and the image. Only the setup by module and unit stays untested there, and it holds no code that differs by architecture.|
|UI components copied into the repository get no fixes from their upstream project by themselves.|The project changes copied components only where it must, so that a newer upstream version can replace them.|
