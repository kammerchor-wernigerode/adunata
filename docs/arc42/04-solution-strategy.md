# Solution strategy

|Topic|Approach|Record|
|-|-|-|
|Stack|The server is written in plain Rust and the client in React with TypeScript. The server compiles to one native binary with a small memory footprint, which fits a small shared server.|[ADR 0002](../decisions/0002-rust-and-react-as-stack.md)|
|Client model|A single-page client talks to a JSON API. One binary serves both on one port. The client code for the API is generated from an OpenAPI document.|[ADR 0003](../decisions/0003-single-page-client-with-api.md)|
|Database|PostgreSQL is the only database, also in tests.|[ADR 0004](../decisions/0004-postgresql-as-database.md)|
|Mail|Every mail goes through an outbox in the database. A background task sends it and retries with growing delays up to a configured number of attempts.|[ADR 0006](../decisions/0006-outbox-for-mail.md)|
|Decomposition|One process whose modules use each other only through their public interface. The compiler enforces the boundaries.|[ADR 0007](../decisions/0007-modular-monolith.md)|
|Client libraries|Accessible widgets, routing, server state and forms come from established libraries. The API client is generated.|[ADR 0008](../decisions/0008-client-libraries.md)|
|Data protection and security|Organizers sign in with a single-use link by mail and keep a session cookie whose hash the database stores. Every request that changes data must come from the origin of the instance.|[ADR 0010](../decisions/0010-sign-in-by-mail-link.md)|
|Data protection and security|Members answer through a personal link whose secret sits in the fragment of the URL, so no request line, log or referrer carries it. The database stores only its hash.|[ADR 0011](../decisions/0011-personal-links-in-the-url-fragment.md)|
|Build and deployment|One Nix flake builds the NixOS module, the OCI image and a release binary for glibc and runs every check except the end-to-end tests.|[ADR 0005](../decisions/0005-nix-flake-as-build-definition.md), [ADR 0015](../decisions/0015-release-binary-linked-against-glibc.md)|
