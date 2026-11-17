## MODIFIED Requirements

### Requirement: Configuration from the environment
Adunata SHALL read its configuration from environment variables and, if the operator names one, from a configuration file. Every setting SHALL have a documented name. Every setting without a safe default SHALL be required. A development build SHALL also read the files `.env.local` and `.env` in its working directory. Each of them SHALL be a source below the configuration file. `.env.local` SHALL win over `.env`. A release build SHALL never read either file. A missing file SHALL count as empty. An empty value in a file SHALL count as unset. A malformed line SHALL stop the start with a message that names the file and the line number.

#### Scenario: Start with all required settings
- **WHEN** an operator sets every required environment variable to a valid value
- **THEN** Adunata starts with that configuration

#### Scenario: Release build ignores settings files
- **WHEN** a release build starts in a directory that holds a `.env` file with a database URL
- **AND** neither the environment nor a configuration file sets a database URL
- **THEN** Adunata exits with a non-zero status and a message that names `ADNT_DATABASE_URL` as missing

#### Scenario: Development build starts without settings in the environment
- **WHEN** a development build starts in a directory whose `.env` sets every required setting
- **AND** neither the environment nor a configuration file sets any of them
- **THEN** Adunata starts with the settings from `.env`

#### Scenario: Development build in a directory without settings files
- **WHEN** a development build starts in a directory without `.env` and without `.env.local`
- **AND** neither the environment nor a configuration file sets a public base URL
- **THEN** Adunata exits with a non-zero status and a message that names `ADNT_HTTP_PUBLIC_URL` as missing

#### Scenario: Malformed line in a settings file
- **WHEN** `.env.local` of a development build holds the line `export ADNT_SMTP_PORT=1025`
- **THEN** Adunata exits with a non-zero status and a message that names `.env.local` and the line number

#### Scenario: Local file overrides the shared file
- **WHEN** `.env` sets the listen address to port 8080 and `.env.local` sets it to port 18010
- **THEN** a development build listens on port 18010

#### Scenario: Configuration file overrides both settings files
- **WHEN** `.env.local` of a development build sets the mail server host to `localhost`
- **AND** the configuration file named with `--config` sets it to `mail.example.org`
- **THEN** Adunata sends mail through `mail.example.org`

#### Scenario: Environment overrides both files
- **WHEN** `.env.local` sets the listen address to port 18010 and the environment sets it to port 9000
- **THEN** a development build listens on port 9000
