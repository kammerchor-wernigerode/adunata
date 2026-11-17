use std::collections::HashMap;
use std::fmt;
use std::net::{IpAddr, SocketAddr};
use std::path::Path;

use axum::http::Uri;
use lettre::message::Mailbox;
use sqlx::postgres::PgConnectOptions;
use time::{Date, Month, OffsetDateTime, PrimitiveDateTime, Time};

const DEFAULT_LISTEN_ADDRESS: &str = "127.0.0.1:8080";
const DEFAULT_SMTP_PORT: &str = "587";
const DEFAULT_SMTP_ENCRYPTION: &str = "starttls";
const DEFAULT_MAIL_MAX_ATTEMPTS: &str = "8";
const DEFAULT_LANGUAGE: &str = "en";
const DEFAULT_TRUSTED_PROXIES: &str = "127.0.0.1/32,::1/128";
const DEFAULT_RETENTION_COMMENT_DAYS: &str = "30";
const DEFAULT_RETENTION_RESPONSE_DAYS: &str = "365";
const FIRST_ORGANIZER_MAIL: &str = "ADNT_BOOTSTRAP_ORGANIZER_MAIL";

pub struct Configuration {
    pub listen_address: SocketAddr,
    pub public_url: PublicUrl,
    pub database: PgConnectOptions,
    pub smtp: Smtp,
    pub mail_sender: Mailbox,
    pub mail_max_attempts: i32,
    pub default_language: String,
    pub time_zone: InstanceTimeZone,
    pub trusted_proxies: Vec<Network>,
    pub retention: Retention,
    pub legal_links: LegalLinks,
    pub first_organizer: Option<FirstOrganizer>,
}

/// How long Adunata keeps personal data after the last day of a gathering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Retention {
    pub comments: RetentionPeriod,
    pub responses: RetentionPeriod,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetentionPeriod {
    Days(u32),
    Never,
}

impl RetentionPeriod {
    fn parse(value: &str) -> Option<Self> {
        if value == "never" {
            return Some(Self::Never);
        }
        value.parse().ok().map(Self::Days)
    }
}

/// Addresses of the privacy notice and the imprint, each optional.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LegalLinks {
    pub privacy: Option<String>,
    pub imprint: Option<String>,
}

/// The member whom Adunata makes organizer when an instance has none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FirstOrganizer {
    pub mail: String,
    pub first_name: String,
    pub last_name: String,
}

/// A network in CIDR notation, such as the one of a trusted reverse proxy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Network {
    address: IpAddr,
    prefix: u8,
}

impl Network {
    pub fn parse(value: &str) -> Option<Self> {
        let (address, prefix) = value.trim().split_once('/')?;
        let address: IpAddr = address.parse().ok()?;
        let prefix: u8 = prefix.parse().ok()?;
        let bits = if address.is_ipv4() { 32 } else { 128 };
        (prefix <= bits).then_some(Self { address, prefix })
    }

    pub fn contains(&self, candidate: IpAddr) -> bool {
        match (self.address, candidate.to_canonical()) {
            (IpAddr::V4(network), IpAddr::V4(candidate)) => {
                let mask = u32::MAX
                    .checked_shl(32 - u32::from(self.prefix))
                    .unwrap_or(0);
                u32::from(network) & mask == u32::from(candidate) & mask
            }
            (IpAddr::V6(network), IpAddr::V6(candidate)) => {
                let mask = u128::MAX
                    .checked_shl(128 - u32::from(self.prefix))
                    .unwrap_or(0);
                u128::from(network) & mask == u128::from(candidate) & mask
            }
            _ => false,
        }
    }
}

pub struct Smtp {
    pub host: String,
    pub port: u16,
    pub encryption: Encryption,
    pub username: Option<String>,
    pub password: Option<String>,
}

/// How Adunata protects the connection to the mail server.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Encryption {
    /// Starts unencrypted and upgrades with STARTTLS. The mail is not sent if the server cannot upgrade.
    StartTls,
    /// Encrypts from the first byte, usually on port 465.
    Tls,
    /// Sends without encryption. Only an explicit setting allows this.
    None,
}

impl Encryption {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "starttls" => Some(Self::StartTls),
            "tls" => Some(Self::Tls),
            "none" => Some(Self::None),
            _ => None,
        }
    }
}

/// The address under which people reach the instance. Every link in a mail starts with it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicUrl {
    base: String,
}

impl PublicUrl {
    #[cfg(test)]
    pub fn parse(value: &str) -> Option<Self> {
        parse_public_url(value)
    }

    /// Scheme, host and port, as a browser sends them in the `Origin` header.
    pub fn origin(&self) -> String {
        let uri: Uri = self
            .base
            .parse()
            .expect("public URL was checked at start-up");
        let scheme = uri
            .scheme_str()
            .expect("public URL has a scheme")
            .to_ascii_lowercase();
        let host = uri
            .host()
            .expect("public URL has a host")
            .to_ascii_lowercase();
        let default_port = if scheme == "https" { 443 } else { 80 };
        match uri.port_u16().filter(|port| *port != default_port) {
            Some(port) => format!("{scheme}://{host}:{port}"),
            None => format!("{scheme}://{host}"),
        }
    }

    /// The port of an instance that runs on `localhost`, where browsers share cookies across ports. `None` for every other host.
    pub fn localhost_port(&self) -> Option<u16> {
        let uri: Uri = self
            .base
            .parse()
            .expect("public URL was checked at start-up");
        (uri.host() == Some("localhost")).then(|| uri.port_u16().unwrap_or(80))
    }

    pub fn link(&self, path: &str) -> String {
        format!("{}/{}", self.base, path.trim_start_matches('/'))
    }
}

/// The time zone in which the instance shows wall-clock times and compares them with the present.
#[derive(Clone, Debug)]
pub struct InstanceTimeZone {
    zone: jiff::tz::TimeZone,
}

impl InstanceTimeZone {
    pub fn parse(name: &str) -> Option<Self> {
        jiff::tz::TimeZone::get(name).ok().map(|zone| Self { zone })
    }

    /// The current date in the time zone of the instance.
    pub fn today(&self) -> Date {
        self.wall_clock(OffsetDateTime::now_utc()).date()
    }

