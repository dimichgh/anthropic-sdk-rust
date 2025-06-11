use anthropic_sdk::{Anthropic, ClientConfig, MessageCreateBuilder};
use std::time::Duration;
use reqwest::header::{HeaderMap, HeaderValue};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("CUSTOM_BEARER_TOKEN")
        .or_else(|_| env::var("ANTHROPIC_API_KEY"))
        .expect("⚠️  No API key found. Please set CUSTOM_BEARER_TOKEN or ANTHROPIC_API_KEY");
    
    let base_url = env::var("CUSTOM_BASE_URL")
        .expect("⚠️  No base URL found. Please set CUSTOM_BASE_URL");

    println!("📋 Testing Authentication Headers with Custom Gateway\n");
    println!("📡 Base URL: {}", base_url);
    println!("🔑 API Key: {}...{}", &api_key[..4.min(api_key.len())], &api_key[api_key.len()-4.min(api_key.len())..]);

    // Test 1: Bearer token (most common)
    println!("\n🧪 Test 1: Bearer Token");
    test_bearer_token(&api_key, &base_url).await?;

    // Test 2: x-api-key header
    println!("\n🧪 Test 2: X-API-Key Header");
    test_api_key_header(&api_key, &base_url).await?;

    // Test 3: Authorization header (direct)
    println!("\n🧪 Test 3: Authorization Header (Direct)");
    test_direct_auth_header(&api_key, &base_url).await?;

    // Test 4: Custom headers
    println!("\n🧪 Test 4: Custom Gateway Headers");
    test_custom_gateway_headers(&api_key, &base_url).await?;

    println!("\n📋 Summary:");
    println!("1. Check your custom gateway API documentation for required headers");
    println!("2. Common patterns: Bearer token, x-api-key, or custom headers");
    println!("3. Most gateways use Bearer token format");

    Ok(())
}

async fn test_bearer_token(api_key: &str, base_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!("{}/messages", base_url);

    let payload = serde_json::json!({
        "model": "claude-3-5-sonnet-latest",
        "max_tokens": 20,
        "messages": [{"role": "user", "content": "Hello Bearer!"}]
    });

    println!("   Testing Bearer token format...");
    
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .header("anthropic-version", "2023-06-01")
        .json(&payload)
        .send()
        .await?;

    if response.status().is_success() {
        println!("   ✅ Bearer token works!");
        let json: serde_json::Value = response.json().await?;
        if let Some(content) = json.get("content").and_then(|c| c.as_array()).and_then(|a| a.first()) {
            if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
                println!("   📝 Response: {}", text);
            }
        }
    } else {
        println!("   ❌ Bearer token failed: HTTP {}", response.status());
    }
    
    Ok(())
}

async fn test_api_key_header(api_key: &str, base_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!("{}/messages", base_url);

    let payload = serde_json::json!({
        "model": "claude-3-5-sonnet-latest",
        "max_tokens": 20,
        "messages": [{"role": "user", "content": "Hello API Key!"}]
    });

    println!("   Testing x-api-key header format...");

    let response = client
        .post(&url)
        .header("x-api-key", api_key)
        .header("Content-Type", "application/json")
        .header("anthropic-version", "2023-06-01")
        .json(&payload)
        .send()
        .await?;

    if response.status().is_success() {
        println!("   ✅ X-API-Key header works!");
        let json: serde_json::Value = response.json().await?;
        if let Some(content) = json.get("content").and_then(|c| c.as_array()).and_then(|a| a.first()) {
            if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
                println!("   📝 Response: {}", text);
            }
        }
    } else {
        println!("   ❌ X-API-Key header failed: HTTP {}", response.status());
    }

    Ok(())
}

async fn test_direct_auth_header(api_key: &str, base_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url = format!("{}/messages", base_url);

    let payload = serde_json::json!({
        "model": "claude-3-5-sonnet-latest",
        "max_tokens": 20,
        "messages": [{"role": "user", "content": "Hello Auth!"}]
    });

    println!("   Testing Authorization header (direct token)...");

    let response = client
        .post(&url)
        .header("Authorization", api_key)
        .header("Content-Type", "application/json")
        .header("anthropic-version", "2023-06-01")
        .json(&payload)
        .send()
        .await?;

    if response.status().is_success() {
        println!("   ✅ Direct Authorization header works!");
        let json: serde_json::Value = response.json().await?;
        if let Some(content) = json.get("content").and_then(|c| c.as_array()).and_then(|a| a.first()) {
            if let Some(text) = content.get("text").and_then(|t| t.as_str()) {
                println!("   📝 Response: {}", text);
            }
        }
    } else {
        println!("   ❌ Direct Authorization header failed: HTTP {}", response.status());
    }

    Ok(())
}

async fn test_custom_gateway_headers(api_key: &str, base_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("   Testing custom gateway headers:");
    
    let client = reqwest::Client::new();
    let url = format!("{}/messages", base_url);

    let payload = serde_json::json!({
        "model": "claude-3-5-sonnet-latest", 
        "max_tokens": 20,
        "messages": [{"role": "user", "content": "Hello Custom!"}]
    });

    let custom_headers = vec![
        ("token", api_key),
        ("custom-token", api_key),
        ("x-custom-token", api_key),
        ("gateway-token", api_key),
        ("x-gateway-token", api_key),
    ];

    for (header_name, token) in custom_headers {
        let response = client
            .post(&url)
            .header(header_name, token)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            println!("   ✅ {} header works!", header_name);
            return Ok(());
        } else {
            println!("   ❌ {} header failed: HTTP {}", header_name, response.status());
        }
    }

    Ok(())
}

fn mask_key(key: &str) -> String {
    if key.len() <= 8 {
        "*".repeat(key.len())
    } else {
        format!("{}...{}", &key[..4], &key[key.len()-4..])
    }
} 