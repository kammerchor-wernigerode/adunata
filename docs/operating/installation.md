# Installation

Adunata runs as one service next to PostgreSQL and a mail server. Put a reverse proxy with HTTPS in front of it, such as Caddy or nginx, and point `ADUNATA_PUBLIC_URL` at the address the proxy serves. The [configuration reference](configuration.md) lists every setting.

There are three supported ways to install Adunata. A NixOS test runs each of them in a virtual machine, see [`nix/test.nix`](../../nix/test.nix).

## NixOS

Add Adunata to the inputs of your system flake and import its module:

```nix
{
  inputs.adunata.url = "github:kammerchor-wernigerode/adunata";

  outputs = { nixpkgs, adunata, ... }: {
    nixosConfigurations.server = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        adunata.nixosModules.default
        {
          services.adunata = {
            enable = true;
            publicUrl = "https://adunata.example.org";
            mail.sender = "Adunata <adunata@example.org>";
            smtp = {
              host = "mail.example.org";
              username = "adunata@example.org";
              passwordFile = "/run/secrets/adunata-smtp";
            };
          };
        }
      ];
    };
  };
}
```

By default the module creates a local PostgreSQL database named `adunata` and connects through its Unix socket without a password. Set `services.adunata.database.createLocally = false` and `services.adunata.database.url` to use another database. The module passes password files to the service as systemd credentials, so the service user never reads them directly.

The service runs as a dynamic user with a hardened sandbox. `systemctl status adunata` shows its state and `journalctl -u adunata` its logs.

## Container

Each release publishes an image to `ghcr.io/kammerchor-wernigerode/adunata`. The image listens on port 8080 and runs as an unprivileged user. A Compose file with PostgreSQL could look like this:

```yaml
services:
  adunata:
    image: ghcr.io/kammerchor-wernigerode/adunata:latest
    ports:
      - "127.0.0.1:8080:8080"
    environment:
      ADUNATA_PUBLIC_URL: https://adunata.example.org
      ADUNATA_DATABASE_URL: postgres://adunata@postgres/adunata
      ADUNATA_DATABASE_PASSWORD_FILE: /run/secrets/database-password
      ADUNATA_SMTP_HOST: mail.example.org
      ADUNATA_MAIL_SENDER: Adunata <adunata@example.org>
    secrets:
      - database-password
    depends_on:
      - postgres

  postgres:
    image: postgres:17
    environment:
      POSTGRES_USER: adunata
      POSTGRES_DB: adunata
      POSTGRES_PASSWORD_FILE: /run/secrets/database-password
    secrets:
      - database-password
    volumes:
      - postgres:/var/lib/postgresql/data

secrets:
  database-password:
    file: ./database-password

volumes:
  postgres:
```

## Release binary

Each release publishes `adunata-x86_64-linux.tar.gz`. It holds a statically linked binary that runs on any Linux distribution with systemd, an example unit and an example environment file.

```sh
tar --extract --gzip --file adunata-x86_64-linux.tar.gz
install -D -m 0755 adunata /usr/local/bin/adunata
install -D -m 0644 adunata.service /etc/systemd/system/adunata.service
install -D -m 0600 adunata.env.example /etc/adunata/adunata.env
install -m 0600 /dev/null /etc/adunata/database-password
install -m 0600 /dev/null /etc/adunata/smtp-password
```

Edit `/etc/adunata/adunata.env` and write the two passwords into their files. Leave a file empty if no password is needed. Then start the service:

```sh
systemctl daemon-reload
systemctl enable --now adunata
```

## Updates

Install the newer release the same way and restart the service. Adunata applies pending database migrations on start. It refuses to start against a database that a newer release has already migrated, so going back to an older release needs a database backup from before the update.

## Health

`GET /api/health` answers with status 200 and `{"status":"ready"}` once Adunata can serve requests. It answers with status 503 and the unavailable dependency otherwise.
