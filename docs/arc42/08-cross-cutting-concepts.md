# Cross-cutting concepts

## Configuration

An instance takes its configuration from environment variables only. There is no configuration file. Secrets come from files that a variable names, which fits secret handling in NixOS, Docker and systemd alike.

The module `configuration` reads all settings at start-up and checks them together. It collects every problem before Adunata touches the database or opens a port, so an operator fixes all mistakes in one round. Each message names the setting and never a secret value. Other modules receive typed settings, such as the public base URL, and never read the environment themselves.

The settings and their defaults are listed in the [configuration reference](../operating/configuration.md).

## Testing

Tests run at several levels. Each level checks what the level below cannot see. Every check runs through `nix flake check`, locally and in CI, so both use the same entry point.

|Level|What it checks|Example|
|-|-|-|
|Unit|Logic without input or output, such as formatting a message.|`plural_forms_in_german` in [`server/src/localization.rs`](../../server/src/localization.rs)|
|Server slice|An API handler called in process with real routing against a real PostgreSQL. Each test gets its own empty database with all migrations applied. Mail goes to a real Mailpit.|`rejects_a_taken_name_at_the_name_field` in [`server/src/spike.rs`](../../server/src/spike.rs)|
|Client slice|A component against the API mocks that Orval generates, driven like a user would. Accessibility checks with axe run on rendered components.|[`client/src/spike/SpikeForm.test.tsx`](../../client/src/spike/SpikeForm.test.tsx) and [`client/src/spike/SpikeForm.a11y.test.tsx`](../../client/src/spike/SpikeForm.a11y.test.tsx)|
|Contract|The committed OpenAPI document matches the server. The generated client code matches the document.|The check `contract` in [`flake.nix`](../../flake.nix)|
|End to end|A few complete flows in a browser against the real binary, PostgreSQL and Mailpit.|Planned|
|NixOS VM|The NixOS module starts, migrates the database and reports healthy.|Planned|

The check `memory` in [`flake.nix`](../../flake.nix) measures peak memory of the release build under load on Linux and fails above 150 MB.
