use std::fmt;
use std::net::{IpAddr, SocketAddr};

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
const FIRST_ORGANIZER_MAIL: &str = "ADUNATA_FIRST_ORGANIZER_MAIL";

pub struct Configuration {
    pub listen_address: SocketAddr,
    pub public_url: PublicUrl,
    pub database: PgConnectOptions,
    pub smtp: Smtp,
    pub mail_sender: Mailbox,
    pub mail_max_attempts: i32,
    pub default_language: String,
    #[expect(dead_code, reason = "nothing compares with the present yet")]
    pub time_zone: InstanceTimeZone,
    pub trusted_proxies: Vec<Network>,
    #[cfg_attr(not(test), expect(dead_code, reason = "no personal data expires yet"))]
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
        format!(
            "{}://{}",
            uri.scheme_str().expect("public URL has a scheme"),
            uri.authority().expect("public URL has a host")
        )
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

    /// Turns an instant into the date and wall-clock time of the instance.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "nothing compares with the present yet")
    )]
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
            Self::Unreadable { setting, path } => {
                write!(
                    formatter,
                    "{setting} names the file {path}, which cannot be read"
                )
            }
        }
    }
}

impl Configuration {
    pub fn from_environment() -> Result<Self, Vec<Problem>> {
        Self::load(&|name| std::env::var(name).ok().filter(|value| !value.is_empty()))
    }

