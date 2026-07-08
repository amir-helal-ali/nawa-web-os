//! Password hashing using SHA-256 + random salt.
//!
//! Format: `{salt_hex}${hash_hex}`
//! Salt: 32 bytes random, hex-encoded (64 chars).
//! Hash: SHA-256(salt || password), hex-encoded (64 chars).

use sha2::{Digest, Sha256};
use rand::RngCore;

/// Password hashing error.
#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("invalid hash format")]
    InvalidFormat,
    #[error("password does not match")]
    Mismatch,
}

/// Hash a password with a random salt.
pub fn hash_password(password: &str) -> String {
    let mut salt = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    let salt_hex = hex::encode(salt);

    let mut hasher = Sha256::new();
    hasher.update(salt);
    hasher.update(password.as_bytes());
    let hash = hasher.finalize();
    let hash_hex = hex::encode(hash);

    format!("{salt_hex}${hash_hex}")
}

/// Verify a password against a stored hash.
pub fn verify_password(password: &str, stored: &str) -> Result<bool, PasswordError> {
    let parts: Vec<&str> = stored.split('$').collect();
    if parts.len() != 2 {
        return Err(PasswordError::InvalidFormat);
    }
    let salt = hex::decode(parts[0]).map_err(|_| PasswordError::InvalidFormat)?;
    let expected_hash = parts[1];

    let mut hasher = Sha256::new();
    hasher.update(&salt);
    hasher.update(password.as_bytes());
    let hash = hasher.finalize();
    let hash_hex = hex::encode(hash);

    Ok(hash_hex == expected_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify() {
        let password = "my-secret-password";
        let hashed = hash_password(password);
        assert!(verify_password(password, &hashed).unwrap());
    }

    #[test]
    fn wrong_password_fails() {
        let hashed = hash_password("correct");
        assert!(!verify_password("wrong", &hashed).unwrap());
    }

    #[test]
    fn different_salts() {
        let h1 = hash_password("same");
        let h2 = hash_password("same");
        assert_ne!(h1, h2); // different salts
        assert!(verify_password("same", &h1).unwrap());
        assert!(verify_password("same", &h2).unwrap());
    }

    #[test]
    fn invalid_format() {
        assert!(verify_password("x", "no-dollar-sign").is_err());
    }
}
