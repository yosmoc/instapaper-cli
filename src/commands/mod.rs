pub mod account;
pub mod auth;
pub mod bookmarks;
pub mod folders;
pub mod highlights;

#[cfg(test)]
pub const TEST_AUTH_HEADER: &str = "OAuth test";

#[cfg(test)]
pub fn test_token() -> oauth1_request::Token<String, String> {
    oauth1_request::Token::from_parts(
        "test-consumer-key".to_string(),
        "test-consumer-secret".to_string(),
        "test-token".to_string(),
        "test-token-secret".to_string(),
    )
}
