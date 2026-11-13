mod api;
mod client;
mod spike;

use std::net::SocketAddr;

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
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("listen address must be free");
    axum::serve(listener, app())
        .await
        .expect("server must keep running");
}

fn app() -> axum::Router {
    api::router().merge(client::router())
}
