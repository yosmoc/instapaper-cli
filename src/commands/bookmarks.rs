use crate::client::ApiClient;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Bookmark {
    #[serde(rename = "type")]
    pub type_field: String,
    pub bookmark_id: i64,
    pub url: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<Tag>>,
    #[serde(default)]
    pub hash: Option<String>,
    #[serde(default)]
    pub time: Option<i64>,
    #[serde(default)]
    pub starred: Option<String>,
    #[serde(default)]
    pub archive: Option<String>,
    #[serde(default)]
    pub private_source: Option<String>,
    #[serde(default)]
    pub progress: Option<f64>,
    #[serde(default)]
    pub progress_timestamp: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Highlight {
    #[serde(rename = "type")]
    pub type_field: String,
    pub highlight_id: i64,
    pub bookmark_id: i64,
    pub text: String,
    pub position: i64,
    pub time: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    #[serde(rename = "type")]
    pub type_field: String,
    pub user_id: i64,
    pub username: String,
    #[serde(default)]
    pub subscription_is_active: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct BookmarkListResponse {
    pub user: Option<User>,
    #[serde(default)]
    pub bookmarks: Vec<Bookmark>,
    #[serde(default)]
    pub highlights: Vec<Highlight>,
    #[serde(default)]
    pub delete_ids: Vec<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Folder {
    #[serde(rename = "type")]
    pub type_field: String,
    pub folder_id: i64,
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiError {
    #[serde(rename = "type")]
    pub type_field: String,
    pub error_code: i64,
    pub message: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ListItem {
    #[serde(rename = "user")]
    User {
        user_id: i64,
        username: String,
        #[serde(default)]
        subscription_is_active: Option<String>,
    },
    #[serde(rename = "bookmark")]
    Bookmark {
        bookmark_id: i64,
        url: String,
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        tags: Option<Vec<Tag>>,
        #[serde(default)]
        hash: Option<String>,
        #[serde(default)]
        time: Option<i64>,
        #[serde(default)]
        starred: Option<String>,
        #[serde(default)]
        archive: Option<String>,
        #[serde(default)]
        private_source: Option<String>,
        #[serde(default)]
        progress: Option<f64>,
        #[serde(default)]
        progress_timestamp: Option<i64>,
    },
    #[serde(rename = "highlight")]
    Highlight {
        highlight_id: i64,
        bookmark_id: i64,
        text: String,
        position: i64,
        time: i64,
    },
    #[serde(rename = "meta")]
    Meta,
    #[serde(rename = "error")]
    Error { error_code: i64, message: String },
    #[serde(other)]
    Unknown,
}

/// Lists bookmarks matching the provided filters.
///
/// # Errors
///
/// Returns an error if the API request fails, the response is not successful,
/// or the response cannot be parsed.
pub async fn list_bookmarks(
    client: &ApiClient,
    limit: Option<i32>,
    folder_id: Option<&str>,
    tag: Option<&str>,
    have: Option<&str>,
    highlights: Option<&str>,
) -> Result<BookmarkListResponse, Box<dyn std::error::Error>> {
    let limit_str = limit.map(|l| l.to_string());
    let params: Vec<(&str, &str)> = [
        ("limit", limit_str.as_deref()),
        ("folder_id", folder_id),
        ("tag", tag),
        ("have", have),
        ("highlights", highlights),
    ]
    .into_iter()
    .filter_map(|(k, v)| v.map(|val| (k, val)))
    .collect();

    let response = client.signed_get("/api/1/bookmarks/list", &params).await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error: {status} - {body}").into());
    }

    let body = response.text().await?;

    // The API returns an array of typed objects: [{"type": "meta"}, {"type": "user", ...}, {"type": "bookmark", ...}]
    let items: Vec<ListItem> = serde_json::from_str(&body)?;

    let mut result = BookmarkListResponse::default();
    for item in items {
        match item {
            ListItem::User {
                user_id,
                username,
                subscription_is_active,
            } => {
                result.user = Some(User {
                    type_field: "user".to_string(),
                    user_id,
                    username,
                    subscription_is_active,
                });
            }
            ListItem::Bookmark {
                bookmark_id,
                url,
                title,
                description,
                tags,
                hash,
                time,
                starred,
                archive,
                private_source,
                progress,
                progress_timestamp,
            } => {
                result.bookmarks.push(Bookmark {
                    type_field: "bookmark".to_string(),
                    bookmark_id,
                    url,
                    title,
                    description,
                    tags,
                    hash,
                    time,
                    starred,
                    archive,
                    private_source,
                    progress,
                    progress_timestamp,
                });
            }
            ListItem::Highlight {
                highlight_id,
                bookmark_id,
                text,
                position,
                time,
            } => {
                result.highlights.push(Highlight {
                    type_field: "highlight".to_string(),
                    highlight_id,
                    bookmark_id,
                    text,
                    position,
                    time,
                });
            }
            ListItem::Error {
                error_code,
                message,
            } => {
                return Err(format!("API error {error_code}: {message}").into());
            }
            _ => {}
        }
    }

    Ok(result)
}

/// Adds a new bookmark.
///
/// # Errors
///
/// Returns an error if the API request fails, the response is not successful,
/// or the response cannot be parsed.
#[expect(clippy::too_many_arguments)]
pub async fn add_bookmark(
    client: &ApiClient,
    url: &str,
    title: Option<&str>,
    description: Option<&str>,
    folder_id: Option<i64>,
    archived: Option<bool>,
    tags: Option<&str>,
    content: Option<&str>,
    is_private_from_source: Option<&str>,
    resolve_final_url: Option<bool>,
) -> Result<Bookmark, Box<dyn std::error::Error>> {
    let folder_id_str = folder_id.map(|f| f.to_string());
    let archived_str = archived.map(|a| if a { "1" } else { "0" });
    let resolve_str = resolve_final_url.map(|r| if r { "1" } else { "0" });

    let params: Vec<(&str, &str)> = [
        ("url", Some(url)),
        ("title", title),
        ("description", description),
        ("folder_id", folder_id_str.as_deref()),
        ("archived", archived_str),
        ("tags", tags),
        ("content", content),
        ("is_private_from_source", is_private_from_source),
        ("resolve_final_url", resolve_str),
    ]
    .into_iter()
    .filter_map(|(k, v)| v.map(|val| (k, val)))
    .collect();

    let response = client.signed_post("/api/1/bookmarks/add", &params).await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error: {status} - {body}").into());
    }

    let body = response.text().await?;
    let items: Vec<serde_json::Value> = serde_json::from_str(&body)?;
    for item in items {
        if item.get("type").and_then(|v| v.as_str()) == Some("bookmark") {
            return Ok(serde_json::from_value(item)?);
        }
        if item.get("type").and_then(|v| v.as_str()) == Some("error") {
            let ApiError {
                error_code,
                message,
                ..
            } = serde_json::from_value(item)?;
            return Err(format!("API error {error_code}: {message}").into());
        }
    }

    Err("no bookmark found in response".into())
}

/// Deletes a bookmark permanently.
///
/// # Errors
///
/// Returns an error if the API request fails or the response is not successful.
pub async fn delete_bookmark(
    client: &ApiClient,
    bookmark_id: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    let bookmark_id_str = bookmark_id.to_string();
    let params = [("bookmark_id", bookmark_id_str.as_str())];

    let response = client
        .signed_post("/api/1/bookmarks/delete", &params)
        .await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error: {status} - {body}").into());
    }

    Ok(())
}

/// Stars a bookmark.
///
/// # Errors
///
/// Returns an error if the API request fails, the response is not successful,
/// or no bookmark is found in the response.
pub async fn star_bookmark(
    client: &ApiClient,
    bookmark_id: i64,
) -> Result<Bookmark, Box<dyn std::error::Error>> {
    let bookmark_id_str = bookmark_id.to_string();
    let params = [("bookmark_id", bookmark_id_str.as_str())];

    let response = client.signed_post("/api/1/bookmarks/star", &params).await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error: {status} - {body}").into());
    }

    let body = response.text().await?;
    let items: Vec<serde_json::Value> = serde_json::from_str(&body)?;
    for item in items {
        if item.get("type").and_then(|v| v.as_str()) == Some("bookmark") {
            return Ok(serde_json::from_value(item)?);
        }
    }

    Err("no bookmark found in response".into())
}

/// Unstars a bookmark.
///
/// # Errors
///
/// Returns an error if the API request fails, the response is not successful,
/// or no bookmark is found in the response.
pub async fn unstar_bookmark(
    client: &ApiClient,
    bookmark_id: i64,
) -> Result<Bookmark, Box<dyn std::error::Error>> {
    let bookmark_id_str = bookmark_id.to_string();
    let params = [("bookmark_id", bookmark_id_str.as_str())];

    let response = client
        .signed_post("/api/1/bookmarks/unstar", &params)
        .await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error: {status} - {body}").into());
    }

    let body = response.text().await?;
    let items: Vec<serde_json::Value> = serde_json::from_str(&body)?;
    for item in items {
        if item.get("type").and_then(|v| v.as_str()) == Some("bookmark") {
            return Ok(serde_json::from_value(item)?);
        }
    }

    Err("no bookmark found in response".into())
}

