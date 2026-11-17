use std::process::Command;

const REQUIRED_FILE: &str = r#"
[http]
public-url = "https://adunata.example.org"

[database]
url = "postgres://adunata@127.0.0.1:1/adunata"

[smtp]
host = "mail.example.org"

[mail]
sender = "Adunata <adunata@example.org>"

[localization]
time-zone = "Europe/Berlin"
"#;

fn configuration_file(name: &str, content: &str) -> String {
    let directory = std::env::temp_dir().join(format!("adunata-startup-{}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("temporary directory must be writable");
    let path = directory.join(name);
    std::fs::write(&path, content).expect("configuration file must be writable");
    path.to_string_lossy().into_owned()
}

/// Starts `serve` with only the given arguments and variables. Returns the exit code and everything the binary printed.
fn serve(arguments: &[&str], variables: &[(&str, &str)]) -> (Option<i32>, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_adunata"))
        .env_clear()
        .envs(variables.iter().copied())
        .args(arguments)
        .arg("serve")
        .output()
        .expect("binary must run");
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (output.status.code(), printed)
}

/// A start that reaches the database, which does not exist, has accepted its configuration.
fn assert_configuration_accepted(log: &str) {
    assert!(log.contains("database is not ready"), "{log}");
    assert!(!log.contains("configuration is invalid"), "{log}");
}

#[test]
fn start_from_a_configuration_file() {
    let file = configuration_file("required.toml", REQUIRED_FILE);
    let (code, log) = serve(&["--config", &file], &[]);
    assert_eq!(code, Some(1));
    assert_configuration_accepted(&log);
}

/// The file holds a listen address that Adunata rejects. A valid one in the environment hides it.
#[test]
fn environment_overrides_the_file() {
    let file = configuration_file(
        "listen.toml",
        &REQUIRED_FILE.replace("[http]\n", "[http]\nlisten-address = \"no address\"\n"),
    );
    let (code, log) = serve(
        &["--config", &file],
        &[("ADNT_HTTP_LISTEN_ADDRESS", "127.0.0.1:9000")],
    );
    assert_eq!(code, Some(1));
    assert_configuration_accepted(&log);

    let (code, log) = serve(&["--config", &file], &[]);
    assert_eq!(code, Some(1));
    assert!(
        log.contains("ADNT_HTTP_LISTEN_ADDRESS must be a socket address"),
        "{log}"
    );
}

#[test]
fn option_wins_over_variable() {
    let file = configuration_file("option.toml", REQUIRED_FILE);
    let (code, log) = serve(
        &["--config", &file],
        &[("ADNT_CONFIG_FILE", "/nonexistent/adunata.toml")],
    );
    assert_eq!(code, Some(1));
    assert_configuration_accepted(&log);
    assert!(!log.contains("/nonexistent/adunata.toml"), "{log}");
}

#[test]
fn empty_variable_names_no_file() {
    let (code, log) = serve(
        &[],
        &[
            ("ADNT_CONFIG_FILE", ""),
            ("ADNT_HTTP_PUBLIC_URL", "https://adunata.example.org"),
            (
                "ADNT_DATABASE_URL",
                "postgres://adunata@127.0.0.1:1/adunata",
            ),
            ("ADNT_SMTP_HOST", "mail.example.org"),
            ("ADNT_MAIL_SENDER", "Adunata <adunata@example.org>"),
            ("ADNT_LOCALIZATION_TIME_ZONE", "Europe/Berlin"),
        ],
    );
    assert_eq!(code, Some(1));
    assert_configuration_accepted(&log);
}

#[test]
fn unknown_key_in_the_file() {
    let file = configuration_file(
        "unknown.toml",
        &format!("{REQUIRED_FILE}\n[smpt]\nhost = \"mail.example.org\"\n"),
    );
    let (code, log) = serve(&["--config", &file], &[]);
    assert_eq!(code, Some(1));
    assert!(log.contains(&file), "{log}");
    assert!(log.contains("smpt.host"), "{log}");
    assert!(!log.contains("database is not ready"), "{log}");
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
        &REQUIRED_FILE.replace("[database]\n", "[database]\npassword = pa$$word\n"),
    );
    let (code, log) = serve(&["--config", &file], &[]);
    assert_eq!(code, Some(1));
    assert!(log.contains(&file), "{log}");
    assert!(log.contains("line 6, column 12"), "{log}");
    assert!(!log.contains("pa$$word"), "{log}");
    assert!(!log.contains("is required but not set"), "{log}");
}

/// Starts the binary without the public URL, the mail server host and the time zone. The database URL points to a server that does not exist, so a start that touched the database would report it instead.
#[test]
fn several_settings_missing_stops_before_anything_opens() {
    let output = Command::new(env!("CARGO_BIN_EXE_adunata"))
        .arg("serve")
        .env_clear()
        .env(
            "ADNT_DATABASE_URL",
            "postgres://adunata@127.0.0.1:1/adunata",
        )
        .env("ADNT_MAIL_SENDER", "Adunata <adunata@example.org>")
        .output()
        .expect("binary must run");

    assert_eq!(output.status.code(), Some(1));
    let log = String::from_utf8_lossy(&output.stdout);
    assert!(log.contains("ADNT_HTTP_PUBLIC_URL"), "{log}");
    assert!(log.contains("ADNT_SMTP_HOST"), "{log}");
    assert!(log.contains("ADNT_LOCALIZATION_TIME_ZONE"), "{log}");
    assert!(!log.contains("database is not ready"), "{log}");
    assert!(!log.contains("accepting requests"), "{log}");
}

#[test]
fn malformed_value_names_setting_and_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_adunata"))
        .arg("serve")
        .env_clear()
        .env("ADNT_HTTP_PUBLIC_URL", "http://adunata.example.org")
        .env(
            "ADNT_DATABASE_URL",
            "postgres://adunata@127.0.0.1:1/adunata",
        )
        .env("ADNT_SMTP_HOST", "mail.example.org")
        .env("ADNT_MAIL_SENDER", "Adunata <adunata@example.org>")
        .env("ADNT_LOCALIZATION_TIME_ZONE", "Europe/Berlin")
        .output()
        .expect("binary must run");

    assert_eq!(output.status.code(), Some(1));
    let log = String::from_utf8_lossy(&output.stdout);
    assert!(
        log.contains("ADNT_HTTP_PUBLIC_URL must be an absolute HTTPS URL"),
        "{log}"
    );
}

