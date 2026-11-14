use std::fmt;
use std::net::SocketAddr;

use axum::http::Uri;
use lettre::message::Mailbox;
use sqlx::postgres::PgConnectOptions;

const DEFAULT_LISTEN_ADDRESS: &str = "127.0.0.1:8080";
const DEFAULT_SMTP_PORT: &str = "587";
const DEFAULT_SMTP_ENCRYPTION: &str = "starttls";
const DEFAULT_MAIL_MAX_ATTEMPTS: &str = "8";
const DEFAULT_LANGUAGE: &str = "en";

pub struct Configuration {
    pub listen_address: SocketAddr,
    pub public_url: PublicUrl,
    pub database: PgConnectOptions,
    pub smtp: Smtp,
    pub mail_sender: Mailbox,
    pub mail_max_attempts: i32,
    pub default_language: String,
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

    pub fn link(&self, path: &str) -> String {
        format!("{}/{}", self.base, path.trim_start_matches('/'))
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
}

impl fmt::Display for Problem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { setting } => write!(formatter, "{setting} is required but not set"),
            Self::Malformed { setting, expected } => {
                write!(formatter, "{setting} must be {expected}")
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

        match (
            listen_address,
            public_url,
            database,
            smtp,
            mail_sender,
            mail_max_attempts,
            default_language,
        ) {
            (
                Some(listen_address),
                Some(public_url),
                Some(database),
                Some(smtp),
                Some(mail_sender),
                Some(mail_max_attempts),
                Some(default_language),
            ) if problems.is_empty() => Ok(Self {
                listen_address,
                public_url,
                database,
                smtp,
                mail_sender,
                mail_max_attempts,
                default_language,
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
    Some(content.trim_end_matches(['\r', '\n']).to_owned())
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
        assert_eq!(
            problems(&variables),
            vec![
                Problem::Missing {
                    setting: "ADUNATA_PUBLIC_URL"
                },
                Problem::Missing {
                    setting: "ADUNATA_SMTP_HOST"
                },
            ]
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
