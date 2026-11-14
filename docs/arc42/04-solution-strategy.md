# Solution strategy

|Topic|Approach|Record|
|-|-|-|
|Stack|The server is written in plain Rust and the client in React with TypeScript. A spike confirmed the choice with a measured peak of 8 MB resident memory.|[ADR 0002](../decisions/0002-rust-and-react-as-stack.md)|
|Client model|A single-page client talks to a JSON API. One binary serves both on one port. The client code for the API is generated from an OpenAPI document.|[ADR 0003](../decisions/0003-single-page-client-with-api.md)|
