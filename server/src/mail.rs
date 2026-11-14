use std::time::Duration;

use lettre::message::{Mailbox, MultiPart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use sqlx::{PgConnection, PgPool};

use crate::configuration::{Encryption, Smtp};

const BATCH_SIZE: i64 = 10;

pub type Transport = AsyncSmtpTransport<Tokio1Executor>;

pub struct Mail {
    pub recipient: String,
    pub subject: String,
    pub text: String,
    pub html: String,
}

pub struct Delivery {
    pub transport: Transport,
    pub sender: Mailbox,
    pub max_attempts: i32,
}

#[derive(sqlx::FromRow)]
struct PendingMail {
    id: i64,
    recipient: String,
    subject: String,
    text_body: String,
    html_body: String,
    attempts: i32,
}

pub fn transport(smtp: &Smtp) -> Result<Transport, lettre::transport::smtp::Error> {
    let builder = match smtp.encryption {
        Encryption::StartTls => Transport::starttls_relay(&smtp.host)?,
        Encryption::Tls => Transport::relay(&smtp.host)?,
        Encryption::None => Transport::builder_dangerous(&smtp.host),
    }
    .port(smtp.port);
    let builder = match (&smtp.username, &smtp.password) {
        (Some(username), Some(password)) => {
            builder.credentials(Credentials::new(username.clone(), password.clone()))
        }
        _ => builder,
    };
    Ok(builder.build())
}

pub async fn enqueue(connection: &mut PgConnection, mail: &Mail) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO mail_outbox (recipient, subject, text_body, html_body) VALUES ($1, $2, $3, $4)")
        .bind(&mail.recipient)
        .bind(&mail.subject)
        .bind(&mail.text)
        .bind(&mail.html)
        .execute(connection)
        .await
        .map(|_| ())
}

pub async fn run(pool: PgPool, delivery: Delivery, interval: Duration) {
    loop {
        if let Err(error) = deliver_due(&pool, &delivery).await {
            tracing::error!(error = %error, "mail outbox could not be processed");
        }
        tokio::time::sleep(interval).await;
    }
}

async fn deliver_due(pool: &PgPool, delivery: &Delivery) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let pending: Vec<PendingMail> = sqlx::query_as(
        "SELECT id, recipient, subject, text_body, html_body, attempts FROM mail_outbox \
          WHERE sent_at IS NULL AND failed_at IS NULL AND next_attempt_at <= now() \
          ORDER BY id LIMIT $1 FOR UPDATE SKIP LOCKED",
    )
    .bind(BATCH_SIZE)
    .fetch_all(&mut *transaction)
    .await?;
    for mail in pending {
        if let Err(failure) = send(delivery, &mail).await {
            tracing::warn!(
                outbox_id = mail.id,
                attempt = mail.attempts + 1,
                %failure,
                "mail delivery failed"
            );
            schedule_retry(&mut transaction, &mail, delivery.max_attempts).await?;
        } else {
            mark_sent(&mut transaction, mail.id).await?;
        }
    }
    transaction.commit().await
}

/// Sends one mail. A failure names only its kind, because messages from mail servers often repeat the recipient.
async fn send(delivery: &Delivery, mail: &PendingMail) -> Result<(), String> {
    let recipient = mail
        .recipient
        .parse::<Mailbox>()
        .map_err(|_| "invalid recipient".to_owned())?;
    let message = Message::builder()
        .from(delivery.sender.clone())
        .to(recipient)
        .subject(&mail.subject)
        .multipart(MultiPart::alternative_plain_html(
            mail.text_body.clone(),
            mail.html_body.clone(),
        ))
        .map_err(|_| "invalid message".to_owned())?;
    delivery
        .transport
        .send(message)
        .await
        .map(|_| ())
        .map_err(|error| failure_kind(&error))
}

/// Describes a failed delivery without data from the mail server. Client errors are fixed texts of lettre and therefore safe to log.
fn failure_kind(error: &lettre::transport::smtp::Error) -> String {
    if error.is_client() {
        error.to_string()
    } else if error.is_permanent() {
        "permanent rejection".to_owned()
    } else if error.is_transient() {
        "temporary rejection".to_owned()
    } else if error.is_tls() {
        "encryption".to_owned()
    } else if error.is_timeout() {
        "timeout".to_owned()
    } else {
        "connection".to_owned()
    }
}