    /// Turns an instant into the date and wall-clock time of the instance.
    pub fn wall_clock(&self, instant: OffsetDateTime) -> PrimitiveDateTime {
        let timestamp = jiff::Timestamp::new(
            instant.unix_timestamp(),
            i32::try_from(instant.nanosecond()).expect("nanoseconds fit into i32"),
        )
        .expect("every instant of time lies within the range of jiff");
        let civil = self.zone.to_datetime(timestamp);
        let month = Month::try_from(u8::try_from(civil.month()).expect("month is positive"))
            .expect("jiff yields valid months");
        let date = Date::from_calendar_date(
            i32::from(civil.year()),
            month,
            u8::try_from(civil.day()).expect("day is positive"),
        )
        .expect("jiff yields valid dates");
        let time = Time::from_hms_nano(
            u8::try_from(civil.hour()).expect("hour is positive"),
            u8::try_from(civil.minute()).expect("minute is positive"),
            u8::try_from(civil.second()).expect("second is positive"),
            u32::try_from(civil.subsec_nanosecond()).expect("nanoseconds are positive"),
        )
        .expect("jiff yields valid times");
        PrimitiveDateTime::new(date, time)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Problem {
    Missing {
        setting: &'static str,
    },
    Malformed {
        setting: &'static str,
        expected: &'static str,
    },
    Unreadable {
        setting: &'static str,
        path: String,
    },
    RequiredWith {
        setting: &'static str,
        with: &'static str,
    },
    BothForms {
        plain: &'static str,
        file: &'static str,
    },
    UnusableFile {
        file: String,
        reason: String,
    },
    UnknownKey {
        file: String,
        key: String,
    },
    UnsupportedValue {
        file: String,
        key: String,
        expected: &'static str,
    },
    UnreadableSettingsFile {
        path: String,
    },
    MalformedSettingsLine {
        path: String,
        line: usize,
    },
}

impl fmt::Display for Problem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { setting } => write!(formatter, "{setting} is required but not set"),
            Self::Malformed { setting, expected } => {
                write!(formatter, "{setting} must be {expected}")
            }
            Self::RequiredWith { setting, with } => {
                write!(formatter, "{setting} is required when {with} is set")
            }
            Self::BothForms { plain, file } => {
                write!(
                    formatter,
                    "{plain} and {file} are both set in the same source, set only one of them"
                )
            }
            Self::Unreadable { setting, path } => {
                write!(
                    formatter,
                    "{setting} names the file {path}, which cannot be read"
                )
            }
            Self::UnusableFile { file, reason } => {
                write!(formatter, "the configuration file {file} {reason}")
            }
            Self::UnknownKey { file, key } => {
                write!(
                    formatter,
                    "the configuration file {file} sets {key}, which is no setting"
                )
            }
            Self::UnsupportedValue {
                file,
                key,
                expected,
            } => write!(
                formatter,
                "{key} in the configuration file {file} must be {expected}"
            ),
            Self::UnreadableSettingsFile { path } => {
                write!(formatter, "the settings file {path} cannot be read")
            }
            Self::MalformedSettingsLine { path, line } => {
                write!(
                    formatter,
                    "line {line} of {path} must have the form KEY=value without quotes or trailing comments"
                )
            }
        }
    }
}

/// Every setting as its table and key. The name of each variable derives from them.
const SETTINGS: [(&str, &str); 23] = [
    ("http", "public-url"),
    ("http", "listen-address"),
    ("http", "trusted-proxies"),
    ("database", "url"),
    ("database", "password"),
    ("database", "password-file"),
    ("smtp", "host"),
    ("smtp", "port"),
    ("smtp", "encryption"),
    ("smtp", "username"),
    ("smtp", "password"),
    ("smtp", "password-file"),
    ("mail", "sender"),
    ("mail", "max-attempts"),
    ("localization", "default-language"),
    ("localization", "time-zone"),
    ("retention", "comment-days"),
    ("retention", "response-days"),
    ("legal", "privacy-url"),
    ("legal", "imprint-url"),
    ("bootstrap", "organizer-mail"),
    ("bootstrap", "organizer-first-name"),
    ("bootstrap", "organizer-last-name"),
];

/// The only setting that takes an array, whose strings Adunata joins with commas.
const LIST_SETTING: (&str, &str) = ("http", "trusted-proxies");

/// Settings that the file accepts only as strings, so that a credential reaches the server exactly as written.
const CREDENTIAL_SETTINGS: [(&str, &str); 3] = [
    ("database", "password"),
    ("smtp", "username"),
    ("smtp", "password"),
];

/// The environment variable of a setting, such as `ADNT_HTTP_PUBLIC_URL` for `public-url` in the table `http`.
fn variable_name(table: &str, key: &str) -> String {
    format!("ADNT_{table}_{key}")
        .to_ascii_uppercase()
        .replace('-', "_")
}

/// Answers the name of a setting with its value, if the source sets it.
pub type Source<'a> = &'a dyn Fn(&str) -> Option<String>;

/// The settings files `.env.local` and `.env` for development. Each file is a source of its own below the configuration file.
#[derive(Default)]
pub struct SettingsFiles {
    /// The values of each file, the one that wins first.
    files: Vec<HashMap<String, String>>,
    problems: Vec<Problem>,
}

impl SettingsFiles {
    /// Development builds read the settings files in the working directory. Release builds read none.
    pub fn of_working_directory() -> Self {
        let directory = std::env::current_dir().ok();
        Self::read(directory.as_deref().filter(|_| cfg!(debug_assertions)))
    }

    /// Reads `.env.local` and `.env` of a directory. A missing file counts as empty, and no directory reads no file.
    pub fn read(directory: Option<&Path>) -> Self {
        let mut files = Self::default();
        let Some(directory) = directory else {
            return files;
        };
        for name in [".env.local", ".env"] {
            let values = files.read_file(&directory.join(name));
            files.files.push(values);
        }
        files
    }

    fn read_file(&mut self, path: &Path) -> HashMap<String, String> {
        let content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return HashMap::new(),
            Err(_) => {
                self.problems.push(Problem::UnreadableSettingsFile {
                    path: path.display().to_string(),
                });
                return HashMap::new();
            }
        };
        match parse_settings(&content) {
            Ok(values) => values.into_iter().collect(),
            Err(line) => {
                self.problems.push(Problem::MalformedSettingsLine {
                    path: path.display().to_string(),
                    line,
                });
                HashMap::new()
            }
        }
    }

    /// The value of a setting from the given environment, else from the settings files in their order.
    #[cfg(test)]
    fn layered(&self, name: &str, environment: Source) -> Option<String> {
        environment(name).or_else(|| {
            self.files
                .iter()
                .find_map(|values| values.get(name).cloned())
        })
    }
}

/// A setting for tests, which find their services through the settings of the server. The settings files are read once in every build profile and must be valid, so that a broken `.env.local` never sends tests to the services of another checkout.
#[cfg(test)]
pub fn test_setting(name: &str) -> Option<String> {
    static FILES: std::sync::LazyLock<SettingsFiles> = std::sync::LazyLock::new(|| {
        let files = SettingsFiles::read(std::env::current_dir().ok().as_deref());
        assert!(
            files.problems.is_empty(),
            "settings files are invalid: {:?}",
            files.problems
        );
        files
    });
    FILES.layered(name, &|name| {
        std::env::var(name).ok().filter(|value| !value.is_empty())
    })
}

/// Parses `KEY=value` lines, skipping blank lines and `#` comments and dropping empty values. Fails with the number of the first malformed line.
///
/// Quotes and trailing comments count as malformed, because other dotenv readers strip them while this parser would keep them.
fn parse_settings(content: &str) -> Result<Vec<(String, String)>, usize> {
    let mut values = Vec::new();
    for (index, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed
            .split_once('=')
            .filter(|(key, value)| is_variable_name(key) && is_plain_value(value))
        else {
            return Err(index + 1);
        };
        if !value.is_empty() {
            values.push((key.to_owned(), value.to_owned()));
        }
    }
    Ok(values)
}

fn is_plain_value(value: &str) -> bool {
    !value.starts_with(['"', '\''])
        && !value.ends_with(char::is_whitespace)
        && !value.contains(" #")
        && !value.contains("\t#")
}

