# ADR 0017: Third-party notices in every artifact

## Context

The binary, the OCI image and the release archive contain third-party code under MIT, Apache-2.0, BSD and similar licenses, which require that their texts and copyright notices travel with the code. The image also contains glibc, the GCC runtime, `libidn2`, `libunistring` and a CA bundle from nixpkgs under LGPL, GPL with the GCC runtime exception and MPL-2.0, which also require access to the source. Browsers receive the client, so they are recipients too. The client contains code copied from shadcn, which no lock file lists. The image has no shell.

## Decision

We will build the notices from the license files of every crate compiled into the server, the copyright file of the Rust standard library, every package of the client and every entry of a manifest for what no lock file shows. Every instance serves them at `/licenses.txt` and `/licenses-rust.html`, and the client links them. The release archive carries both files next to `LICENSE`. The OCI image carries them in `/share/licenses` together with the license texts and a reference to the exact source of each package from nixpkgs in the image, and a check fails when a package enters the image without such an entry. We will answer the source obligations of the image with that reference, because the sources are public and fixed by hash.

## Status

Accepted

## Consequences

- Every artifact carries the notices of its dependencies and copied code.
- Anyone can read the notices of a running instance without signing in.
- A new dependency of the server or the client appears in the notices with the next build, and one without license text fails the build.
- Copying third-party code into the repository needs an entry in `client/notices.json`. Review catches a copy without one.
- A new package in the image fails the build until someone adds its license texts and source.
- The release archive carries no texts for glibc and libgcc_s, because the distribution provides them.
- If a public source behind a reference disappears, the project has to provide it on request. The reference names the hash, so any copy serves.
