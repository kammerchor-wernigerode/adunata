use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

const UNREACHABLE_DATABASE: &str = "postgres://adunata@127.0.0.1:1/adunata";

static NEXT: AtomicU32 = AtomicU32::new(0);

/// A fresh empty directory, so that no settings file of the checkout reaches the binary. Dropping it removes the directory.
struct WorkingDirectory {
    path: PathBuf,
}

impl WorkingDirectory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "adunata-startup-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).expect("temporary directory must be creatable");
        Self { path }
    }
}

impl Drop for WorkingDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Runs the binary in a fresh empty directory with only the given arguments and variables. Returns the exit code, standard output and standard error.
fn run(arguments: &[&str], variables: &[(&str, &str)]) -> (Option<i32>, String, String) {
    run_in(&WorkingDirectory::new().path, arguments, variables)
}

/// The binary under test. Nix points `ADUNATA_TEST_BINARY` at the installed binary, because the path that Cargo compiles in belongs to another build directory there.
fn binary() -> String {
    std::env::var("ADUNATA_TEST_BINARY")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_adunata").to_owned())
}

fn run_in(
    directory: &Path,
    arguments: &[&str],
    variables: &[(&str, &str)],
) -> (Option<i32>, String, String) {
    let output = Command::new(binary())
        .current_dir(directory)
        .env_clear()
        .envs(variables.iter().copied())
        .args(arguments)
        .output()
        .expect("binary must run");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// Runs `serve` and returns the exit code and everything the binary printed.
fn serve(arguments: &[&str], variables: &[(&str, &str)]) -> (Option<i32>, String) {
    let arguments: Vec<&str> = arguments.iter().copied().chain(["serve"]).collect();
    let (code, output, error) = run(&arguments, variables);
    (code, format!("{output}{error}"))
}

fn configuration_file(name: &str, content: &str) -> String {
    let directory = std::env::temp_dir().join(format!("adunata-startup-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("temporary directory must be writable");
    let path = directory.join(name);
    std::fs::write(&path, content).expect("configuration file must be writable");
    path.to_string_lossy().into_owned()
}

/// Starts without the public URL, the mail server host and the time zone. The database URL points to a server that does not exist, so a start that touched the database would report it instead.
#[test]
fn several_settings_missing_stops_before_anything_opens() {
    let (code, log) = serve(
        &[],
        &[
            ("ADNT_DATABASE_URL", UNREACHABLE_DATABASE),
            ("ADNT_MAIL_SENDER", "Adunata <adunata@example.org>"),
        ],
    );
    assert_eq!(code, Some(1));
    assert!(log.contains("ADNT_HTTP_PUBLIC_URL"), "{log}");
    assert!(log.contains("ADNT_SMTP_HOST"), "{log}");
    assert!(log.contains("ADNT_LOCALIZATION_TIME_ZONE"), "{log}");
    assert!(!log.contains("database is not ready"), "{log}");
    assert!(!log.contains("accepting requests"), "{log}");
}

#[test]
fn malformed_value_names_setting_and_format() {
    let (code, log) = serve(
        &[],
        &[
            ("ADNT_HTTP_PUBLIC_URL", "http://adunata.example.org"),
            ("ADNT_DATABASE_URL", UNREACHABLE_DATABASE),
            ("ADNT_SMTP_HOST", "mail.example.org"),
            ("ADNT_MAIL_SENDER", "Adunata <adunata@example.org>"),
            ("ADNT_LOCALIZATION_TIME_ZONE", "Europe/Berlin"),
        ],
    );
    assert_eq!(code, Some(1));
    assert!(
        log.contains("ADNT_HTTP_PUBLIC_URL must be an absolute HTTPS URL"),
        "{log}"
    );
}

#[test]
fn unknown_time_zone_stops_before_the_database() {
    let (code, log) = serve(
        &[],
        &[
            ("ADNT_HTTP_PUBLIC_URL", "https://adunata.example.org"),
            ("ADNT_DATABASE_URL", UNREACHABLE_DATABASE),
            ("ADNT_SMTP_HOST", "mail.example.org"),
            ("ADNT_MAIL_SENDER", "Adunata <adunata@example.org>"),
            ("ADNT_LOCALIZATION_TIME_ZONE", "Europe/Wernigerode"),
        ],
    );
    assert_eq!(code, Some(1));
    assert!(
        log.contains("ADNT_LOCALIZATION_TIME_ZONE must be an IANA time zone"),
        "{log}"
    );
    assert!(!log.contains("database is not ready"), "{log}");
}

/// A development build reports the malformed line. A release build never opens the file and misses the settings in it instead.
#[test]
fn settings_file_in_the_working_directory() {
    let directory = WorkingDirectory::new();
    std::fs::write(
        directory.path.join(".env"),
        "ADNT_DATABASE_URL=postgres://adunata@127.0.0.1:1/adunata\n",
    )
    .expect("settings file must be writable");
    std::fs::write(
        directory.path.join(".env.local"),
        "ADNT_SMTP_HOST=localhost\nexport ADNT_SMTP_PORT=1025\n",
    )
    .expect("settings file must be writable");
    let (code, output, error) = run_in(&directory.path, &["serve"], &[]);
    let log = format!("{output}{error}");

    assert_eq!(code, Some(1));
    if cfg!(debug_assertions) {
        assert!(log.contains("line 2 of"), "{log}");
        assert!(log.contains(".env.local"), "{log}");
        assert!(!log.contains("ADNT_DATABASE_URL"), "{log}");
    } else {
        assert!(!log.contains(".env"), "{log}");
        assert!(
            log.contains("ADNT_DATABASE_URL is required but not set"),
            "{log}"
        );
    }
}

#[test]
fn option_wins_over_variable() {
    let file = configuration_file("option.toml", "[smpt]\nhost = \"mail.example.org\"\n");
    let (code, log) = serve(
        &["--config", &file],
        &[("ADNT_CONFIG_FILE", "/nonexistent/adunata.toml")],
    );
    assert_eq!(code, Some(1));
    assert!(log.contains(&file), "{log}");
    assert!(log.contains("smpt.host"), "{log}");
    assert!(!log.contains("/nonexistent/adunata.toml"), "{log}");
}

#[test]
fn empty_variable_names_no_file() {
    let (code, log) = serve(&[], &[("ADNT_CONFIG_FILE", "")]);
    assert_eq!(code, Some(1));
    assert!(log.contains("is required but not set"), "{log}");
    assert!(!log.contains("configuration file"), "{log}");
}

#[test]
fn named_file_missing() {
    let (code, log) = serve(&[], &[("ADNT_CONFIG_FILE", "/nonexistent/adunata.toml")]);
    assert_eq!(code, Some(1));
    assert!(log.contains("/nonexistent/adunata.toml"), "{log}");
    assert!(!log.contains("is required but not set"), "{log}");
}

#[test]
fn parse_error_never_shows_the_line() {
    let file = configuration_file(
        "secret.toml",
        "[http]\npublic-url = \"https://adunata.example.org\"\n\n[database]\npassword = pa$$word\n",
    );
    let (code, log) = serve(&["--config", &file], &[]);
    assert_eq!(code, Some(1));
    assert!(log.contains(&file), "{log}");
    assert!(log.contains("line 5, column 12"), "{log}");
    assert!(!log.contains("pa$$word"), "{log}");
    assert!(!log.contains("is required but not set"), "{log}");
}

#[test]
fn grant_organizer_needs_mail_and_both_names() {
    let (code, _, error) = run(&["grant-organizer", "ada@example.org", "Ada"], &[]);
    assert_eq!(code, Some(2));
    assert!(error.contains("<LAST_NAME>"), "{error}");
    assert!(error.contains("Usage: adunata grant-organizer"), "{error}");
}

#[test]
fn grant_organizer_rejects_a_malformed_address() {
    let (code, _, error) = run(&["grant-organizer", "ada", "Ada", "Lovelace"], &[]);
    assert_eq!(code, Some(2));
    assert!(error.contains("expected a mail address"), "{error}");
}

#[test]
fn without_a_command_the_binary_prints_its_help() {
    for variables in [
        &[][..],
        &[("ADNT_CONFIG_FILE", "/etc/adunata/adunata.toml")],
    ] {
        let (code, _, error) = run(&[], variables);
        assert_eq!(code, Some(2));
        for command in ["serve", "grant-organizer", "openapi"] {
            assert!(error.contains(command), "{error}");
        }
        assert!(error.contains("Usage: adunata"), "{error}");
    }
}

#[test]
fn help_and_version_succeed() {
    let (code, help, _) = run(&["help", "grant-organizer"], &[]);
    assert_eq!(code, Some(0));
    assert!(help.contains("lost every organizer mailbox"), "{help}");
    let (code, version, _) = run(&["--version"], &[]);
    assert_eq!(code, Some(0));
    assert!(version.starts_with("adunata "), "{version}");
}

#[test]
fn serve_offers_development_mode() {
    let (code, help, _) = run(&["help", "serve"], &[]);
    assert_eq!(code, Some(0));
    assert!(help.contains("--dev"), "{help}");
}

#[test]
fn development_mode_needs_a_public_url_on_localhost() {
    let (code, output, error) = run(
        &["serve", "--dev"],
        &[
            ("ADNT_HTTP_PUBLIC_URL", "https://adunata.example.org"),
            ("ADNT_DATABASE_URL", UNREACHABLE_DATABASE),
            ("ADNT_SMTP_HOST", "mail.example.org"),
            ("ADNT_MAIL_SENDER", "Adunata <adunata@example.org>"),
            ("ADNT_LOCALIZATION_TIME_ZONE", "Europe/Berlin"),
        ],
    );
    let log = format!("{output}{error}");
    assert_eq!(code, Some(1));
    assert!(log.contains("ADNT_HTTP_PUBLIC_URL on localhost"), "{log}");
    assert!(!log.contains("database is not ready"), "{log}");
}

#[test]
fn development_mode_needs_a_loopback_listen_address() {
    let (code, output, error) = run(
        &["serve", "--dev"],
        &[
            ("ADNT_HTTP_PUBLIC_URL", "http://localhost:8080"),
            ("ADNT_HTTP_LISTEN_ADDRESS", "0.0.0.0:8080"),
            ("ADNT_DATABASE_URL", UNREACHABLE_DATABASE),
            ("ADNT_SMTP_HOST", "mail.example.org"),
            ("ADNT_MAIL_SENDER", "Adunata <adunata@example.org>"),
            ("ADNT_LOCALIZATION_TIME_ZONE", "Europe/Berlin"),
        ],
    );
    let log = format!("{output}{error}");
    assert_eq!(code, Some(1));
    assert!(
        log.contains("ADNT_HTTP_LISTEN_ADDRESS on a loopback address"),
        "{log}"
    );
    assert!(!log.contains("database is not ready"), "{log}");
}

#[test]
fn unknown_command() {
    let (code, _, error) = run(&["serv"], &[]);
    assert_eq!(code, Some(2));
    assert!(error.contains("serve"), "{error}");
}