fn is_variable_name(key: &str) -> bool {
    let mut characters = key.chars();
    characters
        .next()
        .is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
        && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

impl Configuration {
    /// Reads the environment of the process and, if one is named, the configuration file.
    pub fn from_environment_and_file(file: Option<&str>) -> Result<Self, Vec<Problem>> {
        Self::from_sources(
            &|name| std::env::var(name).ok().filter(|value| !value.is_empty()),
            file,
            SettingsFiles::of_working_directory(),
        )
    }

    /// Reads the given environment, the named configuration file and the settings files for development, in this order of precedence.
    fn from_sources(
        environment: Source,
        file: Option<&str>,
        development: SettingsFiles,
    ) -> Result<Self, Vec<Problem>> {
        let SettingsFiles {
            files,
            mut problems,
        } = development;
        let settings = match file.map(read_file) {
            Some(Ok((settings, mut file_problems))) => {
                problems.append(&mut file_problems);
                settings
            }
            Some(Err(problem)) => return Err(vec![problem]),
            None => HashMap::new(),
        };
        let from_file = |name: &str| settings.get(name).cloned();
        let from_files: Vec<_> = files
            .iter()
            .map(|values| move |name: &str| values.get(name).cloned())
            .collect();
        let mut sources: Vec<Source> = vec![environment, &from_file];
        sources.extend(from_files.iter().map(|source| source as Source));
        match Self::load(&sources) {
            Ok(configuration) if problems.is_empty() => Ok(configuration),
            Ok(_) => Err(problems),
            Err(mut others) => {
                problems.append(&mut others);
                Err(problems)
            }
        }
    }

    /// Takes each setting from the first source that sets it.
    pub fn load(sources: &[Source]) -> Result<Self, Vec<Problem>> {
        let first = |name: &str| sources.iter().find_map(|source| source(name));
        let variable: Source = &first;
        let mut problems = Vec::new();
        let listen_address = parse(
            &mut problems,
            "ADNT_HTTP_LISTEN_ADDRESS",
            "a socket address such as 127.0.0.1:8080",
            Some(
                with_default(variable, "ADNT_HTTP_LISTEN_ADDRESS", DEFAULT_LISTEN_ADDRESS).as_str(),
            ),
            |value| value.parse().ok(),
        );
        let public_url = parse(
            &mut problems,
            "ADNT_HTTP_PUBLIC_URL",
            "an absolute HTTPS URL such as https://adunata.example.org",
            variable("ADNT_HTTP_PUBLIC_URL").as_deref(),
            parse_public_url,
        );
        let database = load_database(&mut problems, sources, variable);
        let smtp = load_smtp(&mut problems, sources, variable);
        let mail_sender = parse(
            &mut problems,
            "ADNT_MAIL_SENDER",
            "a mailbox such as Adunata <adunata@example.org>",
            variable("ADNT_MAIL_SENDER").as_deref(),
            |value| value.parse::<Mailbox>().ok(),
        );
        let mail_max_attempts = load_mail_max_attempts(&mut problems, variable);
        let default_language = parse(
            &mut problems,
            "ADNT_LOCALIZATION_DEFAULT_LANGUAGE",
            "a supported language: de or en",
            Some(
                with_default(
                    variable,
                    "ADNT_LOCALIZATION_DEFAULT_LANGUAGE",
                    DEFAULT_LANGUAGE,
                )
                .as_str(),
            ),
            |value| crate::localization::is_supported(value).then(|| value.to_owned()),
        );

        let time_zone = load_time_zone(&mut problems, variable);
        let trusted_proxies = load_trusted_proxies(&mut problems, variable);
        let retention = load_retention(&mut problems, variable);
        let legal_links = LegalLinks {
            privacy: optional_link(&mut problems, "ADNT_LEGAL_PRIVACY_URL", variable),
            imprint: optional_link(&mut problems, "ADNT_LEGAL_IMPRINT_URL", variable),
        };
        let first_organizer = load_first_organizer(&mut problems, variable);

        match (
            listen_address,
            public_url,
            database,
            smtp,
            mail_sender,
            mail_max_attempts,
            default_language,
            time_zone,
            trusted_proxies,
            retention,
        ) {
            (
                Some(listen_address),
                Some(public_url),
                Some(database),
                Some(smtp),
                Some(mail_sender),
                Some(mail_max_attempts),
                Some(default_language),
                Some(time_zone),
                Some(trusted_proxies),
                Some(retention),
            ) if problems.is_empty() => Ok(Self {
                listen_address,
                public_url,
                database,
                smtp,
                mail_sender,
                mail_max_attempts,
                default_language,
                time_zone,
                trusted_proxies,
                retention,
                legal_links,
                first_organizer,
            }),
            _ => Err(problems),
        }
    }
}

fn load_database(
    problems: &mut Vec<Problem>,
    sources: &[Source],
    variable: Source,
) -> Option<PgConnectOptions> {
    let database = parse(
        problems,
        "ADNT_DATABASE_URL",
        "a PostgreSQL URL such as postgres://adunata@localhost/adunata",
        variable("ADNT_DATABASE_URL").as_deref(),
        |value| value.parse::<PgConnectOptions>().ok(),
    );
    let password = secret(
        problems,
        "ADNT_DATABASE_PASSWORD",
        "ADNT_DATABASE_PASSWORD_FILE",
        sources,
    );
    match password {
        Some(password) => database.map(|database| database.password(&password)),
        None => database,
    }
}

fn load_smtp(problems: &mut Vec<Problem>, sources: &[Source], variable: Source) -> Option<Smtp> {
    let host = parse(
        problems,
        "ADNT_SMTP_HOST",
        "a host name",
        variable("ADNT_SMTP_HOST").as_deref(),
        |value| Some(value.to_owned()),
    );
    let port = parse(
        problems,
        "ADNT_SMTP_PORT",
        "a port number",
        Some(with_default(variable, "ADNT_SMTP_PORT", DEFAULT_SMTP_PORT).as_str()),
        |value| value.parse().ok(),
    );
    let encryption = parse(
        problems,
        "ADNT_SMTP_ENCRYPTION",
        "one of starttls, tls or none",
        Some(with_default(variable, "ADNT_SMTP_ENCRYPTION", DEFAULT_SMTP_ENCRYPTION).as_str()),
        Encryption::parse,
    );
    let username = variable("ADNT_SMTP_USERNAME");
    let password = secret(
        problems,
        "ADNT_SMTP_PASSWORD",
        "ADNT_SMTP_PASSWORD_FILE",
        sources,
    );
    Some(Smtp {
        host: host?,
        port: port?,
        encryption: encryption?,
        username,
        password,
    })
}

fn load_mail_max_attempts(problems: &mut Vec<Problem>, variable: Source) -> Option<i32> {
    parse(
        problems,
        "ADNT_MAIL_MAX_ATTEMPTS",
        "a whole number from 1 to 100",
        Some(
            with_default(
                variable,
                "ADNT_MAIL_MAX_ATTEMPTS",
                DEFAULT_MAIL_MAX_ATTEMPTS,
            )
            .as_str(),
        ),
        |value| {
            value
                .parse::<i32>()
                .ok()
                .filter(|attempts| (1..=100).contains(attempts))
        },
    )
}

fn load_time_zone(problems: &mut Vec<Problem>, variable: Source) -> Option<InstanceTimeZone> {
    parse(
        problems,
        "ADNT_LOCALIZATION_TIME_ZONE",
        "an IANA time zone such as Europe/Berlin",
        variable("ADNT_LOCALIZATION_TIME_ZONE").as_deref(),
        InstanceTimeZone::parse,
    )
}

fn load_trusted_proxies(problems: &mut Vec<Problem>, variable: Source) -> Option<Vec<Network>> {
    parse(
        problems,
        "ADNT_HTTP_TRUSTED_PROXIES",
        "a comma-separated list of networks such as 172.18.0.0/16,::1/128",
        Some(
            with_default(
                variable,
                "ADNT_HTTP_TRUSTED_PROXIES",
                DEFAULT_TRUSTED_PROXIES,
            )
            .as_str(),
        ),
        |value| value.split(',').map(Network::parse).collect(),
    )
}

fn load_retention(problems: &mut Vec<Problem>, variable: Source) -> Option<Retention> {
    let mut period = |setting, default| {
        parse(
            problems,
            setting,
            "a whole number of days or never",
            Some(with_default(variable, setting, default).as_str()),
            RetentionPeriod::parse,
        )
    };
    let comments = period(
        "ADNT_RETENTION_COMMENT_DAYS",
        DEFAULT_RETENTION_COMMENT_DAYS,
    );
    let responses = period(
        "ADNT_RETENTION_RESPONSE_DAYS",
        DEFAULT_RETENTION_RESPONSE_DAYS,
    );
    Some(Retention {
        comments: comments?,
        responses: responses?,
    })
}

fn optional_link(
    problems: &mut Vec<Problem>,
    setting: &'static str,
    variable: Source,
) -> Option<String> {
    let value = variable(setting)?;
    parse(
        problems,
        setting,
        "an absolute HTTP or HTTPS URL such as https://example.org/privacy",
        Some(&value),
        |value| is_web_address(value).then(|| value.to_owned()),
    )
}

fn is_web_address(value: &str) -> bool {
    value
        .parse::<Uri>()
        .is_ok_and(|uri| matches!(uri.scheme_str(), Some("http" | "https")) && uri.host().is_some())
}

fn load_first_organizer(problems: &mut Vec<Problem>, variable: Source) -> Option<FirstOrganizer> {
    let mail = variable(FIRST_ORGANIZER_MAIL)?;
    let mail = parse(
        problems,
        FIRST_ORGANIZER_MAIL,
        "a mail address such as organizer@example.org",
        Some(&mail),
        |value| {
            value
                .parse::<lettre::Address>()
                .ok()
                .map(|_| value.to_owned())
        },
    );
    let mut name = |setting| {
        let name = variable(setting).filter(|name| !name.trim().is_empty());
        if name.is_none() {
            problems.push(Problem::RequiredWith {
                setting,
                with: FIRST_ORGANIZER_MAIL,
            });
        }
        name.map(|name| name.trim().to_owned())
    };
    let first_name = name("ADNT_BOOTSTRAP_ORGANIZER_FIRST_NAME");
    let last_name = name("ADNT_BOOTSTRAP_ORGANIZER_LAST_NAME");
    Some(FirstOrganizer {
        mail: mail?,
        first_name: first_name?,
        last_name: last_name?,
    })
}

fn with_default(variable: Source, setting: &str, default: &str) -> String {
    variable(setting).unwrap_or_else(|| default.to_owned())
}

fn parse<T>(
    problems: &mut Vec<Problem>,
    setting: &'static str,
    expected: &'static str,
    value: Option<&str>,
    parse: impl Fn(&str) -> Option<T>,
) -> Option<T> {
    let Some(value) = value else {
        problems.push(Problem::Missing { setting });
        return None;
    };
    let parsed = parse(value);
    if parsed.is_none() {
        problems.push(Problem::Malformed { setting, expected });
    }
    parsed
}

/// Takes a secret from the first source that sets its plain value or its file. An empty file means no secret.
fn secret(
    problems: &mut Vec<Problem>,
    plain: &'static str,
    file: &'static str,
    sources: &[Source],
) -> Option<String> {
    let (value, path) = sources
        .iter()
        .map(|source| (source(plain), source(file)))
        .find(|(value, path)| value.is_some() || path.is_some())?;
    let Some(path) = path else {
        return value;
    };
    if value.is_some() {
        problems.push(Problem::BothForms { plain, file });
        return None;
    }
    let Ok(content) = std::fs::read_to_string(&path) else {
        problems.push(Problem::Unreadable {
            setting: file,
            path,
        });
        return None;
    };
    let secret = content.trim_end_matches(['\r', '\n']);
    (!secret.is_empty()).then(|| secret.to_owned())
}

/// The settings of a configuration file by the names of their variables, together with the problems of its tables and keys.
type FileSettings = (HashMap<String, String>, Vec<Problem>);

fn read_file(file: &str) -> Result<FileSettings, Problem> {
    let text = std::fs::read_to_string(file).map_err(|error| Problem::UnusableFile {
        file: file.to_owned(),
        reason: format!("cannot be read: {error}"),
    })?;
    parse_file(file, &text)
}

/// Never uses the `Display` of a parser error, because it quotes the line and with it any secret on that line.
fn parse_file(file: &str, text: &str) -> Result<FileSettings, Problem> {
    let tables = text
        .parse::<toml::Table>()
        .map_err(|error| Problem::UnusableFile {
            file: file.to_owned(),
            reason: parse_failure(text, error.span(), error.message()),
        })?;
    let mut settings = HashMap::new();
    let mut problems = Vec::new();
    for (table, keys) in &tables {
        let Some(keys) = keys.as_table() else {
            problems.push(Problem::UnknownKey {
                file: file.to_owned(),
                key: table.clone(),
            });
            continue;
        };
        for (key, value) in keys {
            let name = format!("{table}.{key}");
            if !SETTINGS.contains(&(table.as_str(), key.as_str())) {
                problems.push(Problem::UnknownKey {
                    file: file.to_owned(),
                    key: name,
                });
                continue;
            }
            match setting_value(Shape::of((table, key)), value) {
                Ok(Some(value)) => {
                    settings.insert(variable_name(table, key), value);
                }
                Ok(None) => {}
                Err(expected) => problems.push(Problem::UnsupportedValue {
                    file: file.to_owned(),
                    key: name,
                    expected,
                }),
            }
        }
    }
    Ok((settings, problems))
}

/// The values a key of the file takes.
#[derive(Clone, Copy)]
enum Shape {
    /// A string only, because TOML would turn a bare `0x1F` or `1_000` into another text.
    Text,
    TextOrNumber,
    TextOrList,
}

impl Shape {
    fn of(setting: (&str, &str)) -> Self {
        if setting == LIST_SETTING {
            Self::TextOrList
        } else if CREDENTIAL_SETTINGS.contains(&setting) {
            Self::Text
        } else {
            Self::TextOrNumber
        }
    }

    fn expected(self) -> &'static str {
        match self {
            Self::Text => "a string",
            Self::TextOrNumber => "a string or a whole number",
            Self::TextOrList => "a string or a non-empty array of strings",
        }
    }
}

