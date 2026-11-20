# Installation

Adunata runs as one service next to PostgreSQL and a mail server. Put a reverse proxy with HTTPS in front of it, such as Caddy or nginx, and point `ADNT_HTTP_PUBLIC_URL` at the address the proxy serves. The [configuration reference](configuration.md) lists every setting. Adunata checks the certificate of the mail server against the trust store of the host, so the host needs one with the authority of the mail server, see [certificate authorities](configuration.md#certificate-authorities).

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
            http = {
              publicUrl = "https://adunata.example.org";
              trustedProxies = [ "127.0.0.1/32" "::1/128" ];
            };
            localization.timeZone = "Europe/Berlin";
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

systemd restarts the service five seconds after a failure, but not after status 65 or 78, see [Commands](#commands). The module sets this policy, so override it with `lib.mkForce`. For example, add this module to `modules` to restart after 65 too:

```nix
{ lib, ... }:
{
  systemd.services.adunata.serviceConfig.RestartPreventExitStatus = lib.mkForce "78";
}
```

## Container

Each release publishes an image to `ghcr.io/kammerchor-wernigerode/adunata`. The image listens on port 8080 and runs as an unprivileged user. It sets `ADNT_HTTP_LISTEN_ADDRESS` to `0.0.0.0:8080`, which wins over `listen-address` in a configuration file, so change the port with that variable. A Compose file with PostgreSQL could look like this:

```yaml
services:
  adunata:
    image: ghcr.io/kammerchor-wernigerode/adunata:latest
    restart: on-failure
    ports:
      - "127.0.0.1:8080:8080"
    networks:
      - adunata
    environment:
      ADNT_HTTP_PUBLIC_URL: https://adunata.example.org
      ADNT_DATABASE_URL: postgres://adunata@postgres/adunata
      ADNT_DATABASE_PASSWORD_FILE: /run/secrets/database-password
      ADNT_SMTP_HOST: mail.example.org
      ADNT_MAIL_SENDER: Adunata <adunata@example.org>
      ADNT_LOCALIZATION_TIME_ZONE: Europe/Berlin
      ADNT_HTTP_TRUSTED_PROXIES: 172.30.0.1/32
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
    networks:
      - adunata
    volumes:
      - postgres:/var/lib/postgresql/data

networks:
  adunata:
    ipam:
      config:
        - subnet: 172.30.0.0/24
          gateway: 172.30.0.1

secrets:
  database-password:
    file: ./database-password

volumes:
  postgres:
```

The reverse proxy on the host reaches Adunata through the published port. Docker forwards these connections from the gateway of the network `adunata`, so `ADNT_HTTP_TRUSTED_PROXIES` names that gateway. A proxy that runs as a container in the same network needs its own address there instead.

With `restart: on-failure` Docker restarts the container after every failure, also after status 65 or 78, because it cannot leave out single statuses. Read the log if the container keeps restarting.

The image carries the license notices in `/share/licenses`. Next to the notices of the binary it holds, for each system package in the image such as glibc and the CA bundle, the license texts and the source the package was built from. The image has no shell, so copy the directory out of it:

```sh
docker create --name adunata-licenses ghcr.io/kammerchor-wernigerode/adunata:latest
docker cp adunata-licenses:/share/licenses ./licenses
docker rm adunata-licenses
```

## Release binary

Each release publishes `adunata-x86_64-linux.tar.gz`. It holds a binary for Linux distributions with systemd, glibc 2.34 or newer and libgcc_s, an example unit, an example environment file and the license notices `licenses.txt` and `licenses-rust.html`. On a distribution based on musl, such as Alpine, use the container instead.

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

The example unit restarts the service five seconds after a failure, but not after status 65 or 78, see [Commands](#commands). To change that policy, run `systemctl edit adunata`, clear the list with an empty assignment and set another one, for example to restart after 65 too:

```ini
[Service]
RestartPreventExitStatus=
RestartPreventExitStatus=78
```

## First organizer

Set `ADNT_BOOTSTRAP_ORGANIZER_MAIL` with both names, as the [configuration reference](configuration.md) describes. On NixOS set `organizerMail`, `organizerFirstName` and `organizerLastName` in `services.adunata.bootstrap`. When Adunata starts and finds no active organizer, it creates this member with the organizer role. The organizer then opens the instance, enters the address and signs in with the link from the mail. Once an organizer exists, Adunata ignores the settings.

## Commands

The binary runs the server with `adunata serve`, which the image, the example unit and the NixOS module call. `adunata --help` lists every command and `adunata help <command>` explains one. `serve` and `grant-organizer` exit with a status from `sysexits.h` for the failures that a supervisor handles differently. The log names the cause.

|Status|Meaning|
|-|-|
|0|The command succeeded.|
|1|Any other failure, such as a wrong database password, a missing privilege in the database or a listen address in use.|
|2|The arguments are wrong.|
|65|The data in the database does not fit this release, such as a schema that a newer release migrated or a migration that fails on the data.|
|75|The connection to the database failed for a reason that may pass, such as a database that does not answer yet or that shuts down. A socket path that does not exist counts as well, because Adunata cannot tell it from a socket that does not exist yet.|
|78|The configuration is invalid.|

### Recover organizer access

An operator who lost access to every organizer mailbox makes another member an organizer:

```sh
adunata grant-organizer organizer@example.org Ada Lovelace
```

The command reads the same settings as the server, so pass it the same `--config` or `ADNT_CONFIG_FILE` if the service uses a [configuration file](configuration.md#configuration-file). It connects to the database and makes the member with that address an active organizer. It creates the member with both names if no member has the address. In a container run it with `docker compose exec adunata adunata grant-organizer …` or the equivalent of your platform. On NixOS and with the release binary run it as the service user with the settings of the service.

### Development mode

`adunata serve --dev` is meant for a debug run on your own machine, not for an instance that others use. It starts only when `ADNT_HTTP_PUBLIC_URL` is on `localhost` and `ADNT_HTTP_LISTEN_ADDRESS` is a loopback address such as the default `127.0.0.1:8080`, and exits with status 78 otherwise. If the instance has no active organizer, it creates the first organizer from the bootstrap settings or, without them, `organizer@example.org` named Ada Lovelace. Before it accepts requests it writes a sign-in link for the oldest active organizer to the log, with the message `sign-in link for development` and the link in the field `link`. While the mode is on, every sign-in link that Adunata sends by mail also goes to the log, and the limit of links per address does not apply. The limit of 20 requests per network address and hour still applies. Anyone who can read the log can therefore sign in as an organizer.

## Updates

Install the newer release the same way and restart the service. Adunata applies pending database migrations on start. It exits with status 65 against a database that a newer release has already migrated, so going back to an older release needs a database backup from before the update.

## Health

`GET /api/health` answers with status 200 and `{"status":"ready"}` once Adunata can serve requests. It answers with status 503 and the unavailable dependency otherwise.

## License notices

Every installation serves the license texts and copyright notices of the third-party code in Adunata without authentication. `/licenses.txt` covers the crates of the server and the packages and copied code of the client, and `/licenses-rust.html` covers the Rust standard library. The release archive and the image carry the same files. The image also carries the notices of its system packages, see [Container](#container).
