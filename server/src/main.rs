mod access;
mod api;
mod client;
mod configuration;
mod database;
mod gatherings;
mod health;
mod legal;
mod localization;
mod mail;
mod members;
mod secret;
mod serving;
mod spike;
mod validation;

use std::time::Duration;

use clap::{Parser, Subcommand};
use configuration::Configuration;

/// Adunata organizes the gatherings of choirs and instrumental ensembles.
#[derive(Parser)]
#[command(version, about, arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Serve the API and the client until the service manager stops the process.
    Serve,
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
    match Cli::parse().command {
        Command::Serve => serve().await,
        Command::GrantOrganizer {
            mail,
            first_name,
            last_name,
        } => grant_organizer(&mail, &first_name, &last_name).await,
        Command::Openapi => {
            let document = api::document()
                .to_pretty_json()
                .expect("OpenAPI document must serialize");
            println!("{document}");
        }
    }
}

fn configuration() -> Configuration {
    Configuration::from_environment().unwrap_or_else(|problems| {
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
async fn grant_organizer(mail: &str, first_name: &str, last_name: &str) {
    tracing_subscriber::fmt().json().init();
    let configuration = configuration();
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

async fn serve() {
    tracing_subscriber::fmt().json().init();
    let configuration = configuration();
    let pool = connect(&configuration).await;
    match access::ensure_first_organizer(
        &pool,
        configuration.first_organizer.as_ref(),
        &configuration.default_language,
    )
    .await
    {
        Ok(access::FirstOrganizerOutcome::Created) => tracing::info!("first organizer created"),
        Ok(access::FirstOrganizerOutcome::AlreadyPresent) => {}
        Ok(access::FirstOrganizerOutcome::NobodyCanSignIn) => tracing::warn!(
            "the instance has no active organizer, so nobody can sign in. Set ADUNATA_FIRST_ORGANIZER_MAIL with both names or run adunata grant-organizer"
        ),
        Err(error) => {
            tracing::error!(%error, "first organizer could not be created");
            std::process::exit(1);
        }
    }
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
        legal_links: configuration.legal_links,
        trusted_proxies: configuration.trusted_proxies,
        sign_in_limiter: access::SignInLimiter::default(),
        time_zone: configuration.time_zone,
    };
    tracing::info!(address = %listener.local_addr().expect("listener must have an address"), "accepting requests");
    serving::serve_until(listener, app(state), serving::termination_signal())
        .await
        .expect("server must keep running until it is asked to stop");
}

fn app(state: api::AppState) -> axum::Router {
    serving::with_security_headers(api::router(state).merge(client::router()))
}

#[cfg(test)]
mod tests {
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

    #[tokio::test]
    async fn api_response_is_never_cached() {
        let headers = headers("/api/catalogs/de").await;
        assert_security_headers(&headers);
        assert_eq!(headers[header::CACHE_CONTROL], "no-store");
    }
}