/// Turns a value of the file into the text a variable would hold. An empty string counts as not set.
fn setting_value(shape: Shape, value: &toml::Value) -> Result<Option<String>, &'static str> {
    match (shape, value) {
        (_, toml::Value::String(text)) => Ok((!text.is_empty()).then(|| text.clone())),
        (Shape::TextOrNumber, toml::Value::Integer(number)) => Ok(Some(number.to_string())),
        (Shape::TextOrList, toml::Value::Array(items)) => items
            .iter()
            .map(toml::Value::as_str)
            .collect::<Option<Vec<&str>>>()
            .filter(|items| !items.is_empty())
            .map(|items| Some(items.join(",")))
            .ok_or(shape.expected()),
        _ => Err(shape.expected()),
    }
}

/// Names the position of a parse error only when the parser knows it.
fn parse_failure(text: &str, span: Option<std::ops::Range<usize>>, message: &str) -> String {
    match span {
        Some(span) => {
            let (line, column) = position(text, span.start);
            format!("cannot be parsed at line {line}, column {column}: {message}")
        }
        None => format!("cannot be parsed: {message}"),
    }
}

/// Line and column, both counted from one, of a byte offset into a text.
fn position(text: &str, offset: usize) -> (usize, usize) {
    let before = text.get(..offset).unwrap_or(text);
    let line = before.matches('\n').count() + 1;
    let column = before
        .rsplit('\n')
        .next()
        .unwrap_or_default()
        .chars()
        .count()
        + 1;
    (line, column)
}

