mod api;
mod client;
mod database;
mod mail;
mod spike;

use std::net::SocketAddr;
use std::time::Duration;

use sqlx::postgres::PgConnectOptions;

const DEFAULT_LISTEN_ADDRESS: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() {
    if std::env::args().nth(1).as_deref() == Some("openapi") {
        let document = api::document()
            .to_pretty_json()
            .expect("OpenAPI document must serialize");
        println!("{document}");
        return;
    }
    serve().await;
}

async fn serve() {
    let address: SocketAddr = std::env::var("ADUNATA_LISTEN_ADDRESS")
        .unwrap_or_else(|_| DEFAULT_LISTEN_ADDRESS.to_owned())
        .parse()
        .expect("ADUNATA_LISTEN_ADDRESS must be a socket address such as 127.0.0.1:8080");
    let database: PgConnectOptions = std::env::var("ADUNATA_DATABASE_URL")
        .expect("ADUNATA_DATABASE_URL must be set")
        .parse()
        .expect("ADUNATA_DATABASE_URL must be a PostgreSQL URL");
    let pool = database::connect(database)
        .await
        .expect("database must be reachable and migrations must apply");
    let delivery = mail::Delivery {
        transport: mail::unencrypted_transport(
            &std::env::var("ADUNATA_SMTP_HOST").unwrap_or_else(|_| "localhost".to_owned()),
            std::env::var("ADUNATA_SMTP_PORT")
                .ok()
                .and_then(|port| port.parse().ok())
                .unwrap_or(1025),
        ),
        sender: "Adunata <adunata@example.org>"
            .parse()
            .expect("sender must be a mailbox"),
        max_attempts: 8,
    };
    tokio::spawn(mail::run(pool.clone(), delivery, Duration::from_secs(2)));
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("listen address must be free");
    axum::serve(listener, app(pool))
        .await
        .expect("server must keep running");
}

fn app(pool: sqlx::PgPool) -> axum::Router {
    api::router(pool).merge(client::router())
}
