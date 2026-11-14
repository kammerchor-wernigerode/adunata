use std::time::Duration;

use lettre::message::{Mailbox, MultiPart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use sqlx::{PgConnection, PgPool};

use crate::configuration::Smtp;

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

pub fn unencrypted_transport(smtp: &Smtp) -> Transport {
    let builder = Transport::builder_dangerous(&smtp.host).port(smtp.port);
    match (&smtp.username, &smtp.password) {
        (Some(username), Some(password)) => builder
            .credentials(Credentials::new(username.clone(), password.clone()))
            .build(),
        _ => builder.build(),
    }
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
            eprintln!("mail delivery failed: {error}");
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
        if send(delivery, &mail).await {
            mark_sent(&mut transaction, mail.id).await?;
        } else {
            schedule_retry(&mut transaction, &mail, delivery.max_attempts).await?;
        }
    }
    transaction.commit().await
}

async fn send(delivery: &Delivery, mail: &PendingMail) -> bool {
    let Ok(recipient) = mail.recipient.parse::<Mailbox>() else {
        return false;
    };
    let message = Message::builder()
        .from(delivery.sender.clone())
        .to(recipient)
        .subject(&mail.subject)
        .multipart(MultiPart::alternative_plain_html(
            mail.text_body.clone(),
            mail.html_body.clone(),
        ));
    match message {
        Ok(message) => delivery.transport.send(message).await.is_ok(),
        Err(_) => false,
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

    fn delivery(port: u16) -> Delivery {
        Delivery {
            transport: unencrypted_transport(&Smtp {
                host: "127.0.0.1".to_owned(),
                port,
                username: None,
                password: None,
            }),
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
