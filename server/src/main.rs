mod access;
mod api;
mod client;
mod configuration;
mod database;
mod gatherings;
mod health;
mod invitations;
mod legal;
mod localization;
mod mail;
mod members;
mod responses;
mod secret;
mod serving;
#[cfg(test)]
mod test_logs;
mod validation;

use std::time::Duration;

use clap::{CommandFactory, Parser, Subcommand};
use configuration::{Configuration, FirstOrganizer, PublicUrl};

/// Adunata organizes the gatherings of choirs and instrumental ensembles.
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Configuration file in TOML. A setting in the environment wins over the same setting in the file.
    #[arg(long, global = true, env = "ADNT_CONFIG_FILE", value_name = "PATH")]
    config: Option<String>,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Serve the API and the client until the service manager stops the process.
    Serve {
        /// Start in development mode for a debug run on localhost
        ///
        /// Creates a default organizer if the instance has none and writes sign-in links to the log. Refuses to start unless the public URL is on localhost.
        #[arg(long)]
        dev: bool,
    },
    /// Make a member an active organizer
    ///
    /// Creates the member if no member has the mail address. Meant for an operator who lost every organizer mailbox. Reads the same settings as `serve`.
    GrantOrganizer {
        /// Mail address of the member
        #[arg(value_parser = mail_address)]
        mail: String,
        /// First name, used only when the member does not exist yet
        #[arg(value_parser = name)]
        first_name: String,
        /// Last name, used only when the member does not exist yet
        #[arg(value_parser = name)]
        last_name: String,
    },
    #[command(about = "Print the OpenAPI document of the API")]
    Openapi,
}

fn mail_address(value: &str) -> Result<String, String> {
    value
        .parse::<lettre::Address>()
        .map(|_| value.to_owned())
        .map_err(|_| "expected a mail address such as organizer@example.org".to_owned())
}

fn name(value: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err("expected a name that is not blank".to_owned())
    } else {
        Ok(trimmed.to_owned())
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let Some(command) = cli.command else {
        eprint!("{}", Cli::command().render_help());
        std::process::exit(2);
    };
    tracing_subscriber::fmt().json().init();
    let file = cli.config.filter(|path| !path.is_empty());
    match command {
        Command::Serve { dev } => serve(configuration(file.as_deref()), dev).await,
        Command::GrantOrganizer {
            mail,
            first_name,
            last_name,
        } => {
            grant_organizer(
                configuration(file.as_deref()),
                &mail,
                &first_name,
                &last_name,
            )
            .await;
        }
        Command::Openapi => {
            let document = api::document()
                .to_pretty_json()
                .expect("OpenAPI document must serialize");
            println!("{document}");
        }
    }
}

fn configuration(file: Option<&str>) -> Configuration {
    Configuration::from_environment_and_file(file).unwrap_or_else(|problems| {
        for problem in problems {
            tracing::error!(%problem, "configuration is invalid");
        }
        std::process::exit(1);
    })
}

async fn connect(configuration: &Configuration) -> sqlx::PgPool {
    database::connect(configuration.database.clone())
        .await
        .unwrap_or_else(|error| {
            tracing::error!(%error, "database is not ready");
            std::process::exit(1);
        })
}

/// Makes the member with a mail address an active organizer, for an operator who lost every organizer mailbox.
async fn grant_organizer(
    configuration: Configuration,
    mail: &str,
    first_name: &str,
    last_name: &str,
) {
    let pool = connect(&configuration).await;
    let mut connection = pool.acquire().await.unwrap_or_else(|error| {
        tracing::error!(%error, "database is not ready");
        std::process::exit(1);
    });
    match members::grant_organizer(
        &mut connection,
        mail,
        first_name,
        last_name,
        &configuration.default_language,
    )
    .await
    {
        Ok(member) => println!(
            "{} is an active organizer and can request a sign-in link",
            member.mail
        ),
        Err(error) => {
            tracing::error!(%error, "organizer role could not be granted");
            std::process::exit(1);
        }
    }
}