/// Archives a bookmark.
///
/// # Errors
///
/// Returns an error if the API request fails, the response is not successful,
/// or no bookmark is found in the response.
pub async fn archive_bookmark(
    client: &ApiClient,
    bookmark_id: i64,
) -> Result<Bookmark, Box<dyn std::error::Error>> {
    let bookmark_id_str = bookmark_id.to_string();
    let params = [("bookmark_id", bookmark_id_str.as_str())];

    let response = client
        .signed_post("/api/1/bookmarks/archive", &params)
        .await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error: {status} - {body}").into());
    }

    let body = response.text().await?;
    let items: Vec<serde_json::Value> = serde_json::from_str(&body)?;
    for item in items {
        if item.get("type").and_then(|v| v.as_str()) == Some("bookmark") {
            return Ok(serde_json::from_value(item)?);
        }
    }

    Err("no bookmark found in response".into())
}

/// Unarchives a bookmark.
///
/// # Errors
///
/// Returns an error if the API request fails, the response is not successful,
/// or no bookmark is found in the response.
pub async fn unarchive_bookmark(
    client: &ApiClient,
    bookmark_id: i64,
) -> Result<Bookmark, Box<dyn std::error::Error>> {
    let bookmark_id_str = bookmark_id.to_string();
    let params = [("bookmark_id", bookmark_id_str.as_str())];

    let response = client
        .signed_post("/api/1/bookmarks/unarchive", &params)
        .await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error: {status} - {body}").into());
    }

    let body = response.text().await?;
    let items: Vec<serde_json::Value> = serde_json::from_str(&body)?;
    for item in items {
        if item.get("type").and_then(|v| v.as_str()) == Some("bookmark") {
            return Ok(serde_json::from_value(item)?);
        }
    }

    Err("no bookmark found in response".into())
}