#[test]
fn unknown_time_zone_stops_before_the_database() {
    let output = Command::new(env!("CARGO_BIN_EXE_adunata"))
        .arg("serve")
        .env_clear()
        .env("ADNT_HTTP_PUBLIC_URL", "https://adunata.example.org")
        .env(
            "ADNT_DATABASE_URL",
            "postgres://adunata@127.0.0.1:1/adunata",
        )
        .env("ADNT_SMTP_HOST", "mail.example.org")
        .env("ADNT_MAIL_SENDER", "Adunata <adunata@example.org>")
        .env("ADNT_LOCALIZATION_TIME_ZONE", "Europe/Wernigerode")
        .output()
        .expect("binary must run");

    assert_eq!(output.status.code(), Some(1));
    let log = String::from_utf8_lossy(&output.stdout);
    assert!(
        log.contains("ADNT_LOCALIZATION_TIME_ZONE must be an IANA time zone"),
        "{log}"
    );
    assert!(!log.contains("database is not ready"), "{log}");
}

fn run(arguments: &[&str]) -> (Option<i32>, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_adunata"))
        .env_clear()
        .args(arguments)
        .output()
        .expect("binary must run");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn grant_organizer_needs_mail_and_both_names() {
    let (code, _, error) = run(&["grant-organizer", "ada@example.org", "Ada"]);
    assert_eq!(code, Some(2));
    assert!(error.contains("<LAST_NAME>"), "{error}");
    assert!(error.contains("Usage: adunata grant-organizer"), "{error}");
}

#[test]
fn grant_organizer_rejects_a_malformed_address() {
    let (code, _, error) = run(&["grant-organizer", "ada", "Ada", "Lovelace"]);
    assert_eq!(code, Some(2));
    assert!(error.contains("expected a mail address"), "{error}");
}

#[test]
fn without_a_command_the_binary_prints_its_help() {
    let (code, _, error) = run(&[]);
    assert_eq!(code, Some(2));
    for command in ["serve", "grant-organizer", "openapi"] {
        assert!(error.contains(command), "{error}");
    }
}

#[test]
fn help_and_version_succeed() {
    let (code, help, _) = run(&["help", "grant-organizer"]);
    assert_eq!(code, Some(0));
    assert!(help.contains("lost every organizer mailbox"), "{help}");
    let (code, version, _) = run(&["--version"]);
    assert_eq!(code, Some(0));
    assert!(version.starts_with("adunata "), "{version}");
}

#[test]
fn unknown_command() {
    let (code, _, error) = run(&["serv"]);
    assert_eq!(code, Some(2));
    assert!(error.contains("serve"), "{error}");
}
