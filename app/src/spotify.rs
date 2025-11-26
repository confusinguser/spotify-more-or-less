use base64::Engine;
use scraper::Html;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Spotify API client for fetching track information
/// Automatically handles OAuth2 authentication using client credentials flow
pub struct SpotifyClient {
    client: reqwest::Client,
    base_url: String,
    auth_url: String,
    client_id: String,
    client_secret: String,
    token_cache: Arc<RwLock<Option<TokenCache>>>,
}

#[derive(Clone)]
struct TokenCache {
    access_token: String,
    expires_at: Instant,
}

impl SpotifyClient {
    /// Create a new Spotify API client
    ///
    /// # Arguments
    /// * `client_id` - Spotify application client ID
    /// * `client_secret` - Spotify application client secret
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://api.spotify.com/v1".to_string(),
            auth_url: "https://accounts.spotify.com/api/token".to_string(),
            client_id,
            client_secret,
            token_cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Get a valid access token, refreshing if necessary
    async fn get_access_token(&self) -> Result<String, SpotifyError> {
        // Check if we have a valid cached token
        {
            let cache = self.token_cache.read().await;
            if let Some(token_cache) = cache.as_ref() {
                if token_cache.expires_at > Instant::now() {
                    return Ok(token_cache.access_token.clone());
                }
            }
        }

        // Request a new token
        let mut cache = self.token_cache.write().await;

        // Double-check in case another thread just refreshed
        if let Some(token_cache) = cache.as_ref() {
            if token_cache.expires_at > Instant::now() {
                return Ok(token_cache.access_token.clone());
            }
        }

        // Encode client credentials as base64
        let credentials = format!("{}:{}", self.client_id, self.client_secret);
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());

        let params = [("grant_type", "client_credentials")];

        let response = self
            .client
            .post(&self.auth_url)
            .header("Authorization", format!("Basic {}", encoded))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await
            .map_err(SpotifyError::RequestError)?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(SpotifyError::AuthenticationError(format!(
                "Failed to authenticate: {} - {}",
                status, error_text
            )));
        }

        let token_response: TokenResponse =
            response.json().await.map_err(SpotifyError::RequestError)?;

        // Cache the token (subtract 60 seconds for safety margin)
        let expires_at =
            Instant::now() + Duration::from_secs(token_response.expires_in.saturating_sub(60));
        let access_token = token_response.access_token.clone();

        *cache = Some(TokenCache {
            access_token: access_token.clone(),
            expires_at,
        });

        Ok(access_token)
    }

    /// Fetch track information from Spotify API
    /// Automatically handles authentication
    pub async fn get_track(&self, track_id: &str) -> Result<SpotifyTrackResponse, SpotifyError> {
        let access_token = self.get_access_token().await?;
        let url = format!("{}/tracks/{}", self.base_url, track_id);

        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", access_token))
            .send()
            .await
            .map_err(SpotifyError::RequestError)?;

        if !resp.status().is_success() {
            let status = resp.status();
            let error_text = resp.text().await.unwrap_or_default();
            return Err(SpotifyError::ApiError(format!(
                "Failed to fetch track: {} - {}",
                status, error_text
            )));
        }

        let mut track = resp
            .json::<SpotifyTrackResponse>()
            .await
            .map_err(SpotifyError::RequestError)?;

        let Ok(scraping_response) = self.client.get(&track.external_urls.spotify).send().await
        else {
            return Ok(track);
        };
        let Ok(html) = scraping_response.text().await else {
            return Ok(track);
        };

        let document = Html::parse_document(&html);

        let mut scdn_links = HashSet::new();

        for node in document.tree.nodes() {
            if let Some(elem) = node.value().as_element() {
                for (_k, v) in elem.attrs.iter() {
                    if v.contains("p.scdn.co") {
                        scdn_links.insert(v.to_string());
                    }
                }
            }
        }

        let scdn_links: Vec<String> = scdn_links.into_iter().collect();
        track.preview_url = Some(scdn_links.first().cloned().unwrap_or_default());
        Ok(track)
    }
}

/// Spotify API error types
#[derive(Debug)]
pub enum SpotifyError {
    RequestError(reqwest::Error),
    AuthenticationError(String),
    ApiError(String),
}

impl std::fmt::Display for SpotifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpotifyError::RequestError(e) => write!(f, "Request error: {}", e),
            SpotifyError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            SpotifyError::ApiError(msg) => write!(f, "API error: {}", msg),
        }
    }
}

impl std::error::Error for SpotifyError {}

/// OAuth2 token response from Spotify
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    // token_type: String,
    expires_in: u64,
}

/// Spotify API track response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpotifyTrackResponse {
    pub id: String,
    pub name: String,
    pub album: Album,
    pub artists: Vec<Artist>,
    pub duration_ms: u64,
    pub popularity: u32,
    pub external_urls: ExternalUrls,
    #[serde(default)]
    pub preview_url: Option<String>,
}

/// Album information from Spotify API
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub images: Vec<Image>,
    pub release_date: String,
    #[serde(default)]
    pub album_type: Option<String>,
}

/// Image information from Spotify API
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Image {
    pub url: String,
    pub height: Option<u32>,
    pub width: Option<u32>,
}

/// Artist information from Spotify API
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub external_urls: ExternalUrls,
}

/// External URLs from Spotify API
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExternalUrls {
    pub spotify: String,
}
