# Proposal

## Why

The binary, the OCI image and the release archive ship code from hundreds of crates and npm packages, code copied from shadcn and, in the image, glibc, libgcc and a CA bundle from nixpkgs. Their licenses require that a copy of the license and its copyright notices travel with the code. LGPL and MPL-2.0, which apply in the image, also require a way to obtain the source. None of the artifacts carries them today, which the first row of the risk register names. The license gate of the change `license-compliance` checks only `Cargo.lock` and `package-lock.json`, so packages from nixpkgs, including the tools of the flake, are checked by no one.

## What Changes

- The build generates `licenses.txt` with the license texts and copyright notices of every crate compiled into the server and of every npm package that reaches the browser, through the JavaScript bundle or the stylesheet. The copyright notices of the Rust standard library ship as `licenses-rust.html`, the file the Rust project maintains for it.
- A manifest in the client names the notices that no lock file or bundle provides. These are code copied into the repository, such as `client/src/shadcn.css` and the UI components in `client/src/components/ui/`, packages that reach the browser only through the stylesheet and license texts for packages that ship none. The build fails when a package in the bundle has no license text.
- The binary embeds both files and serves them over HTTP at fixed paths, next to the font licenses it already serves. The HTML document of the client links `/licenses.txt` as its license.
- The release archive carries both files next to `LICENSE`. The OCI image carries them together with the license texts and source references of the packages from nixpkgs in its runtime closure.
- A check in the target `build` compares the runtime closure of the image with a maintained list of packages. A package that enters the closure without an entry fails the check, so it cannot ship without its notice.
- The flake takes every tool from nixpkgs from one set, and a check in the target `lint` compares their declared licenses with the allowed list in `server/deny.toml`. The NixOS VM test stops naming `curl`, whose license the OSI has not approved.
- The risk register drops the row on missing notices.

This change needs `license-compliance`, which creates `server/deny.toml` and `client/src/shadcn.css` and puts `cacert` into the image. Out of scope are a machine-readable software bill of materials, automated updates and vulnerability scanning. The change `supply-chain-hardening` covers the last two.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `service-operation`: Every artifact makes the licenses and copyright notices of its third-party code available, and a running instance serves them over HTTP.

## Impact

- `flake.nix`: notices generated from the vendored crates, one set of tools from nixpkgs, closure list and check for the image, license check for tools, notices in archive and image.
- `client/vite.config.ts` and `client/index.html`: notices of the client from Vite and the manifest, and a link to them.
- `client/notices.json` and `client/licenses/`: the manifest and the license texts it needs.
- `server/src/client.rs`: `text/plain; charset=utf-8` and an integration test for the served notices.
- `nix/test.nix`: assertions on the notices and no `curl` of its own.
- New build dependency `cargo-about` from nixpkgs.
- Documentation: the README on checks and copied code, arc42 sections on deployment, testing and risks, and the operating guide.
