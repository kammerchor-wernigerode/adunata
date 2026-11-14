# Solution strategy

|Topic|Approach|Record|
|-|-|-|
|Stack|The server is written in plain Rust and the client in React with TypeScript. A spike confirmed the choice with a measured peak of 8 MB resident memory.|[ADR 0002](../decisions/0002-rust-and-react-as-stack.md)|
|Client model|A single-page client talks to a JSON API. One binary serves both on one port. The client code for the API is generated from an OpenAPI document.|[ADR 0003](../decisions/0003-single-page-client-with-api.md)|
|Database|PostgreSQL is the only database, also in tests.|[ADR 0004](../decisions/0004-postgresql-as-database.md)|
|Mail|Every mail goes through an outbox in the database. A background task sends it and retries until the mail server accepts it.|[ADR 0006](../decisions/0006-outbox-for-mail.md)|
|Decomposition|One process whose modules use each other only through their public interface. The compiler enforces the boundaries.|[ADR 0007](../decisions/0007-modular-monolith.md)|
|Client libraries|Accessible widgets, routing, server state and forms come from established libraries. The API client is generated.|[ADR 0008](../decisions/0008-client-libraries.md)|
