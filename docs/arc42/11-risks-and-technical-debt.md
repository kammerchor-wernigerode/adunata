# Risks and technical debt

The risks are ordered by priority.

|Risk|Mitigation|
|-|-|
|Binary, OCI image and release archive carry no notices of the licenses of their dependencies, which MIT, Apache-2.0 and BSD require.|The change for a software bill of materials generates a file with every notice and ships it with each artifact.|
|Coding agents write Rust that compiles but is hard to change. The maintainer is new to Rust and reviews structure rather than individual lines.|The rules for plain Rust in [`AGENTS.md`](../../AGENTS.md), strict lints that fail on any warning and checked module boundaries.|
|Fluent, the localization format on server and client, is maintained slowly. Its React binding saw no release since 2023.|Adunata uses only the core libraries of Fluent and its own small React binding. The catalogs stay plain Fluent files, so another library that reads Fluent could replace them.|
|Fluent in Rust has no date formatting. The catalogs spell out date ranges in mail with one message each for a single day, a range within a month, a range across months and a range across years.|Each language needs all four messages, and tests cover them in German and English. If more languages or formats are needed, ICU4X can take over date formatting on the server.|
|UI components copied into the repository get no fixes from their upstream project by themselves.|The project changes copied components only where it must, so that a newer upstream version can replace them.|
