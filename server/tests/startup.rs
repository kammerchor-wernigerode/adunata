use std::process::Command;

/// Starts the binary without the public URL and the mail server host. The database URL points to a server that does not exist, so a start that touched the database would report it instead.
#[test]
fn several_settings_missing_stops_before_anything_opens() {
    let output = Command::new(env!("CARGO_BIN_EXE_adunata"))
        .env_clear()
        .env(
            "ADUNATA_DATABASE_URL",
            "postgres://adunata@127.0.0.1:1/adunata",
        )
        .env("ADUNATA_MAIL_SENDER", "Adunata <adunata@example.org>")
        .output()
        .expect("binary must run");

    assert_eq!(output.status.code(), Some(1));
    let log = String::from_utf8_lossy(&output.stdout);
    assert!(log.contains("ADUNATA_PUBLIC_URL"), "{log}");
    assert!(log.contains("ADUNATA_SMTP_HOST"), "{log}");
    assert!(!log.contains("database is not ready"), "{log}");
    assert!(!log.contains("accepting requests"), "{log}");
}

#[test]
fn malformed_value_names_setting_and_format() {
    let output = Command::new(env!("CARGO_BIN_EXE_adunata"))
        .env_clear()
        .env("ADUNATA_PUBLIC_URL", "http://adunata.example.org")
        .env(
            "ADUNATA_DATABASE_URL",
            "postgres://adunata@127.0.0.1:1/adunata",
        )
        .env("ADUNATA_SMTP_HOST", "mail.example.org")
        .env("ADUNATA_MAIL_SENDER", "Adunata <adunata@example.org>")
        .output()
        .expect("binary must run");

    assert_eq!(output.status.code(), Some(1));
    let log = String::from_utf8_lossy(&output.stdout);
    assert!(
        log.contains("ADUNATA_PUBLIC_URL must be an absolute HTTPS URL"),
        "{log}"
    );
}
