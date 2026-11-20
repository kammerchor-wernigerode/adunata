# Spec Delta

## ADDED Requirements

### Requirement: TLS to the database
Adunata SHALL take the TLS mode of the database connection from the parameter `sslmode` of the database URL and default to `prefer`. With `verify-ca` or `verify-full` it SHALL check the certificate of the database against the trust store of the host and the authority in the file that `sslrootcert` names, and with `verify-full` also the host name.

#### Scenario: Verified connection with an own authority
- **WHEN** the database URL sets `sslmode=verify-full` and `sslrootcert` to a file with the authority that signed the certificate of the database
- **AND** the certificate names the host of the URL
- **THEN** Adunata connects over TLS and starts

#### Scenario: Verified connection with the trust store of the host
- **WHEN** the database URL sets `sslmode=verify-full` and no `sslrootcert`
- **AND** the trust store of the host holds the authority of the certificate of the database
- **THEN** Adunata connects over TLS and starts

#### Scenario: Authority checked without the host name
- **WHEN** the database URL sets `sslmode=verify-ca` and `sslrootcert` to the authority of the certificate
- **AND** the certificate does not name the host of the URL
- **THEN** Adunata connects over TLS and starts

#### Scenario: Certificate from an unknown authority
- **WHEN** the database URL sets `sslmode=verify-full`
- **AND** neither the trust store of the host nor the file in `sslrootcert` holds the authority of the certificate of the database
- **THEN** Adunata refuses to connect and exits with status 1

#### Scenario: Host name does not match
- **WHEN** the database URL sets `sslmode=verify-full`
- **AND** the certificate of the database does not name the host of the URL
- **THEN** Adunata refuses to connect and exits with status 1

#### Scenario: Server without TLS
- **WHEN** the database URL sets `sslmode=require` or stricter
- **AND** the server does not offer TLS
- **THEN** Adunata refuses to connect and exits with status 1

#### Scenario: URL without TLS parameters
- **WHEN** the database URL sets no `sslmode`
- **THEN** Adunata connects over TLS without a check if the server offers TLS and without TLS otherwise

#### Scenario: Unix socket
- **WHEN** the database URL points to a Unix socket and sets no `sslmode`
- **THEN** Adunata connects without TLS

### Requirement: Parameters of the database URL
Adunata SHALL check the query of the database URL together with the other settings, before it touches the database. A parameter that Adunata does not support SHALL be a problem that names the parameter but not its value. A file named by `sslrootcert` that does not exist or cannot be read SHALL be a problem that names the parameter and the path. Each variable in the environment whose name starts with `PGSSL` SHALL be a problem that names the variable.

#### Scenario: Authority file missing
- **WHEN** the database URL sets `sslrootcert` to a path where no readable file exists
- **THEN** Adunata names `sslrootcert` and the path among all other problems and exits as with an invalid configuration before it touches the database

#### Scenario: Unsupported parameter
- **WHEN** the database URL sets `sslpassword=s3cret`
- **THEN** Adunata names `sslpassword` as unsupported, never prints `s3cret` and exits as with an invalid configuration

#### Scenario: Variable of libpq set
- **WHEN** the environment sets `PGSSLCERT`
- **THEN** Adunata names `PGSSLCERT` among all other problems and exits as with an invalid configuration before it touches the database

#### Scenario: Variable of libpq that sqlx ignores
- **WHEN** the environment sets `PGSSLCRL`
- **THEN** Adunata names `PGSSLCRL` among all other problems and exits as with an invalid configuration