/// Moves a bookmark to a folder.
///
/// # Errors
///
/// Returns an error if the API request fails, the response is not successful,
/// or no bookmark is found in the response.
pub async fn move_bookmark(
    client: &ApiClient,
    bookmark_id: i64,
    folder_id: i64,
) -> Result<Bookmark, Box<dyn std::error::Error>> {
    let bookmark_id_str = bookmark_id.to_string();
    let folder_id_str = folder_id.to_string();
    let params = [
        ("bookmark_id", bookmark_id_str.as_str()),
        ("folder_id", folder_id_str.as_str()),
    ];

    let response = client.signed_post("/api/1/bookmarks/move", &params).await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error: {status} - {body}").into());
    }

    let body = response.text().await?;
    let items: Vec<serde_json::Value> = serde_json::from_str(&body)?;
    for item in items {
        if item.get("type").and_then(|v| v.as_str()) == Some("bookmark") {
            return Ok(serde_json::from_value(item)?);
        }
    }

    Err("no bookmark found in response".into())
}

/// Fetches the processed text HTML for a bookmark.
///
/// # Errors
///
/// Returns an error if the API request fails or the response is not successful.
pub async fn get_bookmark_text(
    client: &ApiClient,
    bookmark_id: i64,
    instaparser_api_key: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let bookmark_id_str = bookmark_id.to_string();
    let params: Vec<(&str, &str)> = [
        ("bookmark_id", bookmark_id_str.as_str()),
        ("instaparser_api_key", instaparser_api_key.unwrap_or("")),
    ]
    .into_iter()
    .filter(|(_, v)| !v.is_empty())
    .collect();

    let response = client
        .signed_post("/api/1/bookmarks/get_text", &params)
        .await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error: {status} - {body}").into());
    }

    let html = response.text().await?;
    Ok(html)
}

