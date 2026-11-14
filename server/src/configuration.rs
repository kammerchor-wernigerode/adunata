use std::net::SocketAddr;

use sqlx::postgres::PgConnectOptions;

const DEFAULT_LISTEN_ADDRESS: &str = "127.0.0.1:8080";
const DEFAULT_SMTP_HOST: &str = "localhost";
const DEFAULT_SMTP_PORT: u16 = 1025;

pub struct Configuration {
    pub listen_address: SocketAddr,
    pub database: PgConnectOptions,
    pub smtp_host: String,
    pub smtp_port: u16,
}

impl Configuration {
    pub fn from_environment() -> Self {
        Self {
            listen_address: variable("ADUNATA_LISTEN_ADDRESS")
                .unwrap_or_else(|| DEFAULT_LISTEN_ADDRESS.to_owned())
                .parse()
                .expect("ADUNATA_LISTEN_ADDRESS must be a socket address such as 127.0.0.1:8080"),
            database: variable("ADUNATA_DATABASE_URL")
                .expect("ADUNATA_DATABASE_URL must be set")
                .parse()
                .expect("ADUNATA_DATABASE_URL must be a PostgreSQL URL"),
            smtp_host: variable("ADUNATA_SMTP_HOST")
                .unwrap_or_else(|| DEFAULT_SMTP_HOST.to_owned()),
            smtp_port: variable("ADUNATA_SMTP_PORT").map_or(DEFAULT_SMTP_PORT, |port| {
                port.parse()
                    .expect("ADUNATA_SMTP_PORT must be a port number")
            }),
        }
    }
}

fn variable(name: &str) -> Option<String> {
    std::env::var(name).ok()
}
