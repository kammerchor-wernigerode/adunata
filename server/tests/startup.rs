use std::process::Command;

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
