## ADDED Requirements

### Requirement: Mail confinement settings
The setting `confine` in the table `mail` SHALL turn mail confinement on with `true` and leave it off with `false`. The setting `confine-to` in the table `mail` SHALL hold the redirect address. In the environment, `confine` SHALL take the strings `true` and `false`. Adunata SHALL accept either setting only for `serve --dev`. `serve` without `--dev` SHALL refuse to start when either setting is present, even when `confine` is `false`, and its message SHALL name the setting but not its value. `serve --dev` SHALL refuse to start when `confine` is `true` and `confine-to` is missing or no valid mail address. The commands `healthcheck` and `grant-organizer` SHALL accept both settings. The option `--dev` alone SHALL change nothing, SHALL apply no guard to the public URL or the listen address and SHALL be independent of `--local`.

#### Scenario: Setting without the option
- **WHEN** an operator starts `serve` without `--dev` and with `ADNT_MAIL_CONFINE_TO=test@example.org`
- **THEN** Adunata exits with status 78 before it touches the database, with a message that names `ADNT_MAIL_CONFINE_TO` and not `test@example.org`

#### Scenario: Switch off without the option
- **WHEN** an operator starts `serve` without `--dev` and with `ADNT_MAIL_CONFINE=false`
- **THEN** Adunata exits with status 78 with a message that names `ADNT_MAIL_CONFINE`

#### Scenario: Redirect address missing
- **WHEN** an operator starts `serve --dev` with `ADNT_MAIL_CONFINE=true` and without `ADNT_MAIL_CONFINE_TO`
- **THEN** Adunata exits with status 78 with a message that names `ADNT_MAIL_CONFINE_TO`

#### Scenario: Redirect address invalid
- **WHEN** an operator starts `serve --dev` with `ADNT_MAIL_CONFINE=true` and `ADNT_MAIL_CONFINE_TO=not an address`
- **THEN** Adunata exits with status 78 with a message that names `ADNT_MAIL_CONFINE_TO`

#### Scenario: Malformed switch
- **WHEN** an operator starts `serve --dev` with `ADNT_MAIL_CONFINE=yes`
- **THEN** Adunata exits with status 78 with a message that names `ADNT_MAIL_CONFINE` and the values `true` and `false`

#### Scenario: Option on a host other than localhost
- **WHEN** an operator starts `serve --dev` with the public URL `https://test.example.org` and without the confinement settings
- **THEN** Adunata starts and delivers mail to its recipients as `serve` does

#### Scenario: Healthcheck of a confined test system
- **WHEN** the settings hold `ADNT_MAIL_CONFINE=true` and `ADNT_MAIL_CONFINE_TO=test@example.org` and an operator runs `healthcheck` against a ready instance
- **THEN** `healthcheck` exits with 0

## MODIFIED Requirements

### Requirement: Values in the configuration file
A key SHALL take a string, an integer, a boolean where a setting is a switch or, where a setting takes a list, a non-empty array of strings. A password or a username SHALL take only a string. An empty string SHALL count as not set. Adunata SHALL refuse to start when a value has a type the setting does not take, and its message SHALL name the file and the key but not the value.

#### Scenario: Unsupported type
- **WHEN** the configuration file sets `port` in the table `smtp` to `true`
- **THEN** Adunata exits with a non-zero status and a message that names the key `smtp.port`

#### Scenario: Password as a number
- **WHEN** the configuration file sets `password` in the table `database` to the bare number `0x1F`
- **THEN** Adunata exits with a non-zero status and a message that names the key `database.password`

#### Scenario: Empty list of proxies
- **WHEN** the configuration file sets `trusted-proxies` in the table `http` to an empty array
- **THEN** Adunata exits with a non-zero status and a message that names the key `http.trusted-proxies`

#### Scenario: Empty string
- **WHEN** the configuration file sets `privacy-url` in the table `legal` to an empty string
- **THEN** Adunata starts without a link to a privacy notice

#### Scenario: Switch as a boolean
- **WHEN** the configuration file sets `confine = true` and `confine-to = "test@example.org"` in the table `mail` and an operator starts `serve --dev`
- **THEN** Adunata starts with mail confinement on
