use oauth1_request::{Builder, Credentials, HMAC_SHA1, HmacSha1, ParameterList, Token};
use reqwest::Client;
use std::time::Duration;
use thiserror::Error;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("INSTAPAPER_OAUTH_CONSUMER_KEY environment variable not set")]
    MissingConsumerKey,

    #[error("INSTAPAPER_OAUTH_CONSUMER_SECRET environment variable not set")]
    MissingConsumerSecret,

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("failed to build HTTP client: {0}")]
    HttpClientBuildError(reqwest::Error),

    #[error("OAuth error: {0}")]
    OAuthError(String),

    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub struct ApiClient {
    client: Client,
    base_url: String,
    consumer_key: String,
    consumer_secret: String,
    token: Option<Token<String, String>>,
}

impl ApiClient {
    /// Creates a new API client.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying HTTP client cannot be built.
    pub fn new(
        base_url: String,
        consumer_key: String,
        consumer_secret: String,
        token: Option<Token<String, String>>,
    ) -> Result<Self, ApiError> {
        let client = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(ApiError::HttpClientBuildError)?;

        Ok(Self {
            client,
            base_url,
            consumer_key,
            consumer_secret,
            token,
        })
    }

    /// Creates a client from environment variables and saved tokens.
    ///
    /// # Errors
    ///
    /// Returns an error if required environment variables are missing, the
    /// token file cannot be read, or the HTTP client cannot be built.
    pub fn from_env(base_url: String) -> Result<Self, ApiError> {
        let consumer_key = std::env::var("INSTAPAPER_OAUTH_CONSUMER_KEY")
            .map_err(|_| ApiError::MissingConsumerKey)?;
        let consumer_secret = std::env::var("INSTAPAPER_OAUTH_CONSUMER_SECRET")
            .map_err(|_| ApiError::MissingConsumerSecret)?;

        let token = Self::load_token_credentials(&consumer_key, &consumer_secret)?;

        Self::new(base_url, consumer_key, consumer_secret, token)
    }

    /// Creates a client from explicit OAuth credentials.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying HTTP client cannot be built.
    pub fn from_credentials(
        base_url: String,
        consumer_key: String,
        consumer_secret: String,
        access_token: String,
        access_token_secret: String,
    ) -> Result<Self, ApiError> {
        let token = Some(Token::from_parts(
            consumer_key.clone(),
            consumer_secret.clone(),
            access_token,
            access_token_secret,
        ));
        Self::new(base_url, consumer_key, consumer_secret, token)
    }

    fn config_dir() -> Result<std::path::PathBuf, ApiError> {
        let dir = dirs::config_dir()
            .ok_or_else(|| ApiError::ConfigError("could not determine config directory".into()))?
            .join("instapaper-cli");
        Ok(dir)
    }

    fn token_file() -> Result<std::path::PathBuf, ApiError> {
        Ok(Self::config_dir()?.join("token.json"))
    }

    fn load_token_credentials(
        consumer_key: &str,
        consumer_secret: &str,
    ) -> Result<Option<Token<String, String>>, ApiError> {
        let token_file = Self::token_file()?;
        if !token_file.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&token_file)
            .map_err(|e| ApiError::ConfigError(format!("failed to read token file: {e}")))?;

