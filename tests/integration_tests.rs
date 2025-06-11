use anthropic_sdk::{Anthropic, ClientConfig, LogLevel};
use std::time::Duration;

#[tokio::test]
async fn test_client_creation_with_api_key() {
    let client = Anthropic::new("test-api-key").expect("Should create client");
    assert_eq!(client.config().api_key, "test-api-key");
    assert_eq!(client.config().base_url, "https://api.anthropic.com");
}

#[tokio::test]
async fn test_client_creation_with_config() {
    let config = ClientConfig::new("test-key")
        .with_timeout(Duration::from_secs(30))
        .with_max_retries(3)
        .with_log_level(LogLevel::Debug)
        .with_base_url("https://custom.api.com");
    
    let client = Anthropic::with_config(config).expect("Should create client");
    assert_eq!(client.config().timeout, Duration::from_secs(30));
    assert_eq!(client.config().max_retries, 3);
    assert_eq!(client.config().base_url, "https://custom.api.com");
}

#[tokio::test]
async fn test_config_validation() {
    let config = ClientConfig::new("");
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("API key cannot be empty"));
}

#[tokio::test]
async fn test_config_with_invalid_url() {
    let config = ClientConfig::new("test-key").with_base_url("invalid-url");
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Base URL must start with http"));
}

#[tokio::test]
async fn test_client_test_connection() {
    let client = Anthropic::new("test-api-key").expect("Should create client");
    // This should pass validation since we have a valid config
    let result = client.test_connection().await;
    assert!(result.is_ok());
} 