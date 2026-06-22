use crate::client::ApiClient;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    #[serde(rename = "type")]
    pub type_field: String,
    pub user_id: i64,
    pub username: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse {
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Verifies stored credentials and returns the current user.
///
/// # Errors
///
/// Returns an error if the API request fails, the response is not successful,
/// or no user object is present in the response.
pub async fn verify_credentials(client: &ApiClient) -> Result<User, Box<dyn std::error::Error>> {
    let response = client
        .signed_post("/api/1/account/verify_credentials", &[])
        .await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error: {status} - {body}").into());
    }

    let users: Vec<User> = response.json().await?;
    users
        .into_iter()
        .find(|u| u.type_field == "user")
        .ok_or_else(|| "no user found in response".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ApiClient;
    use crate::commands::test_token;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_verify_credentials_success() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/1/account/verify_credentials"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "type": "user",
                    "user_id": 54321,
                    "username": "TestUser"
                }
            ])))
            .mount(&mock_server)
            .await;

        let client = ApiClient::new(
            mock_server.uri(),
            "test-consumer-key".to_string(),
            "test-consumer-secret".to_string(),
            Some(test_token()),
        )?;

        let user = verify_credentials(&client).await?;
        assert_eq!(user.user_id, 54321);
        assert_eq!(user.username, "TestUser");
        Ok(())
    }

    #[tokio::test]
    async fn test_verify_credentials_unauthorized() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/1/account/verify_credentials"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!([
                {
                    "type": "error",
                    "error_code": 1040,
                    "message": "Rate-limit exceeded"
                }
            ])))
            .mount(&mock_server)
            .await;

        let client = ApiClient::new(
            mock_server.uri(),
            "test-consumer-key".to_string(),
            "test-consumer-secret".to_string(),
            Some(test_token()),
        )?;

        let result = verify_credentials(&client).await;
        assert!(result.is_err());
        Ok(())
    }
}
