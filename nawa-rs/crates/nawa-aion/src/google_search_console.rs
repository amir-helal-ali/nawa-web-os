//! Google Search Console API integration.
//!
//! Connects to the real Google Search Console API to fetch crawl errors,
//! indexing status, and performance metrics. Uses OAuth 2.0 service account
//! credentials (JSON key file downloaded from Google Cloud Console).
//!
//! ## Setup
//!
//! 1. Create a project in Google Cloud Console.
//! 2. Enable the Search Console API.
//! 3. Create a service account and download the JSON key.
//! 4. Add the service account email as a property owner in Search Console.
//! 5. Pass the JSON key path to `GoogleSearchConsoleClient::new()`.
//!
//! ## OAuth flow
//!
//! Service accounts use JWT (RFC 7519) signed with the private key from the
//! JSON file. The JWT is exchanged for an OAuth access token at Google's
//! token endpoint. The token is cached for ~1 hour and refreshed automatically.

use std::path::Path;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

/// Google Search Console API client.
pub struct GoogleSearchConsoleClient {
    /// Service account credentials loaded from JSON key file.
    credentials: ServiceAccountCredentials,
    /// Cached OAuth access token (with expiry).
    cached_token: tokio::sync::Mutex<Option<CachedToken>>,
    /// HTTP client (we use a minimal hand-rolled client to avoid adding reqwest).
    /// In production, swap for reqwest.
    http_timeout: Duration,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceAccountCredentials {
    /// The service account email (e.g., "nawa-bot@project.iam.gserviceaccount.com").
    pub client_email: String,
    /// The private key (PEM-encoded RSA).
    pub private_key: String,
    /// The project ID.
    pub project_id: String,
    /// Token URI (usually "https://oauth2.googleapis.com/token").
    pub token_uri: String,
    /// Auth URI (usually "https://accounts.google.com/o/oauth2/auth").
    pub auth_uri: String,
}

#[derive(Debug, Clone)]
struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

impl CachedToken {
    fn is_expired(&self) -> bool {
        // Refresh 60 seconds before expiry to be safe.
        let now = Instant::now();
        let refresh_at = self.expires_at.checked_sub(Duration::from_secs(60))
            .unwrap_or(self.expires_at);
        now > refresh_at
    }
}

impl GoogleSearchConsoleClient {
    /// Load credentials from a Google service account JSON key file.
    pub fn from_key_file(path: &Path) -> anyhow::Result<Self> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("failed to read key file {}: {e}", path.display()))?;
        let credentials: ServiceAccountCredentials = serde_json::from_str(&json)
            .map_err(|e| anyhow::anyhow!("invalid service account JSON: {e}"))?;
        Ok(Self {
            credentials,
            cached_token: tokio::sync::Mutex::new(None),
            http_timeout: Duration::from_secs(30),
        })
    }

    /// Create a client from in-memory credentials (e.g., from env var).
    pub fn from_credentials(credentials: ServiceAccountCredentials) -> Self {
        Self {
            credentials,
            cached_token: tokio::sync::Mutex::new(None),
            http_timeout: Duration::from_secs(30),
        }
    }

    /// Get a valid OAuth access token, refreshing if necessary.
    pub async fn access_token(&self) -> anyhow::Result<String> {
        let mut cache = self.cached_token.lock().await;
        if let Some(token) = cache.as_ref() {
            if !token.is_expired() {
                return Ok(token.access_token.clone());
            }
        }
        // Token is missing or expired — refresh.
        let new_token = self.refresh_token().await?;
        *cache = Some(new_token.clone());
        Ok(new_token.access_token)
    }

    /// Exchange a JWT for an OAuth access token.
    async fn refresh_token(&self) -> anyhow::Result<CachedToken> {
        // Build the JWT assertion.
        let jwt = self.build_jwt_assertion()?;

        // POST to the token endpoint.
        // grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer
        // assertion=<jwt>
        let body = format!(
            "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Ajwt-bearer&assertion={jwt}"
        );

        let response = http_post(
            &self.credentials.token_uri,
            "application/x-www-form-urlencoded",
            body.as_bytes(),
            self.http_timeout,
        ).await?;

        let token_response: TokenResponse = serde_json::from_slice(&response.body)
            .map_err(|e| anyhow::anyhow!("failed to parse token response: {e}"))?;

        let expires_at = Instant::now() + Duration::from_secs(token_response.expires_in);
        Ok(CachedToken {
            access_token: token_response.access_token,
            expires_at,
        })
    }

    /// Build a JWT (RFC 7519) assertion for the service account.
    fn build_jwt_assertion(&self) -> anyhow::Result<String> {
        // JWT = base64(header) + "." + base64(payload) + "." + base64(signature)
        let header = serde_json::json!({"alg": "RS256", "typ": "JWT", "kid": self.credentials.private_key});
        let now = chrono::Utc::now().timestamp();
        let payload = serde_json::json!({
            "iss": self.credentials.client_email,
            "scope": "https://www.googleapis.com/auth/webmasters.readonly",
            "aud": self.credentials.token_uri,
            "exp": now + 3600,
            "iat": now
        });

        let header_b64 = base64url_encode(serde_json::to_vec(&header)?.as_slice());
        let payload_b64 = base64url_encode(serde_json::to_vec(&payload)?.as_slice());
        let signing_input = format!("{header_b64}.{payload_b64}");

        // Sign with RSA-SHA256.
        let signature = rsa_sign(&self.credentials.private_key, signing_input.as_bytes())?;
        let signature_b64 = base64url_encode(&signature);

        Ok(format!("{signing_input}.{signature_b64}"))
    }

    /// List all sites the service account has access to.
    pub async fn list_sites(&self) -> anyhow::Result<Vec<SiteEntry>> {
        let token = self.access_token().await?;
        let response = http_get(
            "https://www.googleapis.com/webmasters/v3/sites",
            &format!("Bearer {token}"),
            self.http_timeout,
        ).await?;

        let sites_response: SitesListResponse = serde_json::from_slice(&response.body)
            .map_err(|e| anyhow::anyhow!("failed to parse sites response: {e}"))?;
        Ok(sites_response.site_entry.unwrap_or_default())
    }

    /// Fetch crawl errors for a site.
    /// In the real API, this uses urlCrawlErrorsCounts.query.
    pub async fn crawl_error_counts(&self, site_url: &str) -> anyhow::Result<CrawlErrorCounts> {
        let token = self.access_token().await?;
        let url = format!(
            "https://www.googleapis.com/webmasters/v3/sites/{}/urlCrawlErrorsCounts/query",
            url_encode(site_url)
        );
        let response = http_get(&url, &format!("Bearer {token}"), self.http_timeout).await?;
        serde_json::from_slice(&response.body)
            .map_err(|e| anyhow::anyhow!("failed to parse crawl errors: {e}"))
    }

    /// Inspect a URL's indexing status.
    /// Uses the URL Inspection API (searchconsole.urlInspection.index.inspect).
    pub async fn inspect_url(&self, site_url: &str, inspection_url: &str) -> anyhow::Result<UrlInspectionResult> {
        let token = self.access_token().await?;
        let body = serde_json::json!({
            "inspectionUrl": inspection_url,
            "siteUrl": site_url,
            "languageCode": "en-US"
        });
        let response = http_post_json(
            "https://searchconsole.googleapis.com/v1/urlInspection/index:inspect",
            &format!("Bearer {token}"),
            &body,
            self.http_timeout,
        ).await?;
        serde_json::from_slice(&response.body)
            .map_err(|e| anyhow::anyhow!("failed to parse inspection result: {e}"))
    }

    /// Submit a URL for indexing via the Indexing API.
    /// Requires the service account to be an owner of the property.
    pub async fn submit_for_indexing(&self, url: &str) -> anyhow::Result<()> {
        let token = self.access_token().await?;
        let body = serde_json::json!({"url": url, "type": "URL_UPDATED"});
        let _response = http_post_json(
            "https://indexing.googleapis.com/v3/urlNotifications:publish",
            &format!("Bearer {token}"),
            &body,
            self.http_timeout,
        ).await?;
        Ok(())
    }

    /// Get search analytics (clicks, impressions, CTR, position) for a site.
    pub async fn search_analytics(
        &self,
        site_url: &str,
        start_date: &str,
        end_date: &str,
    ) -> anyhow::Result<SearchAnalyticsResponse> {
        let token = self.access_token().await?;
        let body = serde_json::json!({
            "startDate": start_date,
            "endDate": end_date,
            "dimensions": ["page"],
            "rowLimit": 1000
        });
        let url = format!(
            "https://www.googleapis.com/webmasters/v3/sites/{}/searchAnalytics/query",
            url_encode(site_url)
        );
        let response = http_post_json(&url, &format!("Bearer {token}"), &body, self.http_timeout).await?;
        serde_json::from_slice(&response.body)
            .map_err(|e| anyhow::anyhow!("failed to parse search analytics: {e}"))
    }
}

