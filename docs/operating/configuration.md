# Configuration

Adunata reads its configuration from environment variables when it starts. It checks every setting before it opens a port or connects to the database. If any setting is missing or invalid, Adunata prints every problem at once and exits with status 1.

An empty variable counts as not set.

## Settings

|Variable|Required|Default|Meaning|
|-|-|-|-|
|`ADNT_HTTP_PUBLIC_URL`|yes||The address under which people reach the instance, such as `https://adunata.example.org`. Every link in a mail starts with it. It has to use HTTPS unless its host is `localhost`.|
|`ADNT_DATABASE_URL`|yes||The PostgreSQL connection, such as `postgres://adunata@db.example.org/adunata`. Put the password in `ADNT_DATABASE_PASSWORD_FILE` or `ADNT_DATABASE_PASSWORD` instead of the URL. A password from either wins over a password in the URL.|
|`ADNT_DATABASE_PASSWORD`|no||The database password as a plain value. See [Secrets](#secrets).|
|`ADNT_DATABASE_PASSWORD_FILE`|no||Path to a file that holds the database password.|
|`ADNT_SMTP_HOST`|yes||Host name of the mail server that sends mail for Adunata.|
|`ADNT_SMTP_PORT`|no|`587`|Port of the mail server.|
|`ADNT_SMTP_ENCRYPTION`|no|`starttls`|How Adunata protects the connection to the mail server. `starttls` upgrades the connection and sends nothing if the server cannot upgrade. `tls` encrypts from the start, usually on port `465`. `none` sends without encryption and is meant for a mail server on the same host or for development.|
|`ADNT_SMTP_USERNAME`|no||User name at the mail server. Adunata signs in only if a password is set as well, as a plain value or as a file.|
|`ADNT_SMTP_PASSWORD`|no||The password at the mail server as a plain value. See [Secrets](#secrets).|
|`ADNT_SMTP_PASSWORD_FILE`|no||Path to a file that holds the password at the mail server.|
|`ADNT_MAIL_SENDER`|yes||The sender of every mail, such as `Adunata <adunata@example.org>`.|
|`ADNT_MAIL_MAX_ATTEMPTS`|no|`8`|How often Adunata tries to deliver a mail before it marks the mail as failed. A whole number from 1 to 100.|
|`ADNT_LOCALIZATION_DEFAULT_LANGUAGE`|no|`en`|Language of the first organizer and of members that `grant-organizer` creates. Organizers choose the language of every other member. One of `de` or `en`.|
|`ADNT_LOCALIZATION_TIME_ZONE`|yes||The time zone of the instance as an IANA name, such as `Europe/Berlin`. Adunata shows times of arrival and departure in it and decides with it when a gathering begins and ends.|
|`ADNT_HTTP_LISTEN_ADDRESS`|no|`127.0.0.1:8080`|The address and port on which Adunata accepts HTTP requests. Put a reverse proxy with HTTPS in front of it.|
|`ADNT_HTTP_TRUSTED_PROXIES`|no|`127.0.0.1/32,::1/128`|Comma-separated networks in CIDR notation of the reverse proxies that Adunata trusts. For a request from such a network, Adunata takes the network address of the client from `X-Forwarded-For`. It needs that address to limit sign-in requests per client.|
|`ADNT_RETENTION_COMMENT_DAYS`|no|`30`|Days after the last day of a gathering until Adunata deletes the comments of its responses. A whole number or `never`.|
|`ADNT_RETENTION_RESPONSE_DAYS`|no|`365`|Days after the last day of a gathering until Adunata deletes its invitations together with their responses. The gathering itself stays, but organizers then see it without invitations. A whole number or `never`.|
|`ADNT_LEGAL_PRIVACY_URL`|no||Address of the privacy notice. Every page and every invitation mail links it.|
|`ADNT_LEGAL_IMPRINT_URL`|no||Address of the imprint. Every page and every invitation mail links it.|
|`ADNT_BOOTSTRAP_ORGANIZER_MAIL`|no||Mail address of the first organizer. If the instance has no organizer at start-up, Adunata creates this member with the organizer role.|
|`ADNT_BOOTSTRAP_ORGANIZER_FIRST_NAME`|with the mail address||First name of the first organizer.|
|`ADNT_BOOTSTRAP_ORGANIZER_LAST_NAME`|with the mail address||Last name of the first organizer.|

## Secrets

Adunata takes each secret either as a plain value, such as `ADNT_SMTP_PASSWORD`, or from a file whose path the matching `_FILE` setting names. One source must not set both forms of the same secret. Adunata then refuses to start and names both settings.

Prefer the `_FILE` settings. A plain value in the environment is visible to other processes of the same user and to `docker inspect`. In a systemd unit, every local user can read it through `systemctl show`. A secret file keeps the value out of all of them, and Adunata never needs a secret in its command line.

Adunata removes trailing line breaks from the file content. A file that is empty after that counts as not set, so Adunata then signs in neither to the database nor to the mail server with a password.

If Adunata cannot read a secret file, it names the variable and the path in its message. It never prints a secret value.

## Example

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
