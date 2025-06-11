use anthropic_sdk::{Anthropic, ClientConfig, MessageCreateBuilder};
use anthropic_sdk::types::ContentBlock;
use std::time::Duration;
use std::env;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing Anthropic SDK with Custom Gateway...\n");
    
    // Get API key from environment
    let api_key = env::var("CUSTOM_BEARER_TOKEN")
        .or_else(|_| env::var("ANTHROPIC_API_KEY"))
        .expect("⚠️  No API key found. Please set CUSTOM_BEARER_TOKEN or ANTHROPIC_API_KEY");
    
    let base_url = env::var("CUSTOM_BASE_URL")
        .expect("⚠️  No base URL found. Please set CUSTOM_BASE_URL");
    
    // Custom configuration for custom gateway
    let config = ClientConfig::new(&api_key)
        .with_base_url(&base_url)
        .with_timeout(Duration::from_secs(30));
    
    let client = Anthropic::with_config(config)?;
    
    println!("✅ Client created with custom gateway configuration");
    println!("📡 Base URL: {}", base_url);
    println!("🤖 Model: claude-3-5-sonnet-latest\n");
    
    // Test 1: Simple message
    println!("🧪 Test 1: Simple message...");
    let response = client.messages()
        .create(
            MessageCreateBuilder::new("claude-3-5-sonnet-latest", 100)
                .user("Hello! Please respond with 'SDK Test Successful' to confirm the connection works.")
                .build()
        )
        .await;
    
    match response {
        Ok(msg) => {
            println!("✅ Test 1 PASSED!");
            let text = extract_text_from_content(&msg.content);
            println!("📝 Response: {}", text);
            println!("📊 Usage: {} input tokens, {} output tokens\n", 
                msg.usage.input_tokens, msg.usage.output_tokens);
        }
        Err(e) => {
            println!("❌ Test 1 FAILED!");
            println!("🚨 Error: {}\n", e);
            
            // If it's an authentication error, that's expected with dummy key
            if api_key == "dummy-key" {
                println!("💡 This is expected with dummy API key. Please set a real API key to test actual requests.\n");
            }
        }
    }
    
    // Test 2: System prompt + user message
    println!("🧪 Test 2: System prompt + user message...");
    let response = client.messages()
        .create(
            MessageCreateBuilder::new("claude-3-5-sonnet-latest", 150)
                .system("You are a helpful assistant that responds concisely.")
                .user("What is 2+2? Please answer briefly.")
                .build()
        )
        .await;
    
    match response {
        Ok(msg) => {
            println!("✅ Test 2 PASSED!");
            let text = extract_text_from_content(&msg.content);
            println!("📝 Response: {}", text);
            println!("📊 Usage: {} input tokens, {} output tokens\n", 
                msg.usage.input_tokens, msg.usage.output_tokens);
        }
        Err(e) => {
            println!("❌ Test 2 FAILED!");
            println!("🚨 Error: {}\n", e);
        }
    }
    
    // Test 3: Temperature and max_tokens variation
    println!("🧪 Test 3: Temperature variation...");
    let response = client.messages()
        .create(
            MessageCreateBuilder::new("claude-3-5-sonnet-latest", 200)
                .user("Generate a creative one-liner joke.")
                .temperature(0.8)
                .build()
        )
        .await;
    
    match response {
        Ok(msg) => {
            println!("✅ Test 3 PASSED!");
            let text = extract_text_from_content(&msg.content);
            println!("📝 Response: {}", text);
            println!("📊 Usage: {} input tokens, {} output tokens\n", 
                msg.usage.input_tokens, msg.usage.output_tokens);
        }
        Err(e) => {
            println!("❌ Test 3 FAILED!");
            println!("🚨 Error: {}\n", e);
        }
    }
    
    println!("🎉 All tests completed!");
    
    println!("\n💡 To test with real API calls:");
    println!("   export CUSTOM_BEARER_TOKEN='your-custom-gateway-key'");
    println!("   export CUSTOM_BASE_URL='https://your-gateway.example.com/v1/anthropic'");
    println!("   cargo run --example test_custom_gateway");
    
    Ok(())
} 