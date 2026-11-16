use std::fmt::Write;

use axum::http::{HeaderMap, header};
use sha2::{Digest, Sha256};

const SECRET_BYTES: usize = 32;

/// Creates a random secret of 256 bits from the generator of the operating system, written as 64 hexadecimal characters.
pub fn generate() -> String {
    let mut bytes = [0; SECRET_BYTES];
    getrandom::fill(&mut bytes).expect("the operating system must provide random numbers");
    hex(&bytes)
}

/// The SHA-256 hash of a secret, which is all that Adunata stores of it.
pub fn hash(secret: &str) -> Vec<u8> {
    Sha256::digest(secret.as_bytes()).to_vec()
}

/// The secret that a client sends as `Authorization: Bearer <secret>`. Personal links carry it in the fragment, which never reaches the server on its own.
pub fn from_authorization(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|secret| !secret.is_empty())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut text, byte| {
        write!(text, "{byte:02x}").expect("writing to a string cannot fail");
        text
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_has_64_hexadecimal_characters() {
        let secret = generate();
        assert_eq!(secret.len(), 64);
        assert!(
            secret
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        );
        assert_ne!(secret, generate());
    }

    #[test]
    fn hash_is_stable() {
        assert_eq!(
            hex(&hash("abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(hash("abc"), hash("abc"));
    }
}
