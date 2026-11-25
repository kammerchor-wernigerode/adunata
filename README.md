# Adunata

/a.duˈna.ta/, Italian for gathering.

Adunata is an open-source web application for choirs and instrumental ensembles to organize rehearsals and concerts. Organizers invite members by email. Members reply through a form whether and when they will attend.

## Contents

- [Installation](#installation)
- [Documentation](#documentation)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)

## Installation

Adunata runs on NixOS, in a container or as a systemd service on a Linux distribution with glibc 2.34 or newer. The [installation guide](docs/operating/installation.md) describes each way, and the [configuration reference](docs/operating/configuration.md) lists every setting.

## Documentation

|Path|Content|
|-|-|
|[`docs/operating/`](docs/operating/)|Installation and configuration for operators|
|[`docs/arc42/`](docs/arc42/README.md)|Architecture documentation|
|[`docs/decisions/`](docs/decisions/)|Architecture decision records|
|[`openspec/specs/`](openspec/specs/)|Current behavior as requirements|
|[`CONTRIBUTING.md`](CONTRIBUTING.md)|Settings, tests and checks for contributors|
|[`docs/maintenance/`](docs/maintenance/)|Releases, dependency updates, CI and repository settings for maintainers|

## Development

The server in [`server/`](server/) is written in Rust. The client in [`client/`](client/) is written in React with TypeScript. The server embeds the built client, so the client has to be built before the server compiles.

Development needs PostgreSQL to store data and Mailpit to catch outgoing mail. Both run from [`compose.yaml`](compose.yaml), with and without Nix. Mailpit shows every mail at <http://localhost:8025>.

The [contributing guide](CONTRIBUTING.md#settings) explains the settings files that Compose, the server and the client read.

### With Nix

Install [Nix](https://nixos.org/download/) and enable flakes. Install Docker or Podman with Compose as well. The development shell provides Rust, Node.js 26, pnpm, `cargo-deny` and `cargo-about`. It compiles Rust through sccache, which shares compiled dependencies between checkouts.

```sh
nix develop
```

With [direnv](https://direnv.net) the shell loads by itself whenever you enter the repository and keeps your own shell with its prompt and aliases. [nix-direnv](https://github.com/nix-community/nix-direnv) caches the shell, so entering takes no time after the first load. Allow the tracked `.envrc` once:

```sh
direnv allow
```

### Without Nix

Install these tools:

- [rustup](https://rustup.rs), which installs the version of Rust that [`server/rust-toolchain.toml`](server/rust-toolchain.toml) pins, with `clippy`, `rustfmt` and the sources of the standard library
- Node.js in the version that [`.node-version`](.node-version) names. fnm reads this file and picks the newest installed release of that major version
- [pnpm](https://pnpm.io/installation) in the version that `packageManager` in [`client/package.json`](client/package.json) names
- Docker or Podman with Compose

### Services

Start PostgreSQL and Mailpit in the root of the checkout:

```sh
docker compose up --detach --wait
```

Open the database with `psql` inside its container:

```sh
docker compose exec postgres psql -U adunata
```

Stop the services with `docker compose down`. Add `--volumes` to delete the database as well.

### Run Adunata

Build the client once, then start the server:

```sh
cd client
pnpm install --frozen-lockfile
pnpm run build
cd ../server
cargo run -- serve --local
```

The server listens on <http://localhost:8080> and applies database migrations on start. The option `--local` starts [local mode](docs/operating/installation.md#local-mode). On a fresh database it creates the organizer `organizer@example.org`, and before it accepts requests it logs a sign-in link for the oldest active organizer. Open that link to sign in. Every link you request on the sign-in page also appears in the log, so you need Mailpit only for other mail.

To work on the client, keep the server running and start the Vite development server in `client/`:

```sh
pnpm run dev
```

It serves the client on <http://localhost:5173> and reloads on every change. It forwards requests to `/api` to the server on port 8080 and sets their `Origin` to the public URL. `ADUNATA_VITE_PORT` in `client/.env.local` changes the port of Vite. The forwarding follows `ADNT_HTTP_LISTEN_ADDRESS` there.

## Contributing

The [contributing guide](CONTRIBUTING.md) describes settings, tests and the checks before a push. [`AGENTS.md`](AGENTS.md) holds the rules for workflow, pull requests and code style.

## License

Adunata is licensed under the [GNU Affero General Public License v3.0](LICENSE).
