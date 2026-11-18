use std::time::Duration;

use lettre::message::{Mailbox, MultiPart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use sqlx::{PgConnection, PgPool};

use crate::configuration::{Encryption, Smtp};

const BATCH_SIZE: usize = 10;

pub type Transport = AsyncSmtpTransport<Tokio1Executor>;

pub struct Mail {
    /// The bare mail address. Delivery adds the display name of the member.
    pub recipient: String,
    pub member_id: Option<i64>,
    pub invitation_id: Option<i64>,
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
    recipient_name: Option<String>,
    subject: String,
    text_body: String,
    html_body: String,
    attempts: i32,
}

/// Escapes a text for the HTML part of a mail. Every value that people entered passes through it.
pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
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
    sqlx::query("INSERT INTO mail_outbox (recipient, member_id, invitation_id, subject, text_body, html_body) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(&mail.recipient)
        .bind(mail.member_id)
        .bind(mail.invitation_id)
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

/// Delivers due mail, each in its own transaction, so a mail the server accepted stays marked as sent whatever happens to the next one.
pub async fn deliver_due(pool: &PgPool, delivery: &Delivery) -> Result<(), sqlx::Error> {
    for _ in 0..BATCH_SIZE {
        let mut transaction = pool.begin().await?;
        let pending: Option<PendingMail> = sqlx::query_as(
            "SELECT outbox.id, COALESCE(members.mail, outbox.recipient) AS recipient, members.first_name || ' ' || members.last_name AS recipient_name, outbox.subject, outbox.text_body, outbox.html_body, outbox.attempts FROM mail_outbox outbox LEFT JOIN members ON members.id = outbox.member_id WHERE outbox.sent_at IS NULL AND outbox.failed_at IS NULL AND outbox.next_attempt_at <= now() ORDER BY outbox.id LIMIT 1 FOR UPDATE OF outbox SKIP LOCKED",
        )
        .fetch_optional(&mut *transaction)
        .await?;
        let Some(mail) = pending else {
            return transaction.commit().await;
        };
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
        transaction.commit().await?;
    }
    Ok(())
}

/// Sends one mail. A failure names only its kind, because messages from mail servers often repeat the recipient.
async fn send(delivery: &Delivery, mail: &PendingMail) -> Result<(), String> {
    let recipient = Mailbox::new(
        mail.recipient_name.clone(),
        mail.recipient
            .parse()
            .map_err(|_| "invalid recipient".to_owned())?,
    );
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
    } else if error.is_tls() || failed_handshake(error) {
        "encryption".to_owned()
    } else if error.is_timeout() {
        "timeout".to_owned()
    } else {
        "connection".to_owned()
    }
}

/// lettre reports a failed TLS handshake, such as a rejected certificate, as a connection error. tokio-rustls hands the error of rustls on inside an I/O error of kind `InvalidData`. An I/O error of that kind without an inner error, such as invalid UTF-8 in a reply of the mail server, is no handshake.
fn failed_handshake(error: &(dyn std::error::Error + 'static)) -> bool {
    std::iter::successors(error.source(), |cause| cause.source()).any(|cause| {
        cause.downcast_ref::<std::io::Error>().is_some_and(|io| {
            io.kind() == std::io::ErrorKind::InvalidData && io.get_ref().is_some()
        })
    })
}

async fn mark_sent(connection: &mut PgConnection, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE mail_outbox SET sent_at = now(), attempts = attempts + 1, recipient = NULL, subject = NULL, text_body = NULL, html_body = NULL WHERE id = $1")
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
        "UPDATE mail_outbox SET attempts = $2, next_attempt_at = now() + make_interval(secs => LEAST(10 * power(2, $2 - 1), 3600)), failed_at = CASE WHEN $2 >= $3 THEN now() END, recipient = CASE WHEN $2 < $3 THEN recipient END, subject = CASE WHEN $2 < $3 THEN subject END, text_body = CASE WHEN $2 < $3 THEN text_body END, html_body = CASE WHEN $2 < $3 THEN html_body END WHERE id = $1",
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
    use super::*;

    #[test]
    fn escape_every_character_with_meaning_in_html() {
        assert_eq!(
            escape_html(r#"<a href="x" title='y'>&</a>"#),
            "&lt;a href=&quot;x&quot; title=&#39;y&#39;&gt;&amp;&lt;/a&gt;"
        );
    }

    #[derive(Debug)]
    struct Wrapped(std::io::Error);

    impl std::fmt::Display for Wrapped {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("wrapped")
        }
    }

    impl std::error::Error for Wrapped {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(&self.0)
        }
    }

    #[test]
    fn invalid_data_below_the_error_is_a_failed_handshake() {
        let rejected = Wrapped(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid peer certificate",
        ));
        assert!(failed_handshake(&rejected));
    }

    #[test]
    fn invalid_data_without_inner_error_is_no_failed_handshake() {
        let garbled = Wrapped(std::io::Error::from(std::io::ErrorKind::InvalidData));
        assert!(!failed_handshake(&garbled));
    }

    #[test]
    fn refused_connection_is_no_failed_handshake() {
        let refused = Wrapped(std::io::Error::from(std::io::ErrorKind::ConnectionRefused));
        assert!(!failed_handshake(&refused));
    }

    mod integration {
        use super::*;
        use crate::configuration::test_setting;
        use crate::database::testing::fresh_database;
        use crate::test_logs::captured_logs;
        use std::sync::atomic::{AtomicU32, Ordering};
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        static NEXT: AtomicU32 = AtomicU32::new(0);

        fn setting_port(name: &str) -> u16 {
            test_setting(name)
                .and_then(|value| value.parse().ok())
                .unwrap_or_else(|| panic!("{name} must name the port of the test Mailpit"))
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
                "mail{}x{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            )
        }

        async fn mailpit_count(token: &str) -> u64 {
            let port = setting_port("MAILPIT_UI_PORT");
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
                recipient: "organizer@example.org".to_owned(),
                member_id: None,
                invitation_id: None,
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
        async fn mail_server_comes_back() {
            let database = fresh_database().await;
            let pool = database.pool();
            let token = unique_token();
            enqueue_test_mail(&pool, &token).await;

            deliver_due(&pool, &delivery(1)).await.unwrap();
            assert_eq!(state(&pool).await, (1, false, false));
            assert_eq!(mailpit_count(&token).await, 0);

            sqlx::query("UPDATE mail_outbox SET next_attempt_at = now()")
                .execute(&pool)
                .await
                .unwrap();
            deliver_due(&pool, &delivery(setting_port("ADNT_SMTP_PORT")))
                .await
                .unwrap();
            assert_eq!(state(&pool).await, (2, true, false));
            assert_eq!(mailpit_count(&token).await, 1);
        }

        #[tokio::test]
        async fn log_a_failed_mail_delivery() {
            let logs = captured_logs();
            let database = fresh_database().await;
            let pool = database.pool();
            let token = unique_token();
            enqueue_test_mail(&pool, &token).await;
            let outbox_id: i64 = sqlx::query_scalar("SELECT id FROM mail_outbox")
                .fetch_one(&pool)
                .await
                .unwrap();
            deliver_due(&pool, &delivery(1)).await.unwrap();

            let output = logs.text();
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
            let port = setting_port("MAILPIT_UI_PORT");
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
            let database = fresh_database().await;
            let pool = database.pool();
            let token = unique_token();
            enqueue_test_mail(&pool, &token).await;
            deliver_due(&pool, &delivery(setting_port("ADNT_SMTP_PORT")))
                .await
                .unwrap();
            let message = mailpit_message(&token).await;
            assert_eq!(message["From"]["Address"], "adunata@example.org");
            assert_eq!(message["To"][0]["Address"], "organizer@example.org");
        }

        #[tokio::test]
        async fn server_offers_no_encryption() {
            let database = fresh_database().await;
            let pool = database.pool();
            let token = unique_token();
            enqueue_test_mail(&pool, &token).await;
            let port = setting_port("ADNT_SMTP_PORT");
            let encrypted = Delivery {
                transport: transport(&smtp(port, Encryption::StartTls)).unwrap(),
                ..delivery(port)
            };
            let logs = captured_logs();
            deliver_due(&pool, &encrypted).await.unwrap();
            assert_eq!(state(&pool).await, (1, false, false));
            assert_eq!(mailpit_count(&token).await, 0);
            let output = logs.text();
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

        async fn retry_delay_seconds(pool: &PgPool) -> f64 {
            sqlx::query_scalar(
                "SELECT EXTRACT(EPOCH FROM next_attempt_at - now())::float8 FROM mail_outbox",
            )
            .fetch_one(pool)
            .await
            .unwrap()
        }

        async fn make_due(pool: &PgPool) {
            sqlx::query("UPDATE mail_outbox SET next_attempt_at = now()")
                .execute(pool)
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn retry_delays_grow() {
            let database = fresh_database().await;
            let pool = database.pool();
            enqueue_test_mail(&pool, &unique_token()).await;
            deliver_due(&pool, &delivery(1)).await.unwrap();
            let first = retry_delay_seconds(&pool).await;
            make_due(&pool).await;
            deliver_due(&pool, &delivery(1)).await.unwrap();
            let second = retry_delay_seconds(&pool).await;
            assert!((9.0..=10.0).contains(&first), "first delay was {first}");
            assert!((19.0..=20.0).contains(&second), "second delay was {second}");
        }

        #[tokio::test]
        async fn restart_with_pending_mail() {
            let database = fresh_database().await;
            let pool = database.pool();
            let token = unique_token();
            enqueue_test_mail(&pool, &token).await;
            let options = pool.connect_options().as_ref().clone();
            pool.close().await;

            let restarted = PgPool::connect_with(options).await.unwrap();
            deliver_due(&restarted, &delivery(setting_port("ADNT_SMTP_PORT")))
                .await
                .unwrap();
            assert_eq!(state(&restarted).await, (1, true, false));
            assert_eq!(mailpit_count(&token).await, 1);
        }

        async fn content_left(pool: &PgPool) -> bool {
            sqlx::query_scalar(
                    "SELECT recipient IS NOT NULL OR subject IS NOT NULL OR text_body IS NOT NULL OR html_body IS NOT NULL FROM mail_outbox",
                )
                .fetch_one(pool)
                .await
                .unwrap()
        }

        #[tokio::test]
        async fn accepted_mail_loses_its_content() {
            let database = fresh_database().await;
            let pool = database.pool();
            enqueue_test_mail(&pool, &unique_token()).await;
            assert!(content_left(&pool).await);
            deliver_due(&pool, &delivery(setting_port("ADNT_SMTP_PORT")))
                .await
                .unwrap();
            assert_eq!(state(&pool).await, (1, true, false));
            assert!(!content_left(&pool).await);
        }

        #[tokio::test]
        async fn failed_mail_loses_its_content() {
            let database = fresh_database().await;
            let pool = database.pool();
            enqueue_test_mail(&pool, &unique_token()).await;
            for attempt in 1..=3 {
                assert!(
                    content_left(&pool).await,
                    "content must stay for attempt {attempt}"
                );
                make_due(&pool).await;
                deliver_due(&pool, &delivery(1)).await.unwrap();
            }
            assert_eq!(state(&pool).await, (3, false, true));
            assert!(!content_left(&pool).await);
        }

        #[tokio::test]
        async fn accepted_mail() {
            let database = fresh_database().await;
            let pool = database.pool();
            let token = unique_token();
            enqueue_test_mail(&pool, &token).await;
            let mailpit = delivery(setting_port("ADNT_SMTP_PORT"));
            deliver_due(&pool, &mailpit).await.unwrap();
            make_due(&pool).await;
            deliver_due(&pool, &mailpit).await.unwrap();
            assert_eq!(state(&pool).await, (1, true, false));
            assert_eq!(mailpit_count(&token).await, 1);
        }

        #[tokio::test]
        async fn both_parts_hold_every_link() {
            let database = fresh_database().await;
            let pool = database.pool();
            let token = unique_token();
            let link = format!("https://adunata.example.org/invitations/{token}");
            let mut connection = pool.acquire().await.unwrap();
            let mail = Mail {
                recipient: "organizer@example.org".to_owned(),
                member_id: None,
                invitation_id: None,
                subject: format!("Links {token}"),
                text: format!("Answer here: {link}"),
                html: format!("<p>Answer <a href=\"{link}\">here</a>.</p>"),
            };
            enqueue(&mut connection, &mail).await.unwrap();
            drop(connection);
            deliver_due(&pool, &delivery(setting_port("ADNT_SMTP_PORT")))
                .await
                .unwrap();
            let message = mailpit_message(&token).await;
            assert!(message["Text"].as_str().unwrap().contains(&link));
            assert!(message["HTML"].as_str().unwrap().contains(&link));
        }

        #[tokio::test]
        async fn title_with_markup() {
            let database = fresh_database().await;
            let pool = database.pool();
            let token = unique_token();
            let title = format!("<b>Probe</b> {token}");
            let mut connection = pool.acquire().await.unwrap();
            let mail = Mail {
                recipient: "organizer@example.org".to_owned(),
                member_id: None,
                invitation_id: None,
                subject: title.clone(),
                text: title.clone(),
                html: format!("<p>{}</p>", escape_html(&title)),
            };
            enqueue(&mut connection, &mail).await.unwrap();
            drop(connection);
            deliver_due(&pool, &delivery(setting_port("ADNT_SMTP_PORT")))
                .await
                .unwrap();
            let html = mailpit_message(&token).await["HTML"]
                .as_str()
                .unwrap()
                .to_owned();
            assert!(html.contains("&lt;b&gt;Probe&lt;/b&gt;"), "{html}");
            assert!(!html.contains("<b>"), "{html}");
        }

        #[tokio::test]
        async fn mail_to_a_member_carries_the_name() {
            let database = fresh_database().await;
            let pool = database.pool();
            let token = unique_token();
            let member_id: i64 = sqlx::query_scalar(
                    "INSERT INTO members (first_name, last_name, mail, language) VALUES ('Ada', 'Lovelace', 'ada@example.org', 'en') RETURNING id",
                )
                .fetch_one(&pool)
                .await
                .unwrap();
            let mut connection = pool.acquire().await.unwrap();
            let mail = Mail {
                recipient: "ada@example.org".to_owned(),
                member_id: Some(member_id),
                invitation_id: None,
                subject: format!("Named {token}"),
                text: token.clone(),
                html: format!("<p>{token}</p>"),
            };
            enqueue(&mut connection, &mail).await.unwrap();
            drop(connection);
            deliver_due(&pool, &delivery(setting_port("ADNT_SMTP_PORT")))
                .await
                .unwrap();
            let message = mailpit_message(&token).await;
            assert_eq!(message["To"][0]["Name"], "Ada Lovelace");
            assert_eq!(message["To"][0]["Address"], "ada@example.org");
        }

        #[tokio::test]
        async fn mail_waiting_for_a_member_goes_to_the_current_address() {
            let database = fresh_database().await;
            let pool = database.pool();
            let token = unique_token();
            let member_id: i64 = sqlx::query_scalar(
                    "INSERT INTO members (first_name, last_name, mail, language) VALUES ('Ada', 'Lovelace', 'old@example.org', 'en') RETURNING id",
                )
                .fetch_one(&pool)
                .await
                .unwrap();
            let mut connection = pool.acquire().await.unwrap();
            let mail = Mail {
                recipient: "old@example.org".to_owned(),
                member_id: Some(member_id),
                invitation_id: None,
                subject: format!("Moved {token}"),
                text: token.clone(),
                html: format!("<p>{token}</p>"),
            };
            enqueue(&mut connection, &mail).await.unwrap();
            drop(connection);
            sqlx::query("UPDATE members SET mail = 'new@example.org'")
                .execute(&pool)
                .await
                .unwrap();
            deliver_due(&pool, &delivery(setting_port("ADNT_SMTP_PORT")))
                .await
                .unwrap();
            assert_eq!(
                mailpit_message(&token).await["To"][0]["Address"],
                "new@example.org"
            );
        }

        #[tokio::test]
        async fn action_fails_after_its_mail_was_added() {
            let database = fresh_database().await;
            let pool = database.pool();
            let mut transaction = pool.begin().await.unwrap();
            let mail = Mail {
                recipient: "organizer@example.org".to_owned(),
                member_id: None,
                invitation_id: None,
                subject: "Rolled back".to_owned(),
                text: "Rolled back".to_owned(),
                html: "<p>Rolled back</p>".to_owned(),
            };
            enqueue(&mut transaction, &mail).await.unwrap();
            transaction.rollback().await.unwrap();
            let mails: i64 = sqlx::query_scalar("SELECT count(*) FROM mail_outbox")
                .fetch_one(&pool)
                .await
                .unwrap();
            assert_eq!(mails, 0, "a rolled back action must take its mail with it");
        }

        #[tokio::test]
        async fn permanent_failure() {
            let database = fresh_database().await;
            let pool = database.pool();
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
}
