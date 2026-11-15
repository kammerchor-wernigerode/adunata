# Configuration

Adunata reads its configuration from environment variables when it starts. It checks every setting before it opens a port or connects to the database. If any setting is missing or invalid, Adunata prints every problem at once and exits with status 1.

An empty variable counts as not set.

## Settings

|Variable|Required|Default|Meaning|
|-|-|-|-|
|`ADUNATA_PUBLIC_URL`|yes||The address under which people reach the instance, such as `https://adunata.example.org`. Every link in a mail starts with it. It has to use HTTPS unless its host is `localhost`.|
|`ADUNATA_DATABASE_URL`|yes||The PostgreSQL connection, such as `postgres://adunata@db.example.org/adunata`. Put the password in `ADUNATA_DATABASE_PASSWORD_FILE` instead of the URL.|
|`ADUNATA_DATABASE_PASSWORD_FILE`|no||Path to a file that holds the database password.|
|`ADUNATA_SMTP_HOST`|yes||Host name of the mail server that sends mail for Adunata.|
|`ADUNATA_SMTP_PORT`|no|`587`|Port of the mail server.|
|`ADUNATA_SMTP_ENCRYPTION`|no|`starttls`|How Adunata protects the connection to the mail server. `starttls` upgrades the connection and sends nothing if the server cannot upgrade. `tls` encrypts from the start, usually on port `465`. `none` sends without encryption and is meant for a mail server on the same host or for development.|
|`ADUNATA_SMTP_USERNAME`|no||User name at the mail server. Adunata signs in only if the password file is set as well.|
|`ADUNATA_SMTP_PASSWORD_FILE`|no||Path to a file that holds the password at the mail server.|
|`ADUNATA_MAIL_SENDER`|yes||The sender of every mail, such as `Adunata <adunata@example.org>`.|
|`ADUNATA_MAIL_MAX_ATTEMPTS`|no|`8`|How often Adunata tries to deliver a mail before it marks the mail as failed. A whole number from 1 to 100.|
|`ADUNATA_DEFAULT_LANGUAGE`|no|`en`|Language of mail to recipients without a stored language. One of `de` or `en`.|
|`ADUNATA_TIME_ZONE`|yes||The time zone of the instance as an IANA name, such as `Europe/Berlin`. Adunata shows times of arrival and departure in it and decides with it when a gathering begins and ends.|
|`ADUNATA_LISTEN_ADDRESS`|no|`127.0.0.1:8080`|The address and port on which Adunata accepts HTTP requests. Put a reverse proxy with HTTPS in front of it.|
|`ADUNATA_TRUSTED_PROXIES`|no|`127.0.0.1/32,::1/128`|Comma-separated networks in CIDR notation of the reverse proxies that Adunata trusts. For a request from such a network, Adunata takes the network address of the client from `X-Forwarded-For`. It needs that address to limit sign-in requests per client.|
|`ADUNATA_RETENTION_COMMENT_DAYS`|no|`30`|Days after the last day of a gathering until Adunata deletes the comments of its responses. A whole number or `never`.|
|`ADUNATA_RETENTION_RESPONSE_DAYS`|no|`365`|Days after the last day of a gathering until Adunata deletes its invitations together with their responses. The gathering itself stays, but organizers then see it without invitations. A whole number or `never`.|
|`ADUNATA_PRIVACY_URL`|no||Address of the privacy notice. Every page and every invitation mail links it.|
|`ADUNATA_IMPRINT_URL`|no||Address of the imprint. Every page and every invitation mail links it.|
|`ADUNATA_FIRST_ORGANIZER_MAIL`|no||Mail address of the first organizer. If the instance has no organizer at start-up, Adunata creates this member with the organizer role.|
|`ADUNATA_FIRST_ORGANIZER_FIRST_NAME`|with the mail address||First name of the first organizer.|
|`ADUNATA_FIRST_ORGANIZER_LAST_NAME`|with the mail address||Last name of the first organizer.|

## Secrets

Adunata reads each secret from a file whose path a variable names. Secrets therefore never appear in a command line or in the list of processes. Adunata removes trailing line breaks from the file content. A file that is empty after that counts as not set, so Adunata then signs in neither to the database nor to the mail server with a password.

If Adunata cannot read a secret file, it names the variable and the path in its message. It never prints a secret value.

## Example

```sh
ADUNATA_PUBLIC_URL=https://adunata.example.org
ADUNATA_DATABASE_URL=postgres://adunata@localhost/adunata
ADUNATA_DATABASE_PASSWORD_FILE=/run/secrets/adunata-database
ADUNATA_SMTP_HOST=mail.example.org
ADUNATA_SMTP_USERNAME=adunata@example.org
ADUNATA_SMTP_PASSWORD_FILE=/run/secrets/adunata-smtp
ADUNATA_MAIL_SENDER="Adunata <adunata@example.org>"
ADUNATA_TIME_ZONE=Europe/Berlin
ADUNATA_FIRST_ORGANIZER_MAIL=organizer@example.org
ADUNATA_FIRST_ORGANIZER_FIRST_NAME=Ada
ADUNATA_FIRST_ORGANIZER_LAST_NAME=Lovelace
```
