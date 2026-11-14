mod api;
mod client;
mod configuration;
mod database;
mod localization;
mod mail;
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
    let configuration = Configuration::from_environment().unwrap_or_else(|problems| {
        eprintln!("Adunata cannot start because its configuration is invalid:");
        for problem in problems {
            eprintln!("- {problem}");
        }
        std::process::exit(1);
    });
    let pool = database::connect(configuration.database)
        .await
        .expect("database must be reachable and migrations must apply");
    let delivery = mail::Delivery {
        transport: mail::unencrypted_transport(&configuration.smtp),
        sender: configuration.mail_sender,
        max_attempts: 8,
    };
    tokio::spawn(mail::run(pool.clone(), delivery, Duration::from_secs(2)));
    let listener = tokio::net::TcpListener::bind(configuration.listen_address)
        .await
        .expect("listen address must be free");
    let state = api::AppState {
        pool,
        public_url: configuration.public_url,
    };
    axum::serve(listener, app(state))
        .await
        .expect("server must keep running");
}

fn app(state: api::AppState) -> axum::Router {
    api::router(state).merge(client::router())
}
