use crate::client::ApiClient;
use crate::commands::bookmarks::Highlight;

/// Lists highlights for a bookmark.
///
/// # Errors
///
/// Returns an error if the API request fails, the response is not successful,
/// or the response cannot be parsed.
pub async fn list_highlights(
    client: &ApiClient,
    bookmark_id: i64,
) -> Result<Vec<Highlight>, Box<dyn std::error::Error>> {
    let path = format!("/api/1.1/bookmarks/{bookmark_id}/highlights");
    let response = client.signed_get(&path, &[]).await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error: {status} - {body}").into());
    }

    let highlights: Vec<Highlight> = response.json().await?;
    Ok(highlights)
}

/// Creates a new highlight.
///
/// # Errors
///
/// Returns an error if the API request fails, the response is not successful,
/// or the response cannot be parsed.
pub async fn create_highlight(
    client: &ApiClient,
    bookmark_id: i64,
    text: &str,
    position: Option<i64>,
) -> Result<Highlight, Box<dyn std::error::Error>> {
    let path = format!("/api/1.1/bookmarks/{bookmark_id}/highlight");

    let position_str = position.map(|p| p.to_string());
    let params: Vec<(&str, &str)> = [
        ("text", text),
        ("position", position_str.as_deref().unwrap_or("")),
    ]
    .into_iter()
    .filter(|(_, v)| !v.is_empty())
    .collect();

    let response = client.signed_post(&path, &params).await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error: {status} - {body}").into());
    }

    let highlight: Highlight = response.json().await?;
    Ok(highlight)
}

/// Deletes a highlight.
///
/// # Errors
///
/// Returns an error if the API request fails or the response is not successful.
pub async fn delete_highlight(
    client: &ApiClient,
    highlight_id: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = format!("/api/1.1/highlights/{highlight_id}/delete");
    let response = client.signed_post(&path, &[]).await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error: {status} - {body}").into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ApiClient;
    use crate::commands::test_token;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_list_highlights_success() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/1.1/bookmarks/1234/highlights"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "type": "highlight",
                    "highlight_id": 42,
                    "bookmark_id": 1234,
                    "text": "example page",
                    "position": 0,
                    "time": 1_394_470_555
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

        let highlights = list_highlights(&client, 1234).await?;
        assert_eq!(highlights.len(), 1);
        assert_eq!(highlights[0].highlight_id, 42);
        assert_eq!(highlights[0].text, "example page");
        Ok(())
    }

    #[tokio::test]
    async fn test_create_highlight_success() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/1.1/bookmarks/1234/highlight"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "type": "highlight",
                "highlight_id": 43,
                "bookmark_id": 1234,
                "text": "new highlight",
                "position": 100,
                "time": 1_394_470_600
            })))
            .mount(&mock_server)
            .await;

        let client = ApiClient::new(
            mock_server.uri(),
            "test-consumer-key".to_string(),
            "test-consumer-secret".to_string(),
            Some(test_token()),
        )?;

        let highlight = create_highlight(&client, 1234, "new highlight", Some(100)).await?;
        assert_eq!(highlight.highlight_id, 43);
        assert_eq!(highlight.text, "new highlight");
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_highlight_success() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/1.1/highlights/42/delete"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!(null)))
            .mount(&mock_server)
            .await;

        let client = ApiClient::new(
            mock_server.uri(),
            "test-consumer-key".to_string(),
            "test-consumer-secret".to_string(),
            Some(test_token()),
        )?;

        delete_highlight(&client, 42).await?;
        Ok(())
    }
}
