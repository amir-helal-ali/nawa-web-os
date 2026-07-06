//! JWT (JSON Web Token) codec — HS256 signing.
//!
//! Minimal JWT implementation without external dependencies.
//! Header: {"alg":"HS256","typ":"JWT"}
//! Payload: custom claims (sub, role, exp, iat)

use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// JWT error type.
#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("invalid token format")]
    InvalidFormat,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("token expired")]
    Expired,
    #[error("invalid base64: {0}")]
    Base64(String),
    #[error("invalid json: {0}")]
    Json(String),
}

/// JWT claims — the payload of the token.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JwtClaims {
    /// Subject (user ID).
    pub sub: String,
    /// Role (admin / user).
    pub role: String,
    /// Issued at (Unix timestamp).
    pub iat: u64,
    /// Expiry (Unix timestamp).
    pub exp: u64,
}

/// JWT codec — encode/decode tokens with HS256.
pub struct JwtCodec {
    secret: String,
}

impl JwtCodec {
    /// Create a new codec with the given secret.
    pub fn new(secret: impl Into<String>) -> Self {
        Self { secret: secret.into() }
    }

    /// Encode claims into a JWT string.
    pub fn encode(&self, claims: &JwtClaims) -> Result<String, JwtError> {
        let header = r#"{"alg":"HS256","typ":"JWT"}"#;
        let header_b64 = general_purpose::URL_SAFE_NO_PAD.encode(header);
        let payload_json = serde_json::to_string(claims)
            .map_err(|e| JwtError::Json(e.to_string()))?;
        let payload_b64 = general_purpose::URL_SAFE_NO_PAD.encode(&payload_json);

        let signing_input = format!("{header_b64}.{payload_b64}");
        let signature = self.sign(&signing_input);

        Ok(format!("{signing_input}.{signature}"))
    }

    /// Decode a JWT string into claims (verifies signature + expiry).
    pub fn decode(&self, token: &str) -> Result<JwtClaims, JwtError> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(JwtError::InvalidFormat);
        }

        let signing_input = format!("{}.{}", parts[0], parts[1]);
        let expected_sig = self.sign(&signing_input);

        if parts[2] != expected_sig {
            return Err(JwtError::InvalidSignature);
        }

        let payload_bytes = general_purpose::URL_SAFE_NO_PAD
            .decode(parts[1])
            .map_err(|e| JwtError::Base64(e.to_string()))?;
        let payload_str = String::from_utf8_lossy(&payload_bytes);

        let claims: JwtClaims = serde_json::from_str(&payload_str)
            .map_err(|e| JwtError::Json(e.to_string()))?;

        // Check expiry.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if claims.exp < now {
            return Err(JwtError::Expired);
        }

        Ok(claims)
    }

    /// Create a token for a user.
    pub fn create_token(&self, user_id: &str, role: &str, expiry_secs: u64) -> Result<String, JwtError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let claims = JwtClaims {
            sub: user_id.to_string(),
            role: role.to_string(),
            iat: now,
            exp: now + expiry_secs,
        };
        self.encode(&claims)
    }

    /// Sign data with HMAC-SHA256.
    fn sign(&self, data: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.secret.as_bytes())
            .expect("HMAC accepts any key length");
        mac.update(data.as_bytes());
        let result = mac.finalize();
        general_purpose::URL_SAFE_NO_PAD.encode(result.into_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let codec = JwtCodec::new("test-secret");
        let claims = JwtClaims {
            sub: "user:1".into(),
            role: "admin".into(),
            iat: 1000,
            exp: 9999999999,
        };
        let token = codec.encode(&claims).unwrap();
        let decoded = codec.decode(&token).unwrap();
        assert_eq!(decoded.sub, "user:1");
        assert_eq!(decoded.role, "admin");
    }

    #[test]
    fn invalid_signature_rejected() {
        let codec1 = JwtCodec::new("secret1");
        let codec2 = JwtCodec::new("secret2");
        let token = codec1.create_token("user:1", "admin", 3600).unwrap();
        assert!(codec2.decode(&token).is_err());
    }

    #[test]
    fn expired_token_rejected() {
        let codec = JwtCodec::new("test");
        let claims = JwtClaims {
            sub: "user:1".into(),
            role: "user".into(),
            iat: 1000,
            exp: 1001, // expired
        };
        let token = codec.encode(&claims).unwrap();
        assert!(matches!(codec.decode(&token), Err(JwtError::Expired)));
    }

    #[test]
    fn create_token_works() {
        let codec = JwtCodec::new("secret");
        let token = codec.create_token("user:42", "user", 3600).unwrap();
        let claims = codec.decode(&token).unwrap();
        assert_eq!(claims.sub, "user:42");
        assert_eq!(claims.role, "user");
    }

    #[test]
    fn invalid_format_rejected() {
        let codec = JwtCodec::new("secret");
        assert!(codec.decode("not-a-jwt").is_err());
        assert!(codec.decode("a.b").is_err());
    }
}
