use axum::Router;
use axum::http::{HeaderValue, header};
use axum::middleware::map_response;
use axum::response::Response;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

/// Serves requests until `stop` fires. Requests in progress then finish while new connections are refused.
pub async fn serve_until(
    listener: TcpListener,
    app: Router,
    stop: oneshot::Receiver<()>,
) -> std::io::Result<()> {
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        let _ = stop.await;
    })
    .await
}

/// Adds the headers that keep pages out of foreign frames and their addresses out of referrers to every response.
pub fn with_security_headers(app: Router) -> Router {
    app.layer(map_response(security_headers))
}

/// Keeps every cache from storing a response, because API responses carry personal data.
pub fn without_caching(app: Router) -> Router {
    app.layer(map_response(no_store))
}

async fn security_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'self'; frame-ancestors 'none'"),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response
}

async fn no_store(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

/// Fires once the service manager asks the process to terminate or someone presses Ctrl-C.
pub fn termination_signal() -> oneshot::Receiver<()> {
    let (sender, receiver) = oneshot::channel();
    tokio::spawn(async move {
        wait_for_termination().await;
        let _ = sender.send(());
    });
    receiver
}

async fn wait_for_termination() {
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("the process must be able to listen for SIGTERM");
    tokio::select! {
        _ = terminate.recv() => {}
        _ = tokio::signal::ctrl_c() => {}
    }
}

#[cfg(test)]
mod integration {
    use std::time::Duration;

    use axum::routing::get;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;

    #[tokio::test]
    async fn stop_during_a_request() {
        let app = Router::new().route(
            "/slow",
            get(|| async {
                tokio::time::sleep(Duration::from_millis(300)).await;
                "done"
            }),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (stop, stopped) = oneshot::channel();
        let server = tokio::spawn(serve_until(listener, app, stopped));

        let mut connection = tokio::net::TcpStream::connect(address).await.unwrap();
        connection
            .write_all(b"GET /slow HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;
        stop.send(()).unwrap();

        let mut response = String::new();
        connection.read_to_string(&mut response).await.unwrap();
        assert!(response.starts_with("HTTP/1.1 200"), "{response}");
        assert!(response.ends_with("done"), "{response}");

        tokio::time::timeout(Duration::from_secs(2), server)
            .await
            .expect("server must stop once the request finished")
            .unwrap()
            .unwrap();
        assert!(
            tokio::net::TcpStream::connect(address).await.is_err(),
            "server must refuse new connections after stopping"
        );
    }
}
