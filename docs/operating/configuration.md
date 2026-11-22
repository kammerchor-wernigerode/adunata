# Configuration

Adunata reads its configuration from environment variables and, if the operator names one, from a configuration file. It checks every setting before it opens a port or connects to the database. If any setting is missing or invalid, Adunata prints every problem at once and exits with status 78. Whether `serve` can use the mail server settings shows only after it has connected to the database and applied pending migrations. If it cannot, it exits with status 78 as well.

An empty variable counts as not set.

A release build never reads a file such as `.env`. Only a development build reads `.env.local` and `.env` in its working directory, below the configuration file.

## Settings

Each setting has a key in a table of the [configuration file](#configuration-file). Its variable is `ADNT_`, the table, `_` and the key, in upper case with each `-` replaced by `_`.

|Table|Key|Variable|Required|Default|Meaning|
|-|-|-|-|-|-|
|`http`|`public-url`|`ADNT_HTTP_PUBLIC_URL`|yes||The address under which people reach the instance, such as `https://adunata.example.org`. Every link in a mail starts with it. It has to use HTTPS unless its host is `localhost`.|
|`http`|`listen-address`|`ADNT_HTTP_LISTEN_ADDRESS`|no|`127.0.0.1:8080`|The address and port on which Adunata accepts HTTP requests. Put a reverse proxy with HTTPS in front of it.|
|`http`|`trusted-proxies`|`ADNT_HTTP_TRUSTED_PROXIES`|no|`127.0.0.1/32,::1/128`|Networks in CIDR notation of the reverse proxies that Adunata trusts, comma-separated in a variable or as an array of strings in the file. For a request from such a network, Adunata takes the network address of the client from `X-Forwarded-For`. It needs that address to limit sign-in requests per client.|
|`database`|`url`|`ADNT_DATABASE_URL`|yes||The PostgreSQL connection, such as `postgres://adunata@db.example.org/adunata?sslmode=verify-full&sslrootcert=/etc/adunata/database-ca.crt`. Put the password in `ADNT_DATABASE_PASSWORD_FILE` or `ADNT_DATABASE_PASSWORD` instead of the URL. A password from either wins over a password in the URL. The parameter `sslmode` sets TLS to the database and takes `disable`, `allow`, `prefer`, `require`, `verify-ca` or `verify-full`. The default `prefer` encrypts without a check if the server offers TLS and connects in plain text otherwise. `allow` behaves like `disable`. `require` encrypts without a check. `verify-ca` checks the certificate against the [trust store](#certificate-authorities) and the authority in the file that `sslrootcert` names. `verify-full` also checks the host name and suits a database on another host. `sslrootcert` takes effect only with `verify-ca` or `verify-full`. If a server offers TLS with a broken setup, `sslmode=disable` restores plain text. A Unix socket stays plain under `prefer`. PostgreSQL offers no TLS on a socket, so `require` or stricter stops the start there. Adunata accepts only the parameters `sslmode`, `sslrootcert`, `host`, `hostaddr`, `port`, `dbname`, `user`, `password`, `application_name`, `options` and `statement-cache-capacity` with their aliases. It rejects every other parameter, such as the client certificate in `sslcert` and `sslkey`. It also rejects every variable whose name starts with `PGSSL`. Percent-encode `&`, `=` and `%` in a value, because Adunata would otherwise name a part of the value as a parameter.|
|`database`|`password`|`ADNT_DATABASE_PASSWORD`|no||The database password as a plain value. See [Secrets](#secrets).|
|`database`|`password-file`|`ADNT_DATABASE_PASSWORD_FILE`|no||Path to a file that holds the database password.|
|`smtp`|`host`|`ADNT_SMTP_HOST`|yes||Host name of the mail server that sends mail for Adunata.|
|`smtp`|`port`|`ADNT_SMTP_PORT`|no|`587`|Port of the mail server.|
|`smtp`|`encryption`|`ADNT_SMTP_ENCRYPTION`|no|`starttls`|How Adunata protects the connection to the mail server. `starttls` upgrades the connection and sends nothing if the server cannot upgrade. `tls` encrypts from the start, usually on port `465`. `none` sends without encryption and is meant for a mail server on the same host or for development.|
|`smtp`|`username`|`ADNT_SMTP_USERNAME`|no||User name at the mail server. Adunata signs in only if a password is set as well, as a plain value or as a file.|
|`smtp`|`password`|`ADNT_SMTP_PASSWORD`|no||The password at the mail server as a plain value. See [Secrets](#secrets).|
|`smtp`|`password-file`|`ADNT_SMTP_PASSWORD_FILE`|no||Path to a file that holds the password at the mail server.|
|`mail`|`sender`|`ADNT_MAIL_SENDER`|yes||The sender of every mail, such as `Adunata <adunata@example.org>`.|
|`mail`|`max-attempts`|`ADNT_MAIL_MAX_ATTEMPTS`|no|`8`|How often Adunata tries to deliver a mail before it marks the mail as failed. A whole number from 1 to 100.|
|`mail`|`confine`|`ADNT_MAIL_CONFINE`|no||Mail confinement. With `true`, Adunata delivers every mail to `confine-to` instead of its recipient, see [mail confinement](installation.md#mail-confinement). Takes `true` or `false`, as a boolean in the file. Only `serve --dev` accepts it. `serve` without `--dev` exits with status 78 when it is set, even to `false`.|
|`mail`|`confine-to`|`ADNT_MAIL_CONFINE_TO`|with `confine = true`||The mail address that receives every mail while confinement is on, such as `test@example.org`. Only `serve --dev` accepts it, like `confine`.|
|`localization`|`default-language`|`ADNT_LOCALIZATION_DEFAULT_LANGUAGE`|no|`en`|Language of the first organizer and of members that `grant-organizer` creates. Organizers choose the language of every other member. One of `de` or `en`.|
|`localization`|`time-zone`|`ADNT_LOCALIZATION_TIME_ZONE`|yes||The time zone of the instance as an IANA name, such as `Europe/Berlin`. Adunata shows times of arrival and departure in it and decides with it when a gathering begins and ends.|
|`retention`|`comment-days`|`ADNT_RETENTION_COMMENT_DAYS`|no|`30`|Days after the last day of a gathering until Adunata deletes the comments of its responses. A whole number or `never`.|
|`retention`|`response-days`|`ADNT_RETENTION_RESPONSE_DAYS`|no|`365`|Days after the last day of a gathering until Adunata deletes its invitations together with their responses. The gathering itself stays, but organizers then see it without invitations. A whole number or `never`.|
|`legal`|`privacy-url`|`ADNT_LEGAL_PRIVACY_URL`|no||Address of the privacy notice. Every page and every invitation mail links it.|
|`legal`|`imprint-url`|`ADNT_LEGAL_IMPRINT_URL`|no||Address of the imprint. Every page and every invitation mail links it.|
|`bootstrap`|`organizer-mail`|`ADNT_BOOTSTRAP_ORGANIZER_MAIL`|no||Mail address of the first organizer. If the instance has no organizer at start-up, Adunata creates this member with the organizer role.|
|`bootstrap`|`organizer-first-name`|`ADNT_BOOTSTRAP_ORGANIZER_FIRST_NAME`|with the mail address||First name of the first organizer.|
|`bootstrap`|`organizer-last-name`|`ADNT_BOOTSTRAP_ORGANIZER_LAST_NAME`|with the mail address||Last name of the first organizer.|

## Certificate authorities

Adunata checks the certificate of the mail server against the trust store of the host, such as `/etc/ssl/certs` on most distributions or `security.pki` on NixOS. It checks the certificate of the database there too if `sslmode` in the database URL is `verify-ca` or `verify-full`. To trust a private certificate authority, add it to that store and restart Adunata. For the database alone, `sslrootcert` in the URL adds the file of one authority to the store. The container image has no host store and brings the CA bundle of nixpkgs instead.

Two variables of OpenSSL change where Adunata looks. They are the only settings without the prefix `ADNT_`, have no key in the configuration file and are not checked at start-up.

|Variable|Meaning|
|-|-|
|`SSL_CERT_FILE`|Path to a file with certificate authorities in PEM format. The container image sets it to its own bundle.|
|`SSL_CERT_DIR`|Directories with certificate authorities in PEM format, separated by `:`.|

If either is set, Adunata ignores the store of the host and trusts only the authorities in the file and the directories that are set. The container image keeps its own `SSL_CERT_FILE` unless you set it, so `SSL_CERT_DIR` alone adds to the bundle of the image. To trust only your own authorities in the container, set `SSL_CERT_FILE`. A bundle of your own needs the public authorities too if the mail server or the database uses one of them.

If deliveries fail and the log shows `mail delivery failed` with `failure` set to `encryption`, the mail server offered no encryption or Adunata did not trust its certificate. Check that the host has a trust store, that it contains the authority of the mail server and that `SSL_CERT_FILE` and `SSL_CERT_DIR` point to existing files. If Adunata exits at start-up and the log shows `database is not ready` with a certificate error, check the same for the authority of the database or the file in `sslrootcert`.

## Configuration file

Name a file in TOML with the option `--config <path>` or the variable `ADNT_CONFIG_FILE`. The option wins over the variable. Without either, or with an empty value, Adunata reads no file. It never looks for a file on its own.

The file holds the tables and keys of the [settings](#settings) and nothing else. A key accepts only the spelling in the table, so `public_url` in `[http]` is an unknown key. A key takes a string or a whole number. `trusted-proxies` also takes a non-empty array of strings. `confine` takes a string or a boolean. `password` and `username` take only a string, because TOML would read a bare `0x1F` or `1_000` as a number and Adunata would sign in with its decimal text. An empty string counts as not set.

Adunata takes each setting from the first of these sources that sets it:

1. the environment
2. the configuration file
3. the default in the table of settings

Adunata refuses to start when the file cannot be read or parsed. It then reports only that problem, with the line and the column of a parse error but never the content of the line. An unknown table or key and a value of the wrong type stop the start as well. Their message names the file and the key, such as `smtp.port`, but not the value. Adunata reports a value of the right type that it rejects, such as `port = 99999`, under the name of its variable, such as `ADNT_SMTP_PORT`.

A file with a secret or with the personal data of the first organizer belongs to the user that runs Adunata, with mode `0600`. Keep a file under version control free of both. Use `password-file` keys there and leave out the table `bootstrap`.
## Secrets

Adunata takes each secret either as a plain value, such as `ADNT_SMTP_PASSWORD`, or from a file whose path the matching `_FILE` setting names. Both forms work in the environment and in the configuration file. A secret comes from the first source that sets either form, so a plain value in the environment wins over a `password-file` in the configuration file. One source must not set both forms of the same secret. Adunata then refuses to start and names both settings.

Prefer the `_FILE` settings. A plain value in the environment is visible to other processes of the same user and to `docker inspect`. In a systemd unit, every local user can read it through `systemctl show`. A secret file keeps the value out of all of them, and Adunata never needs a secret in its command line.

Adunata removes trailing line breaks from the file content. A file that is empty after that counts as not set, so Adunata then signs in neither to the database nor to the mail server with a password. It does not look for the secret in a later source then.

A relative path in a `_FILE` setting resolves against the working directory. If Adunata cannot read a secret file, it names the variable and the path in its message. It never prints a secret value.

## Example

In the environment:

```sh
ADNT_HTTP_PUBLIC_URL=https://adunata.example.org
ADNT_DATABASE_URL=postgres://adunata@localhost/adunata
ADNT_DATABASE_PASSWORD_FILE=/run/secrets/adunata-database
ADNT_SMTP_HOST=mail.example.org
ADNT_SMTP_USERNAME=adunata@example.org
ADNT_SMTP_PASSWORD_FILE=/run/secrets/adunata-smtp
ADNT_MAIL_SENDER="Adunata <adunata@example.org>"
ADNT_LOCALIZATION_TIME_ZONE=Europe/Berlin
ADNT_BOOTSTRAP_ORGANIZER_MAIL=organizer@example.org
ADNT_BOOTSTRAP_ORGANIZER_FIRST_NAME=Ada
ADNT_BOOTSTRAP_ORGANIZER_LAST_NAME=Lovelace
```

The same settings in a configuration file, started with `adunata serve --config /etc/adunata/adunata.toml`:

```toml
[http]
public-url = "https://adunata.example.org"

[database]
url = "postgres://adunata@localhost/adunata"
password-file = "/run/secrets/adunata-database"

[smtp]
host = "mail.example.org"
username = "adunata@example.org"
password-file = "/run/secrets/adunata-smtp"

[mail]
sender = "Adunata <adunata@example.org>"

[localization]
time-zone = "Europe/Berlin"

[bootstrap]
organizer-mail = "organizer@example.org"
organizer-first-name = "Ada"
organizer-last-name = "Lovelace"
```
