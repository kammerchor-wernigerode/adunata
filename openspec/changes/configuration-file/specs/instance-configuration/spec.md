## ADDED Requirements

### Requirement: Configuration file
Adunata SHALL read an optional configuration file in TOML when the operator names it through the command-line option `--config` or the environment variable `ADNT_CONFIG_FILE`. The option SHALL win over the variable. An empty value SHALL count as not set. Without a name, Adunata SHALL read no configuration file.

#### Scenario: Start from a configuration file
- **WHEN** an operator starts Adunata with `--config /etc/adunata/adunata.toml`
- **AND** that file sets every required setting
- **THEN** Adunata starts with the settings from the file

#### Scenario: Option wins over variable
- **WHEN** `ADNT_CONFIG_FILE` names one file and `--config` names another
- **THEN** Adunata reads only the file that `--config` names

### Requirement: Environment over configuration file
A variable set in the environment SHALL win over the same setting in the configuration file.

#### Scenario: Environment overrides the file
- **WHEN** the configuration file sets `listen-address` in the table `http` to `127.0.0.1:8080`
- **AND** the environment sets `ADNT_HTTP_LISTEN_ADDRESS` to `127.0.0.1:9000`
- **THEN** Adunata listens on `127.0.0.1:9000`

### Requirement: Absent configuration file
Without a named file, Adunata SHALL take every setting from the environment alone.

#### Scenario: No file named
- **WHEN** an operator names no configuration file or sets `ADNT_CONFIG_FILE` to an empty value
- **THEN** Adunata reads its settings only from the environment

### Requirement: Unusable configuration file
When the named file cannot be read or parsed, Adunata SHALL refuse to start and report only that problem. The message SHALL name the file and, for a parse error whose position the parser reports, the line and the column. It SHALL NOT contain the content of the line.

#### Scenario: Named file missing
- **WHEN** `ADNT_CONFIG_FILE` names a file that does not exist
- **THEN** Adunata exits with a non-zero status and a message that names the file
- **AND** the message names no missing setting

#### Scenario: Parse error in a line with a secret
- **WHEN** the configuration file holds the line `password = pa$$word` without quotes in the table `database`
- **THEN** Adunata exits with a non-zero status and a message that names the file, the line and the column
- **AND** neither the message nor the log contains `pa$$word`

### Requirement: Tables of the configuration file
The configuration file SHALL group its settings in fixed tables. Each setting SHALL have exactly one key in kebab-case in exactly one table, as the operating documentation lists. The file SHALL accept no other spelling of a key. The tables SHALL be `http`, `database`, `smtp`, `mail`, `localization`, `retention`, `legal` and `bootstrap`. Adunata SHALL refuse to start when a table or key is unknown, and its message SHALL name the file and the key.

#### Scenario: Unknown key
- **WHEN** the configuration file holds the key `host` in the table `smpt`
- **THEN** Adunata exits with a non-zero status and a message that names the file and the key `smpt.host`

#### Scenario: Key at the wrong place
- **WHEN** the configuration file sets `smtp-host` at the top
- **THEN** Adunata exits with a non-zero status and a message that names the key `smtp-host`

#### Scenario: Other spelling of a key
- **WHEN** the configuration file sets `public_url` in the table `http`
- **THEN** Adunata exits with a non-zero status and a message that names the key `http.public_url`

### Requirement: Names of the environment variables
The name of the environment variable of a setting SHALL be `ADNT_`, the table, `_` and the key, in upper case with each `-` replaced by `_`.

#### Scenario: Name of the variable
- **WHEN** an operator looks up the variable for `organizer-mail` in the table `bootstrap`
- **THEN** its name is `ADNT_BOOTSTRAP_ORGANIZER_MAIL`

### Requirement: Values in the configuration file
A key SHALL take a string, an integer or, where a setting takes a list, a non-empty array of strings. A password or a username SHALL take only a string. An empty string SHALL count as not set. Adunata SHALL refuse to start when a value has a type the setting does not take, and its message SHALL name the file and the key but not the value.

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

## MODIFIED Requirements

### Requirement: Configuration from the environment
Adunata SHALL read its configuration from environment variables and, if the operator names one, from a configuration file. Every setting SHALL have a documented name. Every setting without a safe default SHALL be required.

#### Scenario: Start with all required settings
- **WHEN** an operator sets every required environment variable to a valid value
- **THEN** Adunata starts with that configuration

### Requirement: Secrets from files
Adunata SHALL accept each secret, such as the database password or the mail server password, either as a plain value or as the path to a file that holds the value. Both SHALL be possible in the environment and in the configuration file. A relative path SHALL resolve against the working directory. The environment SHALL win over the configuration file for secrets too. When one source sets both the plain value and the file of the same secret, Adunata SHALL refuse to start and name both settings. A secret file that is empty SHALL mean no secret and SHALL NOT fall back to the configuration file. A password from these settings SHALL win over a password in the database URL, whatever source each comes from. Adunata SHALL never require a secret to appear in a command line and SHALL never print a secret value.

#### Scenario: Read a secret from a file
- **WHEN** an operator sets the variable for the database password file to a readable file
- **THEN** Adunata uses the content of that file as the database password

#### Scenario: Secret as a plain value
- **WHEN** an operator sets `ADNT_SMTP_PASSWORD` and no file for the mail server password
- **THEN** Adunata signs in to the mail server with that value

#### Scenario: Both forms in one source
- **WHEN** the environment sets both `ADNT_DATABASE_PASSWORD` and `ADNT_DATABASE_PASSWORD_FILE`
- **THEN** Adunata exits with a non-zero status and a message that names both settings but no secret value

#### Scenario: Environment wins over the configuration file
- **WHEN** the environment sets `ADNT_DATABASE_PASSWORD`
- **AND** the configuration file sets `password-file` in the table `database` to a readable file
- **THEN** Adunata uses the value from the environment as the database password

#### Scenario: Empty secret file
- **WHEN** the environment sets `ADNT_DATABASE_PASSWORD_FILE` to an empty file
- **AND** the configuration file sets `password` in the table `database`
- **THEN** Adunata connects to the database without a password

#### Scenario: Secret file unreadable
- **WHEN** a configured secret file does not exist or cannot be read
- **THEN** Adunata refuses to start with a message that names the setting but not any secret value