async fn serve(configuration: Configuration, development_mode: bool) {
    if development_mode && !runs_only_on_this_machine(&configuration) {
        tracing::error!(
            "development mode needs ADNT_HTTP_PUBLIC_URL on localhost and ADNT_HTTP_LISTEN_ADDRESS on a loopback address, because it writes sign-in links to the log"
        );
        std::process::exit(1);
    }
    let pool = connect(&configuration).await;
    let delivery = mail::Delivery {
        transport: mail::transport(&configuration.smtp).unwrap_or_else(|error| {
            tracing::error!(%error, "mail server settings are unusable");
            std::process::exit(1);
        }),
        sender: configuration.mail_sender,
        max_attempts: configuration.mail_max_attempts,
    };
    tokio::spawn(mail::run(pool.clone(), delivery, Duration::from_secs(2)));
    tokio::spawn(responses::run_retention(
        pool.clone(),
        configuration.retention,
        configuration.time_zone.clone(),
        Duration::from_hours(1),
    ));
    let listener = tokio::net::TcpListener::bind(configuration.listen_address)
        .await
        .expect("listen address must be free");
    prepare_organizers(
        &pool,
        configuration.first_organizer.clone(),
        &configuration.default_language,
        &configuration.public_url,
        development_mode,
    )
    .await
    .unwrap_or_else(|error| {
        tracing::error!(%error, "organizers could not be prepared");
        std::process::exit(1);
    });
    let state = api::AppState {
        pool,
        public_url: configuration.public_url,
        legal_links: configuration.legal_links,
        trusted_proxies: configuration.trusted_proxies,
        sign_in_limiter: access::SignInLimiter::default(),
        time_zone: configuration.time_zone,
        development_mode,
    };
    tracing::info!(address = %listener.local_addr().expect("listener must have an address"), "accepting requests");
    serving::serve_until(listener, app(state), serving::termination_signal())
        .await
        .expect("server must keep running until it is asked to stop");
}

/// Whether only this machine can reach the instance, which development mode needs because its log grants access.
fn runs_only_on_this_machine(configuration: &Configuration) -> bool {
    configuration.public_url.localhost_port().is_some()
        && configuration.listen_address.ip().is_loopback()
}

/// Creates the first organizer if the instance has none and reports the outcome.
async fn create_first_organizer(
    pool: &sqlx::PgPool,
    first_organizer: Option<&FirstOrganizer>,
    language: &str,
) -> Result<(), sqlx::Error> {
    match access::ensure_first_organizer(pool, first_organizer, language).await? {
        access::FirstOrganizerOutcome::Created => tracing::info!("first organizer created"),
        access::FirstOrganizerOutcome::AlreadyPresent => {}
        access::FirstOrganizerOutcome::NobodyCanSignIn => tracing::warn!(
            "the instance has no active organizer, so nobody can sign in. Set ADNT_BOOTSTRAP_ORGANIZER_MAIL with both names or run adunata grant-organizer"
        ),
    }
    Ok(())
}

/// Creates the first organizer if the instance has none. In development mode the default organizer stands in for a first organizer that the settings do not name, and a sign-in link for the oldest active organizer goes to the log and is returned.
async fn prepare_organizers(
    pool: &sqlx::PgPool,
    first_organizer: Option<FirstOrganizer>,
    language: &str,
    public_url: &PublicUrl,
    development_mode: bool,
) -> Result<Option<String>, sqlx::Error> {
    if !development_mode {
        create_first_organizer(pool, first_organizer.as_ref(), language).await?;
        return Ok(None);
    }
    tracing::warn!("development mode is on, so the log contains sign-in links");
    let first_organizer = first_organizer.unwrap_or_else(access::development_organizer);
    create_first_organizer(pool, Some(&first_organizer), language).await?;
    let link = access::development_sign_in_link(pool, public_url).await?;
    tracing::info!(link, "sign-in link for development");
    Ok(Some(link))
}

fn app(state: api::AppState) -> axum::Router {
    serving::with_security_headers(api::router(state).merge(client::router()))
}

#[cfg(test)]
mod integration {
    use axum::body::Body;
    use axum::http::{HeaderMap, Request, header};
    use tower::ServiceExt;

    use super::*;

    async fn headers(path: &str) -> HeaderMap {
        let pool = sqlx::PgPool::connect_lazy("postgres://localhost").unwrap();
        app(api::testing::state(pool))
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap()
            .headers()
            .clone()
    }

