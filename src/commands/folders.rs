use crate::client::ApiClient;
use crate::commands::bookmarks::Folder;

/// Lists user-created folders.
///
/// # Errors
///
/// Returns an error if the API request fails, the response is not successful,
/// or the response cannot be parsed.
pub async fn list_folders(client: &ApiClient) -> Result<Vec<Folder>, Box<dyn std::error::Error>> {
    let response = client.signed_post("/api/1/folders/list", &[]).await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error: {status} - {body}").into());
    }

    let folders: Vec<Folder> = response.json().await?;
    Ok(folders)
}

/// Creates a new folder.
///
/// # Errors
///
/// Returns an error if the API request fails, the response is not successful,
/// or no folder is found in the response.
pub async fn add_folder(
    client: &ApiClient,
    title: &str,
) -> Result<Folder, Box<dyn std::error::Error>> {
    let params = [("title", title)];

    let response = client.signed_post("/api/1/folders/add", &params).await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error: {status} - {body}").into());
    }

    let items: Vec<serde_json::Value> = response.json().await?;
    for item in items {
        if item.get("type").and_then(|v| v.as_str()) == Some("folder") {
            return Ok(serde_json::from_value(item)?);
        }
    }

    Err("no folder found in response".into())
}

/// Deletes a folder.
///
/// # Errors
///
/// Returns an error if the API request fails or the response is not successful.
pub async fn delete_folder(
    client: &ApiClient,
    folder_id: i64,
) -> Result<(), Box<dyn std::error::Error>> {
    let folder_id_str = folder_id.to_string();
    let params = [("folder_id", folder_id_str.as_str())];

    let response = client.signed_post("/api/1/folders/delete", &params).await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error: {status} - {body}").into());
    }

    Ok(())
}

/// Re-orders folders.
///
/// # Errors
///
/// Returns an error if the API request fails, the response is not successful,
/// or the response cannot be parsed.
pub async fn set_folder_order(
    client: &ApiClient,
    order: &str,
) -> Result<Vec<Folder>, Box<dyn std::error::Error>> {
    let params = [("order", order)];

    let response = client
        .signed_post("/api/1/folders/set_order", &params)
        .await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error: {status} - {body}").into());
    }

    let folders: Vec<Folder> = response.json().await?;
    Ok(folders)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ApiClient;
    use crate::commands::test_token;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_list_folders_success() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/1/folders/list"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "type": "folder",
                    "folder_id": 100,
                    "title": "Work"
                },
                {
                    "type": "folder",
                    "folder_id": 200,
                    "title": "Personal"
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

        let folders = list_folders(&client).await?;
        assert_eq!(folders.len(), 2);
        assert_eq!(folders[0].title, "Work");
        Ok(())
    }

    #[tokio::test]
    async fn test_add_folder_success() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/1/folders/add"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "type": "folder",
                    "folder_id": 300,
                    "title": "New Folder"
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

        let folder = add_folder(&client, "New Folder").await?;
        assert_eq!(folder.folder_id, 300);
        assert_eq!(folder.title, "New Folder");
        Ok(())
    }

    #[tokio::test]
    async fn test_delete_folder_success() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/1/folders/delete"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
            .mount(&mock_server)
            .await;

        let client = ApiClient::new(
            mock_server.uri(),
            "test-consumer-key".to_string(),
            "test-consumer-secret".to_string(),
            Some(test_token()),
        )?;

        delete_folder(&client, 100).await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_set_folder_order_success() -> Result<(), Box<dyn std::error::Error>> {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/1/folders/set_order"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "type": "folder",
                    "folder_id": 200,
                    "title": "Personal"
                },
                {
                    "type": "folder",
                    "folder_id": 100,
                    "title": "Work"
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

        let folders = set_folder_order(&client, "200:1,100:2").await?;
        assert_eq!(folders[0].folder_id, 200);
        Ok(())
    }
}