/// Updates reading progress for a bookmark.
///
/// # Errors
///
/// Returns an error if the API request fails, the response is not successful,
/// or no bookmark is found in the response.
pub async fn update_read_progress(
    client: &ApiClient,
    bookmark_id: i64,
    progress: f64,
    progress_timestamp: i64,
) -> Result<Bookmark, Box<dyn std::error::Error>> {
    let bookmark_id_str = bookmark_id.to_string();
    let progress_str = progress.to_string();
    let timestamp_str = progress_timestamp.to_string();
    let params = [
        ("bookmark_id", bookmark_id_str.as_str()),
        ("progress", progress_str.as_str()),
        ("progress_timestamp", timestamp_str.as_str()),
    ];

    let response = client
        .signed_post("/api/1/bookmarks/update_read_progress", &params)
        .await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error: {status} - {body}").into());
    }

    let body = response.text().await?;
    let items: Vec<serde_json::Value> = serde_json::from_str(&body)?;
    for item in items {
        if item.get("type").and_then(|v| v.as_str()) == Some("bookmark") {
            return Ok(serde_json::from_value(item)?);
        }
    }

    Err("no bookmark found in response".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ApiClient;
    use crate::commands::test_token;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_list_bookmarks_success() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/1/bookmarks/list"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"type": "meta"},
                {
                    "type": "user",
                    "user_id": 54321,
                    "username": "TestUser"
                },
                {
                    "type": "bookmark",
                    "bookmark_id": 1234,
                    "url": "http://www.example.com/page1.html",
                    "title": "Example page 1",
                    "description": "An example page.",
                    "hash": "abc123",
                    "time": 1_234_567_890,
                    "starred": "0",
                    "archive": "0"
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

        let response = list_bookmarks(&client, Some(10), None, None, None, None).await?;
        assert_eq!(response.bookmarks.len(), 1);
        assert_eq!(response.bookmarks[0].bookmark_id, 1234);
        Ok(())
    }

    #[tokio::test]
    async fn test_add_bookmark_success() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/1/bookmarks/add"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "type": "bookmark",
                    "bookmark_id": 1235,
                    "url": "http://www.example.com/new.html",
                    "title": "New Page",
                    "description": "A new page.",
                    "hash": "def456",
                    "time": 1_234_567_891,
                    "starred": "0",
                    "archive": "0"
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

        let bookmark = add_bookmark(
            &client,
            "http://www.example.com/new.html",
            Some("New Page"),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await?;
        assert_eq!(bookmark.bookmark_id, 1235);
        assert_eq!(bookmark.title, Some("New Page".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn test_get_bookmark_text_success() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/1/bookmarks/get_text"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string("<html><body>Article text</body></html>"),
            )
            .mount(&mock_server)
            .await;

        let client = ApiClient::new(
            mock_server.uri(),
            "test-consumer-key".to_string(),
            "test-consumer-secret".to_string(),
            Some(test_token()),
        )?;

        let html = get_bookmark_text(&client, 1234, None).await?;
        assert_eq!(html, "<html><body>Article text</body></html>");
        Ok(())
    }

    #[tokio::test]
    async fn test_star_bookmark_success() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/1/bookmarks/star"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "type": "bookmark",
                    "bookmark_id": 1234,
                    "url": "http://www.example.com/page1.html",
                    "title": "Example page 1",
                    "starred": "1"
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

        let bookmark = star_bookmark(&client, 1234).await?;
        assert_eq!(bookmark.starred, Some("1".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn test_archive_bookmark_success() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/1/bookmarks/archive"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "type": "bookmark",
                    "bookmark_id": 1234,
                    "url": "http://www.example.com/page1.html",
                    "title": "Example page 1",
                    "archive": "1"
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

        let bookmark = archive_bookmark(&client, 1234).await?;
        assert_eq!(bookmark.archive, Some("1".to_string()));
        Ok(())
    }

    #[tokio::test]
    async fn test_update_read_progress_success() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/1/bookmarks/update_read_progress"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "type": "bookmark",
                    "bookmark_id": 1234,
                    "url": "http://www.example.com/page1.html",
                    "title": "Example page 1",
                    "hash": "newhash123"
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

        let bookmark = update_read_progress(&client, 1234, 0.5, 1_288_584_076).await?;
        assert_eq!(bookmark.bookmark_id, 1234);
        Ok(())
    }
}