    fn assert_security_headers(headers: &HeaderMap) {
        assert_eq!(headers[header::REFERRER_POLICY], "no-referrer");
        assert_eq!(
            headers[header::CONTENT_SECURITY_POLICY],
            "default-src 'self'; frame-ancestors 'none'"
        );
        assert_eq!(headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
    }

    #[tokio::test]
    async fn entry_page_carries_security_headers() {
        let headers = headers("/").await;
        assert_security_headers(&headers);
        assert!(!headers.contains_key(header::CACHE_CONTROL));
    }

    mod development {
        use axum::body::Body;
        use axum::http::{Request, StatusCode, header};
        use sqlx::PgPool;
        use tower::ServiceExt;

        use super::super::*;
        use crate::database::testing::fresh_database;
        use crate::test_logs::captured_logs;

        fn localhost() -> PublicUrl {
            PublicUrl::parse("http://localhost:8080").unwrap()
        }

        async fn organizers(pool: &PgPool) -> Vec<String> {
            sqlx::query_scalar("SELECT mail FROM members WHERE active AND organizer ORDER BY id")
                .fetch_all(pool)
                .await
                .unwrap()
        }

        async fn count(pool: &PgPool, table: &str) -> i64 {
            sqlx::query_scalar(sqlx::AssertSqlSafe(format!("SELECT count(*) FROM {table}")))
                .fetch_one(pool)
                .await
                .unwrap()
        }

        #[tokio::test]
        async fn start_on_an_empty_database() {
            let logs = captured_logs();
            let database = fresh_database().await;
            let pool = database.pool();

            let link = prepare_organizers(&pool, None, "en", &localhost(), true)
                .await
                .unwrap()
                .expect("development mode must create a link");
            assert_eq!(organizers(&pool).await, ["organizer@example.org"]);
            let secret = link.split_once("/sign-in#").unwrap().1;
            assert!(logs.text().contains(secret), "{}", logs.text());
            assert!(
                logs.text().contains("development mode is on"),
                "{}",
                logs.text()
            );

            let request = Request::post("/api/sessions")
                .header(header::AUTHORIZATION, format!("Bearer {secret}"))
                .body(Body::empty())
                .unwrap();
            let response = app(api::testing::state(pool.clone()))
                .oneshot(request)
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::CREATED);
            assert_eq!(count(&pool, "mail_outbox").await, 0);
        }

        #[tokio::test]
        async fn bootstrap_settings_win() {
            let database = fresh_database().await;
            let pool = database.pool();
            let grace = FirstOrganizer {
                mail: "grace@example.org".to_owned(),
                first_name: "Grace".to_owned(),
                last_name: "Hopper".to_owned(),
            };
            prepare_organizers(&pool, Some(grace), "en", &localhost(), true)
                .await
                .unwrap();
            assert_eq!(organizers(&pool).await, ["grace@example.org"]);
        }

        #[tokio::test]
        async fn instance_already_has_an_organizer() {
            let database = fresh_database().await;
            let pool = database.pool();
            sqlx::query("INSERT INTO members (first_name, last_name, mail, language, organizer) VALUES ('Clara', 'Schumann', 'clara@example.org', 'en', true)")
                .execute(&pool)
                .await
                .unwrap();
            prepare_organizers(&pool, None, "en", &localhost(), true)
                .await
                .unwrap();
            assert_eq!(organizers(&pool).await, ["clara@example.org"]);
            assert_eq!(count(&pool, "members").await, 1);
        }

        #[tokio::test]
        async fn start_without_development_mode_creates_no_link() {
            let database = fresh_database().await;
            let pool = database.pool();
            let link = prepare_organizers(&pool, None, "en", &localhost(), false)
                .await
                .unwrap();
            assert_eq!(link, None);
            assert!(organizers(&pool).await.is_empty());
            assert_eq!(count(&pool, "sign_in_links").await, 0);
        }
    }

    #[tokio::test]
    async fn api_response_is_never_cached() {
        let headers = headers("/api/catalogs/de").await;
        assert_security_headers(&headers);
        assert_eq!(headers[header::CACHE_CONTROL], "no-store");
    }
}
