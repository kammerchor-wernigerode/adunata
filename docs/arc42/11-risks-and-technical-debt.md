# Risks and technical debt

The risks are ordered by priority.

|Risk|Mitigation|
|-|-|
|Coding agents write Rust that compiles but is hard to change. The maintainer is new to Rust and reviews structure rather than individual lines.|The rules for plain Rust in [`AGENTS.md`](../../AGENTS.md), strict lints that fail on any warning, checked module boundaries and a spike with fixed success criteria. The spike met its criteria within these rules.|
|Fluent, the localization format on server and client, is maintained slowly. Its React binding saw no release since 2023.|Adunata uses only the core libraries of Fluent and its own small React binding. The catalogs stay plain Fluent files, so another library that reads Fluent could replace them.|
|Fluent in Rust has no date formatting. Date ranges in mail are spelled out by the catalog and cover only ranges within one month.|Ranges across months or years get their own messages. If more languages or formats are needed, ICU4X can take over date formatting on the server.|
|UI components copied into the repository get no fixes from their upstream project by themselves.|The project changes copied components only where it must, so that a newer upstream version can replace them.|