    pub fn load(variable: &dyn Fn(&str) -> Option<String>) -> Result<Self, Vec<Problem>> {
        let mut problems = Vec::new();
        let listen_address = parse(
            &mut problems,
            "ADUNATA_LISTEN_ADDRESS",
            "a socket address such as 127.0.0.1:8080",
            Some(with_default(variable, "ADUNATA_LISTEN_ADDRESS", DEFAULT_LISTEN_ADDRESS).as_str()),
            |value| value.parse().ok(),
        );
        let public_url = parse(
            &mut problems,
            "ADUNATA_PUBLIC_URL",
            "an absolute HTTPS URL such as https://adunata.example.org",
            variable("ADUNATA_PUBLIC_URL").as_deref(),
            parse_public_url,
        );
        let database = load_database(&mut problems, variable);
        let smtp = load_smtp(&mut problems, variable);
        let mail_sender = parse(
            &mut problems,
            "ADUNATA_MAIL_SENDER",
            "a mailbox such as Adunata <adunata@example.org>",
            variable("ADUNATA_MAIL_SENDER").as_deref(),
            |value| value.parse::<Mailbox>().ok(),
        );
        let mail_max_attempts = parse(
            &mut problems,
            "ADUNATA_MAIL_MAX_ATTEMPTS",
            "a whole number from 1 to 100",
            Some(
                with_default(
                    variable,
                    "ADUNATA_MAIL_MAX_ATTEMPTS",
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
        );

        let default_language = parse(
            &mut problems,
            "ADUNATA_DEFAULT_LANGUAGE",
            "a supported language: de or en",
            Some(with_default(variable, "ADUNATA_DEFAULT_LANGUAGE", DEFAULT_LANGUAGE).as_str()),
            |value| crate::localization::is_supported(value).then(|| value.to_owned()),
        );

        let time_zone = load_time_zone(&mut problems, variable);
        let trusted_proxies = load_trusted_proxies(&mut problems, variable);
        let retention = load_retention(&mut problems, variable);
        let legal_links = LegalLinks {
            privacy: optional_link(&mut problems, "ADUNATA_PRIVACY_URL", variable),
            imprint: optional_link(&mut problems, "ADUNATA_IMPRINT_URL", variable),
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
    variable: &dyn Fn(&str) -> Option<String>,
) -> Option<PgConnectOptions> {
    let database = parse(
        problems,
        "ADUNATA_DATABASE_URL",
        "a PostgreSQL URL such as postgres://adunata@localhost/adunata",
        variable("ADUNATA_DATABASE_URL").as_deref(),
        |value| value.parse::<PgConnectOptions>().ok(),
    );
    let password = secret(problems, "ADUNATA_DATABASE_PASSWORD_FILE", variable);
    match password {
        Some(password) => database.map(|database| database.password(&password)),
        None => database,
    }
}

fn load_smtp(
    problems: &mut Vec<Problem>,
    variable: &dyn Fn(&str) -> Option<String>,
) -> Option<Smtp> {
    let host = parse(
        problems,
        "ADUNATA_SMTP_HOST",
        "a host name",
        variable("ADUNATA_SMTP_HOST").as_deref(),
        |value| Some(value.to_owned()),
    );
    let port = parse(
        problems,
        "ADUNATA_SMTP_PORT",
        "a port number",
        Some(with_default(variable, "ADUNATA_SMTP_PORT", DEFAULT_SMTP_PORT).as_str()),
        |value| value.parse().ok(),
    );
    let encryption = parse(
        problems,
        "ADUNATA_SMTP_ENCRYPTION",
        "one of starttls, tls or none",
        Some(with_default(variable, "ADUNATA_SMTP_ENCRYPTION", DEFAULT_SMTP_ENCRYPTION).as_str()),
        Encryption::parse,
    );
    let password = secret(problems, "ADUNATA_SMTP_PASSWORD_FILE", variable);
    Some(Smtp {
        host: host?,
        port: port?,
        encryption: encryption?,
        username: variable("ADUNATA_SMTP_USERNAME"),
        password,
    })
}

fn load_time_zone(
    problems: &mut Vec<Problem>,
    variable: &dyn Fn(&str) -> Option<String>,
) -> Option<InstanceTimeZone> {
    parse(
        problems,
        "ADUNATA_TIME_ZONE",
        "an IANA time zone such as Europe/Berlin",
        variable("ADUNATA_TIME_ZONE").as_deref(),
        InstanceTimeZone::parse,
    )
}

fn load_trusted_proxies(
    problems: &mut Vec<Problem>,
    variable: &dyn Fn(&str) -> Option<String>,
) -> Option<Vec<Network>> {
    parse(
        problems,
        "ADUNATA_TRUSTED_PROXIES",
        "a comma-separated list of networks such as 172.18.0.0/16,::1/128",
        Some(with_default(variable, "ADUNATA_TRUSTED_PROXIES", DEFAULT_TRUSTED_PROXIES).as_str()),
        |value| value.split(',').map(Network::parse).collect(),
    )
}

fn load_retention(
    problems: &mut Vec<Problem>,
    variable: &dyn Fn(&str) -> Option<String>,
) -> Option<Retention> {
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
        "ADUNATA_RETENTION_COMMENT_DAYS",
        DEFAULT_RETENTION_COMMENT_DAYS,
    );
    let responses = period(
        "ADUNATA_RETENTION_RESPONSE_DAYS",
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
    variable: &dyn Fn(&str) -> Option<String>,
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

fn load_first_organizer(
    problems: &mut Vec<Problem>,
    variable: &dyn Fn(&str) -> Option<String>,
) -> Option<FirstOrganizer> {
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
    let first_name = name("ADUNATA_FIRST_ORGANIZER_FIRST_NAME");
    let last_name = name("ADUNATA_FIRST_ORGANIZER_LAST_NAME");
    Some(FirstOrganizer {
        mail: mail?,
        first_name: first_name?,
        last_name: last_name?,
    })
}

fn with_default(variable: &dyn Fn(&str) -> Option<String>, setting: &str, default: &str) -> String {
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

fn secret(
    problems: &mut Vec<Problem>,
    setting: &'static str,
    variable: &dyn Fn(&str) -> Option<String>,
) -> Option<String> {
    let path = variable(setting)?;
    let Ok(content) = std::fs::read_to_string(&path) else {
        problems.push(Problem::Unreadable { setting, path });
        return None;
    };
    let secret = content.trim_end_matches(['\r', '\n']);
    (!secret.is_empty()).then(|| secret.to_owned())
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
                "ADUNATA_PUBLIC_URL",
                "https://adunata.example.org".to_owned(),
            ),
            (
                "ADUNATA_DATABASE_URL",
                "postgres://adunata@localhost/adunata".to_owned(),
            ),
            ("ADUNATA_SMTP_HOST", "mail.example.org".to_owned()),
            (
                "ADUNATA_MAIL_SENDER",
                "Adunata <adunata@example.org>".to_owned(),
            ),
            ("ADUNATA_TIME_ZONE", "Europe/Berlin".to_owned()),
        ])
    }

    fn load(variables: &HashMap<&'static str, String>) -> Result<Configuration, Vec<Problem>> {
        Configuration::load(&|name| variables.get(name).cloned())
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
        variables.insert("ADUNATA_DATABASE_PASSWORD_FILE", secret_file("s3cret\n"));
        variables.insert("ADUNATA_SMTP_PASSWORD_FILE", secret_file("mail-s3cret"));
        let configuration = load(&variables).expect("configuration must be accepted");
        assert_eq!(configuration.smtp.password.as_deref(), Some("mail-s3cret"));
        let debug = format!("{:?}", configuration.database);
        assert!(
            debug.contains("s3cret"),
            "database password must come from the file"
        );
    }

    #[test]
    fn empty_secret_file_counts_as_unset() {
        let mut variables = required();
        variables.insert("ADUNATA_SMTP_USERNAME", "adunata@example.org".to_owned());
        variables.insert("ADUNATA_SMTP_PASSWORD_FILE", secret_file("\n"));
        let configuration = load(&variables).expect("configuration must be accepted");
        assert_eq!(configuration.smtp.password, None);
    }

    #[test]
    fn secret_file_unreadable() {
        let mut variables = required();
        variables.insert(
            "ADUNATA_SMTP_PASSWORD_FILE",
            "/nonexistent/adunata-secret".to_owned(),
        );
        let problems = problems(&variables);
        assert_eq!(
            problems,
            vec![Problem::Unreadable {
                setting: "ADUNATA_SMTP_PASSWORD_FILE",
                path: "/nonexistent/adunata-secret".to_owned()
            }]
        );
        assert!(
            problems[0]
                .to_string()
                .contains("ADUNATA_SMTP_PASSWORD_FILE")
        );
    }

    #[test]
    fn several_settings_missing() {
        let mut variables = required();
        variables.remove("ADUNATA_PUBLIC_URL");
        variables.remove("ADUNATA_SMTP_HOST");
        variables.remove("ADUNATA_TIME_ZONE");
        assert_eq!(
            problems(&variables),
            vec![
                Problem::Missing {
                    setting: "ADUNATA_PUBLIC_URL"
                },
                Problem::Missing {
                    setting: "ADUNATA_SMTP_HOST"
                },
                Problem::Missing {
                    setting: "ADUNATA_TIME_ZONE"
                },
            ]
        );
    }

    #[test]
    fn unknown_time_zone() {
        let mut variables = required();
        variables.insert("ADUNATA_TIME_ZONE", "Europe/Wernigerode".to_owned());
        variables.remove("ADUNATA_SMTP_HOST");
        let problems = problems(&variables);
        assert_eq!(
            problems,
            vec![
                Problem::Missing {
                    setting: "ADUNATA_SMTP_HOST"
                },
                Problem::Malformed {
                    setting: "ADUNATA_TIME_ZONE",
                    expected: "an IANA time zone such as Europe/Berlin"
                },
            ]
        );
    }

    #[test]
    fn optional_settings() {
        let mut variables = required();
        variables.insert("ADUNATA_RETENTION_COMMENT_DAYS", "never".to_owned());
        variables.insert("ADUNATA_RETENTION_RESPONSE_DAYS", "90".to_owned());
        variables.insert(
            "ADUNATA_PRIVACY_URL",
            "https://choir.example.org/privacy".to_owned(),
        );
        variables.insert(
            "ADUNATA_IMPRINT_URL",
            "https://choir.example.org/imprint".to_owned(),
        );
        variables.insert("ADUNATA_FIRST_ORGANIZER_MAIL", "ada@example.org".to_owned());
        variables.insert("ADUNATA_FIRST_ORGANIZER_FIRST_NAME", "Ada".to_owned());
        variables.insert("ADUNATA_FIRST_ORGANIZER_LAST_NAME", "Lovelace".to_owned());
        variables.insert("ADUNATA_TRUSTED_PROXIES", "172.18.0.0/16".to_owned());
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
        variables.insert("ADUNATA_FIRST_ORGANIZER_MAIL", "ada@example.org".to_owned());
        variables.insert("ADUNATA_FIRST_ORGANIZER_FIRST_NAME", "Ada".to_owned());
        let problems = problems(&variables);
        assert_eq!(
            problems,
            vec![Problem::RequiredWith {
                setting: "ADUNATA_FIRST_ORGANIZER_LAST_NAME",
                with: "ADUNATA_FIRST_ORGANIZER_MAIL"
            }]
        );
        assert_eq!(
            problems[0].to_string(),
            "ADUNATA_FIRST_ORGANIZER_LAST_NAME is required when ADUNATA_FIRST_ORGANIZER_MAIL is set"
        );
    }

    #[test]
    fn retention_other_than_days_or_never() {
        for value in ["-1", "30d", "forever", "1.5"] {
            let mut variables = required();
            variables.insert("ADUNATA_RETENTION_COMMENT_DAYS", value.to_owned());
            assert_eq!(
                problems(&variables),
                vec![Problem::Malformed {
                    setting: "ADUNATA_RETENTION_COMMENT_DAYS",
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
            "ADUNATA_PRIVACY_URL",
            "choir.example.org/privacy".to_owned(),
        );
        variables.insert("ADUNATA_TRUSTED_PROXIES", "10.0.0.0/33".to_owned());
        variables.insert("ADUNATA_FIRST_ORGANIZER_MAIL", "ada".to_owned());
        variables.insert("ADUNATA_FIRST_ORGANIZER_FIRST_NAME", "Ada".to_owned());
        variables.insert("ADUNATA_FIRST_ORGANIZER_LAST_NAME", "Lovelace".to_owned());
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
                "ADUNATA_TRUSTED_PROXIES",
                "ADUNATA_PRIVACY_URL",
                "ADUNATA_FIRST_ORGANIZER_MAIL"
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
            variables.insert("ADUNATA_PUBLIC_URL", url.to_owned());
            let problems = problems(&variables);
            assert_eq!(problems.len(), 1, "{url} must be rejected");
            let message = problems[0].to_string();
            assert!(
                message.contains("ADUNATA_PUBLIC_URL") && message.contains("HTTPS"),
                "{message}"
            );
        }
    }

    #[test]
    fn localhost_may_use_http() {
        let mut variables = required();
        variables.insert("ADUNATA_PUBLIC_URL", "http://localhost:8080/".to_owned());
        let configuration = load(&variables).expect("configuration must be accepted");
        assert_eq!(
            configuration.public_url.link("/gatherings"),
            "http://localhost:8080/gatherings"
        );
    }
}
