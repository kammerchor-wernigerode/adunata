# Spec Delta

## ADDED Requirements

### Requirement: Third-party notices served by the instance
A running instance SHALL serve its third-party notices without authentication. `/licenses.txt` SHALL be UTF-8 plain text with the license text and copyright notices of every third-party crate compiled into the server, of every third-party package in the browser client and of third-party code copied into the repository. `/licenses-rust.html` SHALL hold the copyright notices of the Rust standard library, and `/licenses.txt` SHALL name it.

#### Scenario: Read the notices of an instance
- **WHEN** anyone requests `/licenses.txt` from a running instance
- **THEN** Adunata responds with `text/plain; charset=utf-8` and a file that names each third-party crate and client package with its version, license text and copyright notices

#### Scenario: Notices of the Rust standard library
- **WHEN** anyone requests `/licenses-rust.html` from a running instance
- **THEN** Adunata responds with the copyright notices of the Rust standard library that the binary was built with, including its own third-party dependencies

#### Scenario: Notices match the build
- **WHEN** a dependency is added to the server or to the client
- **THEN** the next build of the binary serves notices that contain it

#### Scenario: Code that reaches the browser outside the JavaScript bundle
- **WHEN** a package reaches the browser only through the stylesheet, such as Tailwind CSS
- **THEN** the notices contain its license text and copyright notice

#### Scenario: Copied code
- **WHEN** a running instance serves its notices
- **THEN** they contain the source, version, license text and copyright notice of each piece of third-party code copied into the repository, such as the UI components and the stylesheet from shadcn

#### Scenario: The client points to the notices
- **WHEN** a browser loads the client
- **THEN** its HTML document links `/licenses.txt` as its license

### Requirement: Third-party notices in the release archive
The release archive SHALL contain the same notice files next to `LICENSE`.

#### Scenario: Unpack the release archive
- **WHEN** an operator unpacks the release archive
- **THEN** it contains `licenses.txt` and `licenses-rust.html` with the same content the binary serves

### Requirement: Third-party notices in the OCI image
The OCI image SHALL contain the notices of the binary and, for each package from nixpkgs in its runtime closure, the license text and a reference to the exact source the package was built from.

#### Scenario: Inspect the image
- **WHEN** an operator copies the license directory out of the OCI image as the operating guide describes
- **THEN** they find the notices of the binary and an entry for each package from nixpkgs in the image, such as glibc and the CA bundle, with its license text and source reference
