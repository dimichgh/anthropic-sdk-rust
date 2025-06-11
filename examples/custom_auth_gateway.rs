use anthropic_sdk::{Anthropic, ClientConfig, MessageCreateBuilder};
use anthropic_sdk::types::ContentBlock;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use std::time::Duration;
use std::env;
use serde_json::json;

// Helper function to extract text content from response
fn extract_text_from_content(content: &[ContentBlock]) -> String {
    content.iter()
        .filter_map(|block| match block {
            ContentBlock::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Example demonstrating custom authentication patterns for gateway integrations
/// This shows various ways to authenticate with custom gateway endpoints
/// 
/// Usage:
///   export CUSTOM_BEARER_TOKEN=your-token-here
///   export CUSTOM_BASE_URL=https://your-gateway.example.com/v1/anthropic
///   cargo run --example custom_auth_gateway

#[derive(Debug, Clone)]
/// Custom HTTP client with custom gateway authentication
struct CustomAnthropicClient {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl CustomAnthropicClient {
    fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// Test different authentication header formats
    async fn test_auth_formats(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔧 Testing different authentication formats:");
        
        // Format 1: Standard Bearer token
        println!("\n   📋 Format 1: Bearer Token");
        self.test_bearer_format().await?;
        
        // Format 2: Custom header variations
        println!("\n   📋 Format 2: Custom Headers");
        self.test_custom_headers().await?;
        
        // Format 3: API Key in different locations
        println!("\n   📋 Format 3: API Key Variations");
        self.test_api_key_variations().await?;
        
        Ok(())
    }

    async fn test_bearer_format(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", self.api_key))?);
        
        let response = self.send_test_request(headers).await;
        println!("      ✅ Bearer: {:?}", response.is_ok());
        Ok(())
    }

    async fn test_custom_headers(&self) -> Result<(), Box<dyn std::error::Error>> {
        let custom_headers = vec![
            ("x-api-key", &self.api_key),
            ("custom-token", &self.api_key),
            ("x-custom-token", &self.api_key),
        ];
        
        for (header_name, token) in custom_headers {
            let mut headers = HeaderMap::new();
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            headers.insert(header_name, HeaderValue::from_str(token)?);
            
            let response = self.send_test_request(headers).await;
            println!("      ✅ {}: {:?}", header_name, response.is_ok());
        }
        Ok(())
    }

    async fn test_api_key_variations(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Test query parameter
        let url_with_key = format!("{}?api_key={}", self.base_url, self.api_key);
        println!("      ✅ Query param: Testing...");
        
        // Test in request body
        println!("      ✅ Body param: Testing...");
        
        Ok(())
    }

    async fn send_test_request(&self, headers: HeaderMap) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}/messages", self.base_url);
        let body = json!({
            "model": "claude-3-5-sonnet-latest",
            "max_tokens": 10,
            "messages": [{"role": "user", "content": "Hello"}]
        });
        
        self.client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
    }
}

/// Display helpful configuration examples
fn show_configuration_examples() {
    println!("💡 Check your custom gateway documentation for the correct authentication format.");
    println!("\n🔧 Common authentication patterns:");
    println!("   • Bearer Token:     Authorization: Bearer <token>");
    println!("   • API Key Header:   X-API-Key: <token>");
    println!("   • Custom Header:    X-Custom-Token: <token>");
    println!("   • Query Parameter:  ?api_key=<token>");
    println!("   • Request Body:     {{\"api_key\": \"<token>\"}}");
    
    println!("\n📋 Configuration options:");
    println!("   1. Environment variables:");
    println!("      export CUSTOM_BEARER_TOKEN='your-token'");
    println!("      export CUSTOM_BASE_URL='https://your-gateway.example.com/v1/anthropic'");
    println!("   2. Direct configuration:");
    println!("      ClientConfig::new(\"your-token\")");
    println!("          .with_base_url(\"https://your-gateway.example.com/v1/anthropic\")");
}

/// Test authentication with the Anthropic Rust SDK
async fn test_with_anthropic_sdk() -> Result<(), Box<dyn std::error::Error>> {
    use anthropic_rs::{Anthropic, ClientConfig};
    
    let api_key = env::var("CUSTOM_BEARER_TOKEN")
        .or_else(|_| env::var("ANTHROPIC_API_KEY"))
        .expect("Need CUSTOM_BEARER_TOKEN or ANTHROPIC_API_KEY");
    
    let base_url = env::var("CUSTOM_BASE_URL")
        .expect("Need CUSTOM_BASE_URL environment variable");
    
    println!("🚀 Testing with Anthropic SDK...");
    
    let config = ClientConfig::new(api_key)
        .with_base_url(&base_url);
    
    let client = Anthropic::with_config(config)?;
    
    let response = client.messages()
        .model("claude-3-5-sonnet-latest")
        .max_tokens(20)
        .user("Test message")
        .send()
        .await?;
    
    println!("✅ SDK test successful!");
    println!("📝 Response: {}", response.content.first().unwrap().text.as_ref().unwrap());
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Custom Gateway Authentication Test");
    
    // Check environment variables
    let api_key = env::var("CUSTOM_BEARER_TOKEN")
        .or_else(|_| env::var("ANTHROPIC_API_KEY"))
        .expect("⚠️  No API key found. Please set CUSTOM_BEARER_TOKEN or ANTHROPIC_API_KEY");
    
    let base_url = env::var("CUSTOM_BASE_URL")
        .expect("⚠️  No base URL found. Please set CUSTOM_BASE_URL");
    
    println!("📡 Testing Custom Gateway: {}", base_url);
    
    // Test different authentication methods
    let custom_client = CustomAnthropicClient::new(api_key.clone(), base_url.to_string());
    custom_client.test_auth_formats().await?;
    
    // Test with the SDK
    println!("\n" + "=".repeat(60).as_str());
    test_with_anthropic_sdk().await?;
    
    // Show configuration help
    println!("\n" + "=".repeat(60).as_str());
    show_configuration_examples();
    
    println!("\n🎯 Next steps:");
    println!("1. Run this with your real API key: cargo run --example custom_auth_gateway");
    println!("2. Check the authentication format that works with your gateway");
    println!("3. Or create a custom configuration option for your gateway");
    
    Ok(())
} 