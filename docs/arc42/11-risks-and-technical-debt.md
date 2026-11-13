# Risks and technical debt

The risks are ordered by priority.

|Risk|Mitigation|
|-|-|
|Coding agents write Rust that compiles but is hard to change. The maintainer is new to Rust and reviews structure rather than individual lines.|The rules for plain Rust in [`AGENTS.md`](../../AGENTS.md), strict lints that fail on any warning, checked module boundaries and a spike with fixed success criteria. If the spike fails, TypeScript on server and client replaces Rust.|
|UI components copied into the repository get no fixes from their upstream project by themselves.|The project changes copied components only where it must, so that a newer upstream version can replace them.|