        let token_data: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| ApiError::ConfigError(format!("failed to parse token file: {e}")))?;

        let token = token_data
            .get("token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ApiError::ConfigError("missing token in token file".into()))?;

        let token_secret = token_data
            .get("token_secret")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ApiError::ConfigError("missing token_secret in token file".into()))?;

        Ok(Some(Token::from_parts(
            consumer_key.to_string(),
            consumer_secret.to_string(),
            token.to_string(),
            token_secret.to_string(),
        )))
    }

    /// Saves OAuth token credentials to the config file.
    ///
    /// # Errors
    ///
    /// Returns an error if the config directory or token file cannot be
    /// created or written, or if permissions cannot be restricted.
    pub fn save_token_credentials(&self, token: &str, token_secret: &str) -> Result<(), ApiError> {
        let dir = Self::config_dir()?;
        std::fs::create_dir_all(&dir).map_err(|e| {
            ApiError::ConfigError(format!("failed to create config directory: {e}"))
        })?;

        let token_file = Self::token_file()?;
        let content = serde_json::json!({
            "token": token,
            "token_secret": token_secret,
        });

        std::fs::write(&token_file, serde_json::to_string_pretty(&content)?)
            .map_err(|e| ApiError::ConfigError(format!("failed to write token file: {e}")))?;

        Self::restrict_token_file_permissions(&token_file)?;

        Ok(())
    }

    #[cfg(unix)]
    fn restrict_token_file_permissions(path: &std::path::Path) -> Result<(), ApiError> {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(path)
            .map_err(|e| ApiError::ConfigError(format!("failed to read token file metadata: {e}")))?
            .permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(path, permissions).map_err(|e| {
            ApiError::ConfigError(format!("failed to set token file permissions: {e}"))
        })?;
        Ok(())
    }

    #[cfg(not(unix))]
    fn restrict_token_file_permissions(_path: &std::path::Path) -> Result<(), ApiError> {
        Ok(())
    }

    #[must_use]
    pub fn token(&self) -> Option<&Token<String, String>> {
        self.token.as_ref()
    }

    /// Performs xAuth login and returns the access token pair.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, the response is not successful,
    /// or the expected OAuth fields are missing.
    pub async fn xauth_login(
        &self,
        username: &str,
        password: &str,
    ) -> Result<(String, String), ApiError> {
        let url = format!("{}/api/1/oauth/access_token", self.base_url);

        let params = ParameterList::new([
            ("x_auth_username", &username as &dyn std::fmt::Display),
            ("x_auth_password", &password as &dyn std::fmt::Display),
            ("x_auth_mode", &"client_auth" as &dyn std::fmt::Display),
        ]);

        let consumer_creds =
            Credentials::new(self.consumer_key.as_str(), self.consumer_secret.as_str());

        // For xAuth, we don't have a token yet, so use Builder without token
        let header_value: String =
            Builder::<_, &str, &str>::new(consumer_creds, HmacSha1::new()).post(&url, &params);

        let response = self
            .client
            .post(&url)
            .header("Authorization", &header_value)
            .form(&[
                ("x_auth_username", username),
                ("x_auth_password", password),
                ("x_auth_mode", "client_auth"),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::ApiError {
                status,
                message: body,
            });
        }

        let body = response.text().await?;
        let token = Self::parse_qline_field(&body, "oauth_token")
            .ok_or_else(|| ApiError::OAuthError("missing oauth_token in response".into()))?;
        let token_secret = Self::parse_qline_field(&body, "oauth_token_secret")
            .ok_or_else(|| ApiError::OAuthError("missing oauth_token_secret in response".into()))?;

        Ok((token, token_secret))
    }

    fn parse_qline_field(qline: &str, field: &str) -> Option<String> {
        for pair in qline.split('&') {
            let parts: Vec<&str> = pair.splitn(2, '=').collect();
            if parts.len() == 2 && parts[0] == field {
                return Some(urlencoding::decode(parts[1]).ok()?.into_owned());
            }
        }
        None
    }

    /// Sends a signed POST request to the given API path.
    ///
    /// # Errors
    ///
    /// Returns an error if no OAuth token is available, the request fails, or
    /// the response cannot be received.
    pub async fn signed_post(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<reqwest::Response, ApiError> {
        let url = format!("{}{}", self.base_url, path);

        let token = self.token.as_ref().ok_or_else(|| {
            ApiError::OAuthError("no OAuth token available. Run 'auth' command first.".into())
        })?;

        let param_list: Vec<(&str, &dyn std::fmt::Display)> = params
            .iter()
            .map(|(k, v)| (*k, v as &dyn std::fmt::Display))
            .collect();
        let request = ParameterList::new(param_list);

        let header_value = oauth1_request::post(&url, &request, token, HMAC_SHA1);

        let response = self
            .client
            .post(&url)
            .header("Authorization", &header_value)
            .form(params)
            .send()
            .await?;

        Ok(response)
    }

    /// Sends a signed GET request to the given API path.
    ///
    /// # Errors
    ///
    /// Returns an error if no OAuth token is available, the request fails, or
    /// the response cannot be received.
    pub async fn signed_get(
        &self,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<reqwest::Response, ApiError> {
        let url = format!("{}{}", self.base_url, path);

        let token = self.token.as_ref().ok_or_else(|| {
            ApiError::OAuthError("no OAuth token available. Run 'auth' command first.".into())
        })?;

        let param_list: Vec<(&str, &dyn std::fmt::Display)> = params
            .iter()
            .map(|(k, v)| (*k, v as &dyn std::fmt::Display))
            .collect();
        let request = ParameterList::new(param_list);

        let header_value = oauth1_request::get(&url, &request, token, HMAC_SHA1);

        let response = self
            .client
            .get(&url)
            .header("Authorization", &header_value)
            .query(params)
            .send()
            .await?;

        Ok(response)
    }

    #[must_use]
    pub fn client(&self) -> &Client {
        &self.client
    }

    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_qline_field() {
        let qline = "oauth_token=abc123&oauth_token_secret=def456";
        assert_eq!(
            ApiClient::parse_qline_field(qline, "oauth_token"),
            Some("abc123".to_string())
        );
        assert_eq!(
            ApiClient::parse_qline_field(qline, "oauth_token_secret"),
            Some("def456".to_string())
        );
        assert_eq!(ApiClient::parse_qline_field(qline, "missing"), None);
    }

    #[test]
    fn test_parse_qline_field_with_encoding() {
        let qline = "oauth_token=hello%20world&oauth_token_secret=test%26value";
        assert_eq!(
            ApiClient::parse_qline_field(qline, "oauth_token"),
            Some("hello world".to_string())
        );
        assert_eq!(
            ApiClient::parse_qline_field(qline, "oauth_token_secret"),
            Some("test&value".to_string())
        );
    }

    #[test]
    fn test_api_client_new_builds_successfully() -> Result<(), ApiError> {
        let _client = ApiClient::new(
            "https://www.instapaper.com".to_string(),
            "key".to_string(),
            "secret".to_string(),
            None,
        )?;
        Ok(())
    }
}
