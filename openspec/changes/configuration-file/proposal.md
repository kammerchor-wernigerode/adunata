# Proposal

## Why

An operator configures Adunata only through environment variables and reads each secret from a file or puts the database password into the database URL. That suits NixOS, containers and systemd, but it forces every setting into a unit file, a Compose file or a wrapper script. An operator who keeps configuration in one file, or who has no secret store, has no direct way to do so. The choice between a file, the environment and a secret store belongs to the operator.

## What Changes

- Adunata reads an optional configuration file in TOML. The operator names it with `--config <path>` or `ADNT_CONFIG_FILE`. Without either, Adunata reads no file, as today.
- Every setting can come from the file. The file groups the settings in fixed tables by concern: `http`, `database`, `smtp`, `mail`, `localization`, `retention`, `legal` and `bootstrap`. Each setting has exactly one key in kebab-case, such as `public-url` in `[http]`.
- **BREAKING** Every environment variable derives from table and key, after the relaxed binding of Spring Boot, with the shorter prefix `ADNT_`. `public-url` in `[http]` becomes `ADNT_HTTP_PUBLIC_URL`. Every setting of the server changes its name. Variables of development tooling and tests are no settings and stay as they are. The options of the NixOS module regroup by the same tables. Adunata has no stable release, so there is no migration and no handling of old names.
- A variable set in the environment wins over the same setting in the file.
- Every secret can come as a plain value, `ADNT_DATABASE_PASSWORD` and `ADNT_SMTP_PASSWORD`, or from a file through the existing `_FILE` variables. Both may stand in the environment or in the configuration file. The environment wins. Both forms of one secret in one source stop the start.
- An unknown table or key, a value of the wrong type or a file that cannot be read or parsed stops the start with a message that names the file and the key but never a value.
- The operating documentation describes the file, its tables, the precedence and the plain secret values.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `instance-configuration`: New requirements for the configuration file, its tables and the names of the variables. "Configuration from the environment" names the file as a second source. "Secrets from files" allows a plain value next to the file and rejects both in one source.

## Impact

- `server/src/configuration.rs` takes its settings from an ordered list of sources instead of one lookup. One list of tables and keys yields every setting name. The parsers of the single settings stay as they are.
- `server/src/main.rs` gains the global option `--config`, which `clap` also reads from `ADNT_CONFIG_FILE`.
- New dependency `toml` under MIT or Apache-2.0 to parse the file. The feature `env` of `clap` reads the option from the environment.
- The renamed variables and module options change in `nix/module.nix`, `nix/test.nix`, `flake.nix`, `deploy/systemd/adunata.env.example`, `client/vite.config.ts`, `dev/worktree_plugin.py`, `server/tests/startup.rs` and `README.md`.
- `docs/operating/configuration.md`, `docs/operating/installation.md`, `docs/arc42/` and a new decision record change.
- An installation that still sets an old name loses that setting without a message. Without a stable release nobody runs such an installation.
