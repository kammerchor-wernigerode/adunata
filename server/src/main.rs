mod api;
mod client;
mod configuration;
mod database;
mod health;
mod localization;
mod mail;
mod serving;
mod spike;

use std::time::Duration;

use configuration::Configuration;

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
    tracing_subscriber::fmt().json().init();
    let configuration = Configuration::from_environment().unwrap_or_else(|problems| {
        for problem in problems {
            tracing::error!(%problem, "configuration is invalid");
        }
        std::process::exit(1);
    });
    let pool = database::connect(configuration.database)
        .await
        .unwrap_or_else(|error| {
            tracing::error!(%error, "database is not ready");
            std::process::exit(1);
        });
    let delivery = mail::Delivery {
        transport: mail::transport(&configuration.smtp).unwrap_or_else(|error| {
            tracing::error!(%error, "mail server settings are unusable");
            std::process::exit(1);
        }),
        sender: configuration.mail_sender,
        max_attempts: configuration.mail_max_attempts,
    };
    tokio::spawn(mail::run(pool.clone(), delivery, Duration::from_secs(2)));
    let listener = tokio::net::TcpListener::bind(configuration.listen_address)
        .await
        .expect("listen address must be free");
    let state = api::AppState {
        pool,
        public_url: configuration.public_url,
        default_language: configuration.default_language,
    };
    tracing::info!(address = %listener.local_addr().expect("listener must have an address"), "accepting requests");
    serving::serve_until(listener, app(state), serving::termination_signal())
        .await
        .expect("server must keep running until it is asked to stop");
}

fn app(state: api::AppState) -> axum::Router {
    api::router(state).merge(client::router())
}