fn parse_public_url(value: &str) -> Option<PublicUrl> {
    let uri: Uri = value.parse().ok()?;
    let host = uri.host()?;
    let secure = match uri.scheme_str()? {
        "https" => true,
        "http" => host == "localhost",
        _ => false,
    };
    let plain = uri.query().is_none() && !value.contains('#');
    (secure && plain).then(|| PublicUrl {
        base: value.trim_end_matches('/').to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU32, Ordering};

    use super::*;

    static NEXT: AtomicU32 = AtomicU32::new(0);

    fn required() -> HashMap<&'static str, String> {
        HashMap::from([
            (
                "ADNT_HTTP_PUBLIC_URL",
                "https://adunata.example.org".to_owned(),
            ),
            (
                "ADNT_DATABASE_URL",
                "postgres://adunata@localhost/adunata".to_owned(),
            ),
            ("ADNT_SMTP_HOST", "mail.example.org".to_owned()),
            (
                "ADNT_MAIL_SENDER",
                "Adunata <adunata@example.org>".to_owned(),
            ),
            ("ADNT_LOCALIZATION_TIME_ZONE", "Europe/Berlin".to_owned()),
        ])
    }

    fn load(variables: &HashMap<&'static str, String>) -> Result<Configuration, Vec<Problem>> {
        Configuration::load(&[&|name| variables.get(name).cloned()])
    }

    fn load_two(
        first: &HashMap<&'static str, String>,
        second: &HashMap<&'static str, String>,
    ) -> Result<Configuration, Vec<Problem>> {
        Configuration::load(&[&|name| first.get(name).cloned(), &|name| {
            second.get(name).cloned()
        }])
    }

    fn database_debug(configuration: &Configuration) -> String {
        format!("{:?}", configuration.database)
    }

    /// Loads once more without the public URL, so that `load` fails, and asserts that no problem shows the secret.
    fn assert_no_problem_shows(
        secret: &str,
        first: &HashMap<&'static str, String>,
        second: &HashMap<&'static str, String>,
    ) {
        let mut first = first.clone();
        let mut second = second.clone();
        first.remove("ADNT_HTTP_PUBLIC_URL");
        second.remove("ADNT_HTTP_PUBLIC_URL");
        let problems = load_two(&first, &second)
            .err()
            .expect("configuration without public URL must be rejected");
        for problem in problems {
            assert!(!problem.to_string().contains(secret), "{problem}");
        }
    }

    fn secret_file(content: &str) -> String {
        let path = std::env::temp_dir().join(format!(
            "adunata-secret-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, content).unwrap();
        path.to_string_lossy().into_owned()
    }

    fn problems(variables: &HashMap<&'static str, String>) -> Vec<Problem> {
        load(variables)
            .err()
            .expect("configuration must be rejected")
    }

    fn derived_names() -> Vec<String> {
        SETTINGS
            .iter()
            .map(|(table, key)| variable_name(table, key))
            .collect()
    }

    #[test]
    fn variable_name_of_a_key_with_dashes() {
        assert_eq!(
            variable_name("bootstrap", "organizer-mail"),
            "ADNT_BOOTSTRAP_ORGANIZER_MAIL"
        );
    }

    #[test]
    fn derived_names_are_unique() {
        let names = derived_names();
        let unique: std::collections::BTreeSet<&String> = names.iter().collect();
        assert_eq!(unique.len(), names.len());
    }

    #[test]
    fn load_reads_exactly_the_derived_names() {
        let asked = std::cell::RefCell::new(std::collections::BTreeSet::new());
        let _ = Configuration::load(&[&|name| {
            asked.borrow_mut().insert(name.to_owned());
            Some("x".to_owned())
        }]);
        let derived: std::collections::BTreeSet<String> = derived_names().into_iter().collect();
        assert_eq!(asked.into_inner(), derived);
    }

    fn file_settings(text: &str) -> HashMap<String, String> {
        let (settings, problems) = parse_file("adunata.toml", text).expect("file must parse");
        assert_eq!(problems, vec![]);
        settings
    }

    fn file_problems(text: &str) -> Vec<String> {
        let (_, problems) = parse_file("adunata.toml", text).expect("file must parse");
        problems.iter().map(ToString::to_string).collect()
    }

    #[test]
    fn file_values_of_each_type() {
        let settings = file_settings(
            "[http]\npublic-url = \"https://adunata.example.org\"\ntrusted-proxies = [\"10.0.0.0/8\", \"::1/128\"]\n[smtp]\nport = 465\n",
        );
        assert_eq!(
            settings,
            HashMap::from([
                (
                    "ADNT_HTTP_PUBLIC_URL".to_owned(),
                    "https://adunata.example.org".to_owned()
                ),
                (
                    "ADNT_HTTP_TRUSTED_PROXIES".to_owned(),
                    "10.0.0.0/8,::1/128".to_owned()
                ),
                ("ADNT_SMTP_PORT".to_owned(), "465".to_owned()),
            ])
        );
    }

    #[test]
    fn empty_string_in_file_counts_as_unset() {
        assert_eq!(
            file_settings("[legal]\nprivacy-url = \"\"\n"),
            HashMap::new()
        );
    }

    #[test]
    fn unknown_table_and_key_in_file() {
        assert_eq!(
            file_problems(
                "[smpt]\nhost = \"mail.example.org\"\n[http]\npublic_url = \"https://adunata.example.org\"\n"
            ),
            vec![
                "the configuration file adunata.toml sets http.public_url, which is no setting",
                "the configuration file adunata.toml sets smpt.host, which is no setting",
            ]
        );
    }

    #[test]
    fn key_at_the_top_of_the_file() {
        assert_eq!(
            file_problems("smtp-host = \"mail.example.org\"\n"),
            vec!["the configuration file adunata.toml sets smtp-host, which is no setting"]
        );
    }

    #[test]
    fn unsupported_values_in_file() {
        let problems = file_problems(
            "[smtp]\nport = true\nhost = [\"mail.example.org\"]\n[http]\ntrusted-proxies = []\n",
        );
        assert_eq!(
            problems,
            vec![
                "http.trusted-proxies in the configuration file adunata.toml must be a string or a non-empty array of strings",
                "smtp.host in the configuration file adunata.toml must be a string or a whole number",
                "smtp.port in the configuration file adunata.toml must be a string or a whole number",
            ]
        );
        assert!(!problems.iter().any(|problem| problem.contains("true")));
    }

    #[test]
    fn parse_error_names_line_and_column_but_not_the_line() {
        let problem = parse_file("adunata.toml", "[database]\npassword = pa$$word\n")
            .expect_err("file must not parse")
            .to_string();
        assert!(
            problem.starts_with(
                "the configuration file adunata.toml cannot be parsed at line 2, column 12: "
            ),
            "{problem}"
        );
        assert!(!problem.contains("pa$$word"), "{problem}");
    }

    const REQUIRED_FILE: &str = "[http]\npublic-url = \"https://adunata.example.org\"\nlisten-address = \"no address\"\n[database]\nurl = \"postgres://adunata@localhost/adunata\"\n[smtp]\nhost = \"mail.example.org\"\n[mail]\nsender = \"Adunata <adunata@example.org>\"\n[localization]\ntime-zone = \"Europe/Berlin\"\n";

    /// A temporary directory with settings files that is removed when the test ends.
    struct SettingsDirectory {
        path: std::path::PathBuf,
    }

    impl Drop for SettingsDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    fn settings_directory(files: &[(&str, &str)]) -> SettingsDirectory {
        let path = std::env::temp_dir().join(format!(
            "adunata-settings-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        for (name, content) in files {
            std::fs::write(path.join(name), content).unwrap();
        }
        SettingsDirectory { path }
    }

    #[test]
    fn settings_parser_takes_plain_lines() {
        let content = "# comment\n\n  # indented comment\nA=1\nB=x=y\nEMPTY=\n  INDENTED=x\nHASH=a#b\nURL=postgres://a:b@c/d?e=f\r\n";
        assert_eq!(
            parse_settings(content),
            Ok(vec![
                ("A".to_owned(), "1".to_owned()),
                ("B".to_owned(), "x=y".to_owned()),
                ("INDENTED".to_owned(), "x".to_owned()),
                ("HASH".to_owned(), "a#b".to_owned()),
                ("URL".to_owned(), "postgres://a:b@c/d?e=f".to_owned()),
            ])
        );
    }

    #[test]
    fn settings_parser_names_the_malformed_line() {
        for line in [
            "export ADNT_SMTP_PORT=1025",
            "NO_EQUALS",
            "1KEY=x",
            "=x",
            "ADNT_MAIL_SENDER=\"Adunata <adunata@localhost>\"",
            "ADNT_LOCALIZATION_TIME_ZONE='Europe/Berlin'",
            "ADNT_SMTP_PORT=1025 # mailpit",
            "ADNT_SMTP_HOST=localhost\t# mailpit",
            "ADNT_SMTP_HOST=localhost ",
        ] {
            assert_eq!(
                parse_settings(&format!("A=1\n\n{line}\n")),
                Err(3),
                "{line}"
            );
        }
    }

    #[test]
    fn environment_wins_over_local_file_over_shared_file() {
        let directory = settings_directory(&[
            (
                ".env",
                "ALL=shared\nFILES=shared\nSHARED=shared\nBLANKED=shared\n",
            ),
            (
                ".env.local",
                "ALL=local\nFILES=first\nBLANKED=\nFILES=local\n",
            ),
        ]);
        let files = SettingsFiles::read(Some(&directory.path));
        let environment = |name: &str| (name == "ALL").then(|| "environment".to_owned());
        let lookup = |name| files.layered(name, &environment);
        assert_eq!(lookup("ALL").as_deref(), Some("environment"));
        assert_eq!(lookup("FILES").as_deref(), Some("local"));
        assert_eq!(lookup("SHARED").as_deref(), Some("shared"));
        assert_eq!(lookup("BLANKED").as_deref(), Some("shared"));
        assert_eq!(lookup("UNSET"), None);
        assert!(files.problems.is_empty());
    }

    #[test]
    fn configuration_file_wins_over_settings_files() {
        let file = secret_file(&REQUIRED_FILE.replace("listen-address = \"no address\"\n", ""));
        let directory = settings_directory(&[
            (".env", "ADNT_SMTP_HOST=localhost\nADNT_SMTP_PORT=1025\n"),
            (".env.local", "ADNT_HTTP_LISTEN_ADDRESS=127.0.0.1:18010\n"),
        ]);
        let configuration = Configuration::from_sources(
            &|_| None,
            Some(&file),
            SettingsFiles::read(Some(&directory.path)),
        )
        .expect("configuration must be accepted");
        assert_eq!(configuration.smtp.host, "mail.example.org");
        assert_eq!(configuration.smtp.port, 1025);
        assert_eq!(configuration.listen_address.to_string(), "127.0.0.1:18010");
    }

    #[test]
    fn missing_settings_files_count_as_empty() {
        let files = SettingsFiles::read(Some(&settings_directory(&[]).path));
        assert!(files.files.iter().all(HashMap::is_empty));
        assert!(files.problems.is_empty());
    }

    #[test]
    fn without_a_directory_no_file_is_read() {
        let files = SettingsFiles::read(None);
        assert!(files.files.iter().all(HashMap::is_empty));
        assert!(files.problems.is_empty());
    }

    #[test]
    fn malformed_settings_line_names_file_and_line() {
        let directory = settings_directory(&[(".env.local", "A=1\nexport ADNT_SMTP_PORT=1025\n")]);
        let files = SettingsFiles::read(Some(&directory.path));
        let path = directory.path.join(".env.local").display().to_string();
        assert_eq!(
            files.problems,
            vec![Problem::MalformedSettingsLine {
                path: path.clone(),
                line: 2
            }]
        );
        assert_eq!(
            files.problems[0].to_string(),
            format!(
                "line 2 of {path} must have the form KEY=value without quotes or trailing comments"
            )
        );
    }

    #[test]
    fn start_from_a_configuration_file() {
        let file = secret_file(&REQUIRED_FILE.replace("listen-address = \"no address\"\n", ""));
        let configuration =
            Configuration::from_sources(&|_| None, Some(&file), SettingsFiles::default())
                .expect("configuration must be accepted");
        assert_eq!(configuration.smtp.host, "mail.example.org");
    }

    #[test]
    fn environment_overrides_the_file() {
        let file = secret_file(REQUIRED_FILE);
        let configuration = Configuration::from_sources(
            &|name| (name == "ADNT_HTTP_LISTEN_ADDRESS").then(|| "127.0.0.1:9000".to_owned()),
            Some(&file),
            SettingsFiles::default(),
        )
        .expect("configuration must be accepted");
        assert_eq!(configuration.listen_address.to_string(), "127.0.0.1:9000");

        let problems =
            Configuration::from_sources(&|_| None, Some(&file), SettingsFiles::default())
                .err()
                .expect("listen address of the file must be rejected");
        assert_eq!(
            problems,
            vec![Problem::Malformed {
                setting: "ADNT_HTTP_LISTEN_ADDRESS",
                expected: "a socket address such as 127.0.0.1:8080"
            }]
        );
    }

    #[test]
    fn problems_of_the_file_join_the_others() {
        let file = secret_file("[smpt]\nhost = \"mail.example.org\"\n");
        let problems =
            Configuration::from_sources(&|_| None, Some(&file), SettingsFiles::default())
                .err()
                .expect("configuration must be rejected");
        assert!(
            matches!(problems[0], Problem::UnknownKey { .. }),
            "{problems:?}"
        );
        assert!(problems.contains(&Problem::Missing {
            setting: "ADNT_HTTP_PUBLIC_URL"
        }));
    }

    #[test]
    fn credentials_in_the_file_take_only_strings() {
        let problems =
            file_problems("[database]\npassword = 0x1F\n[smtp]\nusername = 1_000\npassword = 5\n");
        assert_eq!(
            problems,
            vec![
                "database.password in the configuration file adunata.toml must be a string",
                "smtp.password in the configuration file adunata.toml must be a string",
                "smtp.username in the configuration file adunata.toml must be a string",
            ]
        );
        assert_eq!(
            file_settings("[database]\npassword = \"0x1F\"\n"),
            HashMap::from([("ADNT_DATABASE_PASSWORD".to_owned(), "0x1F".to_owned())])
        );
    }

    #[test]
    fn parse_error_without_position() {
        assert_eq!(
            parse_failure("a = 1\n", None, "invalid table"),
            "cannot be parsed: invalid table"
        );
    }

    #[test]
    fn start_with_all_required_settings() {
        let configuration = load(&required()).expect("configuration must be accepted");
        assert_eq!(
            configuration.listen_address.to_string(),
            DEFAULT_LISTEN_ADDRESS
        );
        assert_eq!(configuration.smtp.host, "mail.example.org");
        assert_eq!(configuration.smtp.port, 587);
        assert_eq!(configuration.smtp.encryption, Encryption::StartTls);
        assert_eq!(configuration.mail_max_attempts, 8);
        assert_eq!(
            configuration.retention,
            Retention {
                comments: RetentionPeriod::Days(30),
                responses: RetentionPeriod::Days(365),
            }
        );
        assert_eq!(configuration.legal_links, LegalLinks::default());
        assert_eq!(configuration.first_organizer, None);
        assert_eq!(
            configuration.trusted_proxies,
            vec![
                Network::parse("127.0.0.1/32").unwrap(),
                Network::parse("::1/128").unwrap()
            ]
        );
        assert_eq!(
            configuration.mail_sender.email.to_string(),
            "adunata@example.org"
        );
    }

    #[test]
    fn read_a_secret_from_a_file() {
        let mut variables = required();
        variables.insert("ADNT_DATABASE_PASSWORD_FILE", secret_file("s3cret\n"));
        variables.insert("ADNT_SMTP_PASSWORD_FILE", secret_file("mail-s3cret"));
        let configuration = load(&variables).expect("configuration must be accepted");
        assert_eq!(configuration.smtp.password.as_deref(), Some("mail-s3cret"));
        let debug = format!("{:?}", configuration.database);
        assert!(
            debug.contains("s3cret"),
            "database password must come from the file"
        );
    }

    #[test]
    fn secret_as_a_plain_value() {
        let mut variables = required();
        variables.insert("ADNT_DATABASE_PASSWORD", "db-pl41n".to_owned());
        variables.insert("ADNT_SMTP_PASSWORD", "mail-pl41n".to_owned());
        let configuration = load(&variables).expect("configuration must be accepted");
        assert_eq!(configuration.smtp.password.as_deref(), Some("mail-pl41n"));
        assert!(database_debug(&configuration).contains("db-pl41n"));
        assert_no_problem_shows("pl41n", &variables, &HashMap::new());
    }

    #[test]
    fn both_forms_in_one_source() {
        let mut variables = required();
        variables.insert("ADNT_DATABASE_PASSWORD", "db-pl41n".to_owned());
        variables.insert("ADNT_DATABASE_PASSWORD_FILE", secret_file("db-f1le"));
        let problems = problems(&variables);
        assert_eq!(
            problems,
            vec![Problem::BothForms {
                plain: "ADNT_DATABASE_PASSWORD",
                file: "ADNT_DATABASE_PASSWORD_FILE"
            }]
        );
        let message = problems[0].to_string();
        assert!(message.contains("ADNT_DATABASE_PASSWORD "), "{message}");
        assert!(message.contains("ADNT_DATABASE_PASSWORD_FILE"), "{message}");
        assert_no_problem_shows("pl41n", &variables, &HashMap::new());
        assert_no_problem_shows("f1le", &variables, &HashMap::new());
    }

    #[test]
    fn plain_value_in_first_source_wins_over_file_in_second() {
        let mut environment = required();
        environment.insert("ADNT_DATABASE_PASSWORD", "db-pl41n".to_owned());
        let file = HashMap::from([("ADNT_DATABASE_PASSWORD_FILE", secret_file("db-f1le"))]);
        let configuration = load_two(&environment, &file).expect("configuration must be accepted");
        let debug = database_debug(&configuration);
        assert!(debug.contains("db-pl41n"), "{debug}");
        assert!(!debug.contains("db-f1le"), "{debug}");
        assert_no_problem_shows("pl41n", &environment, &file);
        assert_no_problem_shows("f1le", &environment, &file);
    }

    #[test]
    fn empty_secret_file_does_not_fall_back() {
        let mut environment = required();
        environment.insert("ADNT_DATABASE_PASSWORD_FILE", secret_file("\n"));
        let file = HashMap::from([("ADNT_DATABASE_PASSWORD", "db-pl41n".to_owned())]);
        let configuration = load_two(&environment, &file).expect("configuration must be accepted");
        assert!(!database_debug(&configuration).contains("db-pl41n"));
        assert_no_problem_shows("pl41n", &environment, &file);
    }

    #[test]
    fn unreadable_secret_file_does_not_fall_back() {
        let mut environment = required();
        environment.insert(
            "ADNT_SMTP_PASSWORD_FILE",
            "/nonexistent/adunata-secret".to_owned(),
        );
        let file = HashMap::from([("ADNT_SMTP_PASSWORD", "mail-pl41n".to_owned())]);
        let problems = load_two(&environment, &file)
            .err()
            .expect("configuration must be rejected");
        assert_eq!(
            problems,
            vec![Problem::Unreadable {
                setting: "ADNT_SMTP_PASSWORD_FILE",
                path: "/nonexistent/adunata-secret".to_owned()
            }]
        );
        assert_no_problem_shows("pl41n", &environment, &file);
    }

    #[test]
    fn empty_secret_file_counts_as_unset() {
        let mut variables = required();
        variables.insert("ADNT_SMTP_USERNAME", "adunata@example.org".to_owned());
        variables.insert("ADNT_SMTP_PASSWORD_FILE", secret_file("\n"));
        let configuration = load(&variables).expect("configuration must be accepted");
        assert_eq!(configuration.smtp.password, None);
    }

    #[test]
    fn secret_file_unreadable() {
        let mut variables = required();
        variables.insert(
            "ADNT_SMTP_PASSWORD_FILE",
            "/nonexistent/adunata-secret".to_owned(),
        );
        let problems = problems(&variables);
        assert_eq!(
            problems,
            vec![Problem::Unreadable {
                setting: "ADNT_SMTP_PASSWORD_FILE",
                path: "/nonexistent/adunata-secret".to_owned()
            }]
        );
        assert!(problems[0].to_string().contains("ADNT_SMTP_PASSWORD_FILE"));
    }

    #[test]
    fn several_settings_missing() {
        let mut variables = required();
        variables.remove("ADNT_HTTP_PUBLIC_URL");
        variables.remove("ADNT_SMTP_HOST");
        variables.remove("ADNT_LOCALIZATION_TIME_ZONE");
        assert_eq!(
            problems(&variables),
            vec![
                Problem::Missing {
                    setting: "ADNT_HTTP_PUBLIC_URL"
                },
                Problem::Missing {
                    setting: "ADNT_SMTP_HOST"
                },
                Problem::Missing {
                    setting: "ADNT_LOCALIZATION_TIME_ZONE"
                },
            ]
        );
    }

    #[test]
    fn unknown_time_zone() {
        let mut variables = required();
        variables.insert(
            "ADNT_LOCALIZATION_TIME_ZONE",
            "Europe/Wernigerode".to_owned(),
        );
        variables.remove("ADNT_SMTP_HOST");
        let problems = problems(&variables);
        assert_eq!(
            problems,
            vec![
                Problem::Missing {
                    setting: "ADNT_SMTP_HOST"
                },
                Problem::Malformed {
                    setting: "ADNT_LOCALIZATION_TIME_ZONE",
                    expected: "an IANA time zone such as Europe/Berlin"
                },
            ]
        );
    }

    #[test]
    fn optional_settings() {
        let mut variables = required();
        variables.insert("ADNT_RETENTION_COMMENT_DAYS", "never".to_owned());
        variables.insert("ADNT_RETENTION_RESPONSE_DAYS", "90".to_owned());
        variables.insert(
            "ADNT_LEGAL_PRIVACY_URL",
            "https://choir.example.org/privacy".to_owned(),
        );
        variables.insert(
            "ADNT_LEGAL_IMPRINT_URL",
            "https://choir.example.org/imprint".to_owned(),
        );
        variables.insert(
            "ADNT_BOOTSTRAP_ORGANIZER_MAIL",
            "ada@example.org".to_owned(),
        );
        variables.insert("ADNT_BOOTSTRAP_ORGANIZER_FIRST_NAME", "Ada".to_owned());
        variables.insert("ADNT_BOOTSTRAP_ORGANIZER_LAST_NAME", "Lovelace".to_owned());
        variables.insert("ADNT_HTTP_TRUSTED_PROXIES", "172.18.0.0/16".to_owned());
        let configuration = load(&variables).expect("configuration must be accepted");
        assert_eq!(
            configuration.retention,
            Retention {
                comments: RetentionPeriod::Never,
                responses: RetentionPeriod::Days(90),
            }
        );
        assert_eq!(
            configuration.legal_links.privacy.as_deref(),
            Some("https://choir.example.org/privacy")
        );
        assert_eq!(
            configuration.first_organizer,
            Some(FirstOrganizer {
                mail: "ada@example.org".to_owned(),
                first_name: "Ada".to_owned(),
                last_name: "Lovelace".to_owned(),
            })
        );
        assert_eq!(configuration.trusted_proxies.len(), 1);
    }

    #[test]
    fn first_organizer_without_both_names() {
        let mut variables = required();
        variables.insert(
            "ADNT_BOOTSTRAP_ORGANIZER_MAIL",
            "ada@example.org".to_owned(),
        );
        variables.insert("ADNT_BOOTSTRAP_ORGANIZER_FIRST_NAME", "Ada".to_owned());
        let problems = problems(&variables);
        assert_eq!(
            problems,
            vec![Problem::RequiredWith {
                setting: "ADNT_BOOTSTRAP_ORGANIZER_LAST_NAME",
                with: "ADNT_BOOTSTRAP_ORGANIZER_MAIL"
            }]
        );
        assert_eq!(
            problems[0].to_string(),
            "ADNT_BOOTSTRAP_ORGANIZER_LAST_NAME is required when ADNT_BOOTSTRAP_ORGANIZER_MAIL is set"
        );
    }

    #[test]
    fn retention_other_than_days_or_never() {
        for value in ["-1", "30d", "forever", "1.5"] {
            let mut variables = required();
            variables.insert("ADNT_RETENTION_COMMENT_DAYS", value.to_owned());
            assert_eq!(
                problems(&variables),
                vec![Problem::Malformed {
                    setting: "ADNT_RETENTION_COMMENT_DAYS",
                    expected: "a whole number of days or never"
                }],
                "{value} must be rejected"
            );
        }
    }

    #[test]
    fn malformed_optional_settings() {
        let mut variables = required();
        variables.insert(
            "ADNT_LEGAL_PRIVACY_URL",
            "choir.example.org/privacy".to_owned(),
        );
        variables.insert("ADNT_HTTP_TRUSTED_PROXIES", "10.0.0.0/33".to_owned());
        variables.insert("ADNT_BOOTSTRAP_ORGANIZER_MAIL", "ada".to_owned());
        variables.insert("ADNT_BOOTSTRAP_ORGANIZER_FIRST_NAME", "Ada".to_owned());
        variables.insert("ADNT_BOOTSTRAP_ORGANIZER_LAST_NAME", "Lovelace".to_owned());
        let settings: Vec<&str> = problems(&variables)
            .iter()
            .map(|problem| match problem {
                Problem::Malformed { setting, .. } => *setting,
                other => panic!("unexpected {other:?}"),
            })
            .collect();
        assert_eq!(
            settings,
            vec![
                "ADNT_HTTP_TRUSTED_PROXIES",
                "ADNT_LEGAL_PRIVACY_URL",
                "ADNT_BOOTSTRAP_ORGANIZER_MAIL"
            ]
        );
    }

    #[test]
    fn network_contains_its_addresses_only() {
        let docker = Network::parse("172.18.0.0/16").unwrap();
        assert!(docker.contains("172.18.4.2".parse().unwrap()));
        assert!(docker.contains("::ffff:172.18.4.2".parse().unwrap()));
        assert!(!docker.contains("172.19.0.1".parse().unwrap()));
        assert!(!docker.contains("::1".parse().unwrap()));
        let loopback = Network::parse("::1/128").unwrap();
        assert!(loopback.contains("::1".parse().unwrap()));
        assert!(!loopback.contains("::2".parse().unwrap()));
        assert!(
            Network::parse("0.0.0.0/0")
                .unwrap()
                .contains("203.0.113.9".parse().unwrap())
        );
    }

    fn berlin_wall_clock(instant: OffsetDateTime) -> PrimitiveDateTime {
        InstanceTimeZone::parse("Europe/Berlin")
            .unwrap()
            .wall_clock(instant)
    }

    #[test]
    fn wall_clock_around_the_start_of_summer_time() {
        use time::macros::datetime;
        assert_eq!(
            berlin_wall_clock(datetime!(2027-03-28 00:59:59 UTC)),
            datetime!(2027-03-28 01:59:59)
        );
        assert_eq!(
            berlin_wall_clock(datetime!(2027-03-28 01:00 UTC)),
            datetime!(2027-03-28 03:00)
        );
    }

    #[test]
    fn wall_clock_around_the_end_of_summer_time() {
        use time::macros::datetime;
        assert_eq!(
            berlin_wall_clock(datetime!(2027-10-31 00:30 UTC)),
            datetime!(2027-10-31 02:30)
        );
        assert_eq!(
            berlin_wall_clock(datetime!(2027-10-31 01:30 UTC)),
            datetime!(2027-10-31 02:30)
        );
        assert_eq!(
            berlin_wall_clock(datetime!(2027-10-31 02:00 UTC)),
            datetime!(2027-10-31 03:00)
        );
    }

    #[test]
    fn wall_clock_keeps_the_date_of_the_instance() {
        use time::macros::datetime;
        assert_eq!(
            berlin_wall_clock(datetime!(2027-03-13 23:30 UTC)),
            datetime!(2027-03-14 00:30)
        );
    }

    #[test]
    fn malformed_value() {
        for url in [
            "http://adunata.example.org",
            "adunata.example.org",
            "https://adunata.example.org/?a=b",
        ] {
            let mut variables = required();
            variables.insert("ADNT_HTTP_PUBLIC_URL", url.to_owned());
            let problems = problems(&variables);
            assert_eq!(problems.len(), 1, "{url} must be rejected");
            let message = problems[0].to_string();
            assert!(
                message.contains("ADNT_HTTP_PUBLIC_URL") && message.contains("HTTPS"),
                "{message}"
            );
        }
    }

    #[test]
    fn localhost_may_use_http() {
        let mut variables = required();
        variables.insert("ADNT_HTTP_PUBLIC_URL", "http://localhost:8080/".to_owned());
        let configuration = load(&variables).expect("configuration must be accepted");
        assert_eq!(
            configuration.public_url.link("/gatherings"),
            "http://localhost:8080/gatherings"
        );
    }
}