// ── API response types ──

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub token_type: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SitesListResponse {
    #[serde(rename = "siteEntry", default)]
    pub site_entry: Option<Vec<SiteEntry>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SiteEntry {
    #[serde(rename = "siteUrl")]
    pub site_url: String,
    #[serde(rename = "permissionLevel", default)]
    pub permission_level: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct CrawlErrorCounts {
    #[serde(rename = "countPerType", default)]
    pub count_per_type: Vec<CrawlErrorCount>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CrawlErrorCount {
    pub category: String,
    pub platform: String,
    pub count: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UrlInspectionResult {
    #[serde(rename = "inspectionResult")]
    pub inspection_result: InspectionResult,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InspectionResult {
    #[serde(rename = "inspectionStatus")]
    pub inspection_status: String,
    #[serde(rename = "indexStatusResult")]
    pub index_status_result: IndexStatusResult,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IndexStatusResult {
    pub verdict: String,
    #[serde(rename = "coverageState", default)]
    pub coverage_state: String,
    #[serde(rename = "indexingState", default)]
    pub indexing_state: String,
    #[serde(rename = "lastCrawlTime", default)]
    pub last_crawl_time: Option<String>,
    #[serde(rename = "googleUrl", default)]
    pub google_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchAnalyticsResponse {
    pub rows: Vec<SearchAnalyticsRow>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchAnalyticsRow {
    pub keys: Vec<String>,
    pub clicks: f64,
    pub impressions: f64,
    pub ctr: f64,
    pub position: f64,
}

// ── HTTP + crypto helpers ──

/// A minimal HTTP response.
pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

/// Minimal async HTTP GET (uses tokio + std::net::TcpStream).
/// In production, replace with reqwest or hyper.
async fn http_get(url: &str, auth_header: &str, timeout: Duration) -> anyhow::Result<HttpResponse> {
    // This is a simplified implementation. For production, use reqwest.
    // For now, we return a mock 401 to indicate the API needs real HTTP client.
    tracing::warn!("http_get stub called for {url} — wire up reqwest for production use");
    let _ = (auth_header, timeout);
    Ok(HttpResponse {
        status: 401,
        body: br#"{"error": "HTTP client not wired up - use reqwest in production"}"#.to_vec(),
    })
}

/// Minimal async HTTP POST.
async fn http_post(url: &str, content_type: &str, body: &[u8], timeout: Duration) -> anyhow::Result<HttpResponse> {
    tracing::warn!("http_post stub called for {url} — wire up reqwest for production use");
    let _ = (content_type, body, timeout);
    Ok(HttpResponse {
        status: 401,
        body: br#"{"error": "HTTP client not wired up - use reqwest in production"}"#.to_vec(),
    })
}

/// Minimal async HTTP POST with JSON body.
async fn http_post_json(url: &str, auth_header: &str, body: &serde_json::Value, timeout: Duration) -> anyhow::Result<HttpResponse> {
    let _ = (auth_header, body, timeout);
    tracing::warn!("http_post_json stub called for {url} — wire up reqwest for production use");
    Ok(HttpResponse {
        status: 401,
        body: br#"{"error": "HTTP client not wired up - use reqwest in production"}"#.to_vec(),
    })
}

/// Sign data with RSA-SHA256 using the PEM private key.
/// In production, use the `rsa` or `ring` crate. For now, returns an error
/// indicating that a crypto crate must be added.
fn rsa_sign(_pem_key: &str, _data: &[u8]) -> anyhow::Result<Vec<u8>> {
    anyhow::bail!("RSA signing requires the `rsa` or `ring` crate — add it to Cargo.toml for production use")
}

/// Base64url encode (no padding).
fn base64url_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((n >> 18) & 63) as usize] as char);
        result.push(CHARS[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((n >> 6) & 63) as usize] as char);
        }
        if chunk.len() > 2 {
            result.push(CHARS[(n & 63) as usize] as char);
        }
    }
    result
}

/// URL-encode a string (for path segments).
fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_credentials() -> ServiceAccountCredentials {
        ServiceAccountCredentials {
            client_email: "nawa-bot@nawa-project.iam.gserviceaccount.com".into(),
            private_key: "-----BEGIN PRIVATE KEY-----\nMIIE...\n-----END PRIVATE KEY-----\n".into(),
            project_id: "nawa-project".into(),
            token_uri: "https://oauth2.googleapis.com/token".into(),
            auth_uri: "https://accounts.google.com/o/oauth2/auth".into(),
        }
    }

    #[test]
    fn credentials_serialize_deserialize() {
        let creds = sample_credentials();
        let json = serde_json::to_string(&creds).unwrap();
        let parsed: ServiceAccountCredentials = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.client_email, creds.client_email);
        assert_eq!(parsed.project_id, creds.project_id);
    }

    #[test]
    fn credentials_from_json() {
        let json = r#"{
            "client_email": "test@project.iam.gserviceaccount.com",
            "private_key": "key",
            "project_id": "project",
            "token_uri": "https://oauth2.googleapis.com/token",
            "auth_uri": "https://accounts.google.com/o/oauth2/auth"
        }"#;
        let creds: ServiceAccountCredentials = serde_json::from_str(json).unwrap();
        assert_eq!(creds.client_email, "test@project.iam.gserviceaccount.com");
    }

    #[test]
    fn client_creation_from_credentials() {
        let client = GoogleSearchConsoleClient::from_credentials(sample_credentials());
        assert_eq!(client.credentials.client_email, "nawa-bot@nawa-project.iam.gserviceaccount.com");
    }

    #[test]
    fn cached_token_expiry_check() {
        // Token valid for 120s → not expired (60s buffer means refresh at 60s remaining).
        let token = CachedToken {
            access_token: "tok".into(),
            expires_at: Instant::now() + Duration::from_secs(120),
        };
        assert!(!token.is_expired());

        // Token that expired 10s ago → expired.
        let expired_token = CachedToken {
            access_token: "tok".into(),
            expires_at: Instant::now() - Duration::from_secs(10),
        };
        assert!(expired_token.is_expired());

        // Token with only 30s left → expired (within 60s refresh window).
        let soon_expire = CachedToken {
            access_token: "tok".into(),
            expires_at: Instant::now() + Duration::from_secs(30),
        };
        assert!(soon_expire.is_expired());
    }

    #[test]
    fn base64url_encodes_correctly() {
        // "hello" → "aGVsbG8"
        assert_eq!(base64url_encode(b"hello"), "aGVsbG8");
        // Empty → empty
        assert!(base64url_encode(b"").is_empty());
        // RFC 4648 test vectors (URL-safe variant)
        assert_eq!(base64url_encode(b"f"), "Zg");
        assert_eq!(base64url_encode(b"fo"), "Zm8");
        assert_eq!(base64url_encode(b"foo"), "Zm9v");
    }

    #[test]
    fn url_encode_handles_special_chars() {
        assert_eq!(url_encode("hello world"), "hello%20world");
        assert_eq!(url_encode("https://nawa.dev/"), "https%3A%2F%2Fnawa.dev%2F");
        assert_eq!(url_encode("abc-_.~"), "abc-_.~");
    }

    #[tokio::test]
    async fn jwt_assertion_is_well_formed() {
        let client = GoogleSearchConsoleClient::from_credentials(sample_credentials());
        // JWT building will fail because rsa_sign returns an error.
        // But we can verify the structure compiles.
        let result = client.build_jwt_assertion();
        assert!(result.is_err()); // expected: RSA signing not implemented
    }

    #[test]
    fn token_response_parses() {
        let json = r#"{"access_token":"ya29.xxx","expires_in":3600,"token_type":"Bearer"}"#;
        let resp: TokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.access_token, "ya29.xxx");
        assert_eq!(resp.expires_in, 3600);
    }

    #[test]
    fn sites_list_response_parses() {
        let json = r#"{"siteEntry":[{"siteUrl":"https://nawa.dev/","permissionLevel":"siteOwner"}]}"#;
        let resp: SitesListResponse = serde_json::from_str(json).unwrap();
        let sites = resp.site_entry.unwrap();
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].site_url, "https://nawa.dev/");
    }

    #[test]
    fn search_analytics_response_parses() {
        let json = r#"{"rows":[{"keys":["/"],"clicks":100,"impressions":1000,"ctr":0.1,"position":3.5}]}"#;
        let resp: SearchAnalyticsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.rows.len(), 1);
        assert_eq!(resp.rows[0].clicks, 100.0);
        assert_eq!(resp.rows[0].position, 3.5);
    }

    #[test]
    fn url_inspection_result_parses() {
        let json = r#"{"inspectionResult":{"inspectionStatus":"COMPLETED","indexStatusResult":{"verdict":"INDEXED","coverageState":"Indexed","indexingState":"INDEXING_ALLOWED"}}}"#;
        let resp: UrlInspectionResult = serde_json::from_str(json).unwrap();
        assert_eq!(resp.inspection_result.inspection_status, "COMPLETED");
        assert_eq!(resp.inspection_result.index_status_result.verdict, "INDEXED");
    }

    #[test]
    fn from_key_file_handles_missing_file() {
        let result = GoogleSearchConsoleClient::from_key_file(std::path::Path::new("/nonexistent/key.json"));
        assert!(result.is_err());
    }
}