async fn mark_sent(connection: &mut PgConnection, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE mail_outbox SET sent_at = now(), attempts = attempts + 1 WHERE id = $1")
        .bind(id)
        .execute(connection)
        .await
        .map(|_| ())
}

async fn schedule_retry(
    connection: &mut PgConnection,
    mail: &PendingMail,
    max_attempts: i32,
) -> Result<(), sqlx::Error> {
    let attempts = mail.attempts + 1;
    sqlx::query(
        "UPDATE mail_outbox SET attempts = $2, \
          next_attempt_at = now() + make_interval(secs => LEAST(10 * power(2, $2 - 1), 3600)), \
          failed_at = CASE WHEN $2 >= $3 THEN now() END \
          WHERE id = $1",
    )
    .bind(mail.id)
    .bind(attempts)
    .bind(max_attempts)
    .execute(connection)
    .await
    .map(|_| ())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;
    use crate::database::testing::fresh_database;

    static NEXT: AtomicU32 = AtomicU32::new(0);

    fn env_port(name: &str, default: u16) -> u16 {
        std::env::var(name)
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(default)
    }

    fn smtp(port: u16, encryption: Encryption) -> Smtp {
        Smtp {
            host: "127.0.0.1".to_owned(),
            port,
            encryption,
            username: None,
            password: None,
        }
    }

    fn delivery(port: u16) -> Delivery {
        Delivery {
            transport: transport(&smtp(port, Encryption::None)).unwrap(),
            sender: "Adunata <adunata@example.org>".parse().unwrap(),
            max_attempts: 3,
        }
    }

    fn unique_token() -> String {
        format!(
            "spike{}x{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )
    }

    async fn mailpit_count(token: &str) -> u64 {
        let port = env_port("ADUNATA_TEST_MAILPIT_HTTP_PORT", 8025);
        let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .unwrap();
        let request =
            format!("GET /api/v1/search?query={token} HTTP/1.0\r\nHost: localhost\r\n\r\n");
        stream.write_all(request.as_bytes()).await.unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).await.unwrap();
        let body = response.split("\r\n\r\n").nth(1).unwrap_or_default();
        let json: serde_json::Value = serde_json::from_str(body).unwrap();
        json["messages_count"].as_u64().unwrap()
    }

    async fn enqueue_test_mail(pool: &PgPool, token: &str) {
        let mut connection = pool.acquire().await.unwrap();
        let mail = Mail {
            recipient: "Organizer <organizer@example.org>".to_owned(),
            subject: format!("Response {token}"),
            text: format!("Plain {token}"),
            html: format!("<p>Rich {token}</p>"),
        };
        enqueue(&mut connection, &mail).await.unwrap();
    }

    async fn state(pool: &PgPool) -> (i32, bool, bool) {
        sqlx::query_as(
            "SELECT attempts, sent_at IS NOT NULL, failed_at IS NOT NULL FROM mail_outbox",
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn delivers_after_the_mail_server_comes_back() {
        let pool = fresh_database().await;
        let token = unique_token();
        enqueue_test_mail(&pool, &token).await;

        deliver_due(&pool, &delivery(1)).await.unwrap();
        assert_eq!(state(&pool).await, (1, false, false));
        assert_eq!(mailpit_count(&token).await, 0);

        sqlx::query("UPDATE mail_outbox SET next_attempt_at = now()")
            .execute(&pool)
            .await
            .unwrap();
        deliver_due(
            &pool,
            &delivery(env_port("ADUNATA_TEST_MAILPIT_SMTP_PORT", 1025)),
        )
        .await
        .unwrap();
        assert_eq!(state(&pool).await, (2, true, false));
        assert_eq!(mailpit_count(&token).await, 1);
    }

    #[derive(Clone, Default)]
    struct CapturedLogs(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

    impl std::io::Write for CapturedLogs {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// Captures the logs of every test in this binary. A thread-local subscriber would miss events whose callsites other test threads registered first.
    fn captured_logs() -> &'static CapturedLogs {
        static LOGS: std::sync::OnceLock<CapturedLogs> = std::sync::OnceLock::new();
        LOGS.get_or_init(|| {
            let logs = CapturedLogs::default();
            let writer = logs.clone();
            let subscriber = tracing_subscriber::fmt()
                .json()
                .with_writer(move || writer.clone())
                .finish();
            tracing::subscriber::set_global_default(subscriber)
                .expect("no other test may set a global subscriber");
            logs
        })
    }

    #[tokio::test]
    async fn log_a_failed_mail_delivery() {
        let logs = captured_logs();
        let pool = fresh_database().await;
        let token = unique_token();
        enqueue_test_mail(&pool, &token).await;
        let outbox_id: i64 = sqlx::query_scalar("SELECT id FROM mail_outbox")
            .fetch_one(&pool)
            .await
            .unwrap();
        deliver_due(&pool, &delivery(1)).await.unwrap();

        let output = String::from_utf8(logs.0.lock().unwrap().clone()).unwrap();
        let entries: Vec<serde_json::Value> = output
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .filter(|entry: &serde_json::Value| {
                entry["fields"]["message"] == "mail delivery failed"
            })
            .collect();
        assert!(!entries.is_empty(), "a failed delivery must be logged");
        assert!(
            entries
                .iter()
                .all(|entry| entry["fields"]["failure"].is_string())
        );
        assert!(
            entries
                .iter()
                .any(|entry| entry["fields"]["outbox_id"] == outbox_id),
            "{output}"
        );
        assert!(!output.contains("organizer@example.org"), "{output}");
        assert!(!output.contains(&token), "{output}");
    }

    async fn mailpit_message(token: &str) -> serde_json::Value {
        let port = env_port("ADUNATA_TEST_MAILPIT_HTTP_PORT", 8025);
        let search = mailpit_get(port, &format!("/api/v1/search?query={token}")).await;
        let id = search["messages"][0]["ID"].as_str().unwrap().to_owned();
        mailpit_get(port, &format!("/api/v1/message/{id}")).await
    }

    async fn mailpit_get(port: u16, path: &str) -> serde_json::Value {
        let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .unwrap();
        let request = format!("GET {path} HTTP/1.0\r\nHost: localhost\r\n\r\n");
        stream.write_all(request.as_bytes()).await.unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).await.unwrap();
        serde_json::from_str(response.split("\r\n\r\n").nth(1).unwrap_or_default()).unwrap()
    }

    #[tokio::test]
    async fn send_through_the_configured_server() {
        let pool = fresh_database().await;
        let token = unique_token();
        enqueue_test_mail(&pool, &token).await;
        deliver_due(
            &pool,
            &delivery(env_port("ADUNATA_TEST_MAILPIT_SMTP_PORT", 1025)),
        )
        .await
        .unwrap();
        let message = mailpit_message(&token).await;
        assert_eq!(message["From"]["Address"], "adunata@example.org");
        assert_eq!(message["To"][0]["Address"], "organizer@example.org");
    }

    #[tokio::test]
    async fn server_offers_no_encryption() {
        let pool = fresh_database().await;
        let token = unique_token();
        enqueue_test_mail(&pool, &token).await;
        let port = env_port("ADUNATA_TEST_MAILPIT_SMTP_PORT", 1025);
        let encrypted = Delivery {
            transport: transport(&smtp(port, Encryption::StartTls)).unwrap(),
            ..delivery(port)
        };
        let logs = captured_logs();
        deliver_due(&pool, &encrypted).await.unwrap();
        assert_eq!(state(&pool).await, (1, false, false));
        assert_eq!(mailpit_count(&token).await, 0);
        let output = String::from_utf8(logs.0.lock().unwrap().clone()).unwrap();
        let reasons: Vec<String> = output
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .filter(|entry| entry["fields"]["message"] == "mail delivery failed")
            .filter_map(|entry| entry["fields"]["failure"].as_str().map(str::to_owned))
            .collect();
        assert!(
            reasons.iter().any(|reason| reason.contains("STARTTLS")),
            "the log must name the missing encryption: {reasons:?}"
        );
    }

    #[tokio::test]
    async fn gives_up_after_the_last_attempt() {
        let pool = fresh_database().await;
        enqueue_test_mail(&pool, &unique_token()).await;
        for _ in 0..3 {
            sqlx::query("UPDATE mail_outbox SET next_attempt_at = now()")
                .execute(&pool)
                .await
                .unwrap();
            deliver_due(&pool, &delivery(1)).await.unwrap();
        }
        assert_eq!(state(&pool).await, (3, false, true));
    }
}
