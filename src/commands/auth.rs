use crate::client::ApiClient;

pub async fn xauth_login(
    client: &ApiClient,
    username: &str,
    password: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let (token, token_secret) = client.xauth_login(username, password).await?;
    client.save_token_credentials(&token, &token_secret)?;
    Ok((token, token_secret))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ApiClient;

    #[test]
    fn test_xauth_login_requires_credentials() {
        let client = ApiClient::new(
            "https://www.instapaper.com".to_string(),
            "key".to_string(),
            "secret".to_string(),
            None,
        );

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(xauth_login(&client, "", ""));
        assert!(result.is_err());
    }
}
