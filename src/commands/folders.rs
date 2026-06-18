use crate::client::ApiClient;
use crate::commands::bookmarks::Folder;

pub async fn list_folders(client: &ApiClient) -> Result<Vec<Folder>, Box<dyn std::error::Error>> {
    let response = client.signed_post("/api/1/folders/list", &[]).await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error: {} - {}", status, body).into());
    }

    let folders: Vec<Folder> = response.json().await?;
    Ok(folders)
}

pub async fn add_folder(
    client: &ApiClient,
    title: &str,
) -> Result<Folder, Box<dyn std::error::Error>> {
    let params = [("title", title)];

    let response = client.signed_post("/api/1/folders/add", &params).await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("API error: {} - {}", status, body).into());
    }

    let items: Vec<serde_json::Value> = response.json().await?;
    for item in items {
        if item.get("type").and_then(|v| v.as_str()) == Some("folder") {
            return Ok(serde_json::from_value(item)?);
        }
    }

    Err("no folder found in response".into())
}

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
        return Err(format!("API error: {} - {}", status, body).into());
    }

    Ok(())
}

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
        return Err(format!("API error: {} - {}", status, body).into());
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
    async fn test_list_folders_success() {
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
        );

        let result = list_folders(&client).await;
        assert!(result.is_ok());
        let folders = result.unwrap();
        assert_eq!(folders.len(), 2);
        assert_eq!(folders[0].title, "Work");
    }

    #[tokio::test]
    async fn test_add_folder_success() {
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
        );

        let result = add_folder(&client, "New Folder").await;
        assert!(result.is_ok());
        let folder = result.unwrap();
        assert_eq!(folder.folder_id, 300);
        assert_eq!(folder.title, "New Folder");
    }

    #[tokio::test]
    async fn test_delete_folder_success() {
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
        );

        let result = delete_folder(&client, 100).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_folder_order_success() {
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
        );

        let result = set_folder_order(&client, "200:1,100:2").await;
        assert!(result.is_ok());
        let folders = result.unwrap();
        assert_eq!(folders[0].folder_id, 200);
    }
}
