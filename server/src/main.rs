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
    let configuration = Configuration::from_environment();
    let pool = database::connect(configuration.database)
        .await
        .expect("database must be reachable and migrations must apply");
    let delivery = mail::Delivery {
        transport: mail::unencrypted_transport(&configuration.smtp_host, configuration.smtp_port),
        sender: "Adunata <adunata@example.org>"
            .parse()
            .expect("sender must be a mailbox"),
        max_attempts: 8,
    };
    tokio::spawn(mail::run(pool.clone(), delivery, Duration::from_secs(2)));
    let listener = tokio::net::TcpListener::bind(configuration.listen_address)
        .await
        .expect("listen address must be free");
    axum::serve(listener, app(pool))
        .await
        .expect("server must keep running");
}

fn app(pool: sqlx::PgPool) -> axum::Router {
    api::router(pool).merge(client::router())
}
