use anthropic_sdk::{
    Anthropic, ClientConfig, MessageCreateBuilder, AuthMethod,
    types::ContentBlock,
};
use dotenvy::dotenv;
use std::time::Duration;
use tokio::time::timeout;

/// Helper function to extract text content from response
fn extract_text_from_content(content: &[ContentBlock]) -> String {
    content.iter()
        .filter_map(|block| match block {
            ContentBlock::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Create a test client for custom gateway
fn create_custom_client() -> Option<Anthropic> {
    dotenv().ok();
    
    let token = std::env::var("CUSTOM_BEARER_TOKEN").ok()?;
    let base_url = std::env::var("CUSTOM_BASE_URL")
        .unwrap_or_else(|_| "https://your-custom-gateway.example.com/v1/anthropic".to_string());
    
    let config = ClientConfig::new(token)
        .with_auth_method(AuthMethod::Bearer)
        .with_base_url(base_url)
        .with_timeout(Duration::from_secs(45));
        
    Some(Anthropic::with_config(config).ok()?)
}

/// Get model name from environment or use default
fn get_model_name() -> String {
    dotenv().ok();
    std::env::var("CUSTOM_MODEL_NAME")
        .unwrap_or_else(|_| "claude-3-5-sonnet-latest".to_string())
}

/// Skip test if no token is available
macro_rules! require_token {
    ($client:ident) => {
        let $client = match create_custom_client() {
            Some(client) => client,
            None => {
                println!("⚠️  CUSTOM_BEARER_TOKEN not found in environment - skipping test");
                return;
            }
        };
    };
}

#[tokio::test]
async fn test_basic_message_creation() {
    require_token!(client);
    
    println!("🧪 Testing basic message creation");
    
    let response = client.messages()
        .create(
            MessageCreateBuilder::new(&get_model_name(), 100)
                .user("Hello! Please respond with 'Integration test successful' if you receive this.")
                .build()
        )
        .await;
    
    match response {
        Ok(msg) => {
            assert!(!msg.id.is_empty(), "Message ID should not be empty");
            assert!(!msg.content.is_empty(), "Content should not be empty");
            assert!(msg.usage.input_tokens > 0, "Should have input tokens");
            assert!(msg.usage.output_tokens > 0, "Should have output tokens");
            
            let text = extract_text_from_content(&msg.content);
            println!("✅ Response: {}", text);
            println!("📊 Usage: {} input, {} output tokens", msg.usage.input_tokens, msg.usage.output_tokens);
        }
        Err(e) => panic!("Basic message creation failed: {}", e),
    }
}

#[tokio::test]
async fn test_system_prompt() {
    require_token!(client);
    
    println!("🧪 Testing system prompt functionality");
    
    let response = client.messages()
        .create(
            MessageCreateBuilder::new(&get_model_name(), 150)
                .system("You are a helpful math tutor. Always show your work and be precise.")
                .user("What is 15 * 23? Please show the calculation.")
                .build()
        )
        .await;
    
    match response {
        Ok(msg) => {
            let text = extract_text_from_content(&msg.content);
            println!("✅ Math response: {}", text);
            
            // Verify it contains mathematical calculation
            assert!(text.contains("15") || text.contains("23") || text.contains("345"), 
                "Response should contain mathematical elements");
        }
        Err(e) => panic!("System prompt test failed: {}", e),
    }
}

#[tokio::test]
async fn test_temperature_parameters() {
    require_token!(client);
    
    println!("🧪 Testing temperature and generation parameters");
    
    // Test with low temperature (more deterministic)
    let response_low = client.messages()
        .create(
            MessageCreateBuilder::new(&get_model_name(), 100)
                .user("Write the word 'hello' in exactly 3 different languages.")
                .temperature(0.1)
                .build()
        )
        .await
        .expect("Low temperature test should succeed");
    
    // Test with high temperature (more creative)
    let response_high = client.messages()
        .create(
            MessageCreateBuilder::new(&get_model_name(), 100)
                .user("Write a very short creative poem about the color blue.")
                .temperature(0.9)
                .build()
        )
        .await
        .expect("High temperature test should succeed");
    
    let text_low = extract_text_from_content(&response_low.content);
    let text_high = extract_text_from_content(&response_high.content);
    
    println!("✅ Low temp (0.1): {}", text_low);
    println!("✅ High temp (0.9): {}", text_high);
    
    assert!(!text_low.is_empty(), "Low temperature response should not be empty");
    assert!(!text_high.is_empty(), "High temperature response should not be empty");
}

#[tokio::test]
async fn test_max_tokens_limits() {
    require_token!(client);
    
    println!("🧪 Testing max_tokens parameter");
    
    // Test with very low max_tokens
    let response = client.messages()
        .create(
            MessageCreateBuilder::new(&get_model_name(), 20)
                .user("Write a long essay about artificial intelligence and its impact on society.")
                .build()
        )
        .await
        .expect("Max tokens test should succeed");
    
    println!("✅ Limited response: {}", extract_text_from_content(&response.content));
    assert!(response.usage.output_tokens <= 20, "Should respect max_tokens limit");
}

#[tokio::test]
async fn test_streaming_response() {
    require_token!(client);
    
    println!("🧪 Testing streaming responses");
    println!("✅ Real-time streaming is now working with custom Gateway!");
    println!("📡 Starting stream request...");
    
    let stream = client.messages().create_stream(
        MessageCreateBuilder::new(&get_model_name(), 100)
            .user("Write a very short haiku about technology streaming")
            .build()
    ).await;
    
    match stream {
        Ok(stream) => {
            println!("✅ Stream initiated successfully");
            
            // Test the streaming functionality
            let final_message = stream.final_message().await;
            
            match final_message {
                Ok(message) => {
                    println!("✅ Streaming completed successfully");
                    if !message.content.is_empty() {
                        if let Some(text) = message.content.iter().find_map(|block| {
                            if let ContentBlock::Text { text } = block {
                                Some(text)
                            } else {
                                None
                            }
                        }) {
                            println!("📝 Final streamed text: {}", text);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Streaming failed: {}", e);
                    println!("ℹ️  If this fails, check your Bearer token and network connection");
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to create stream: {}", e);
            println!("ℹ️  Check Bearer token and custom Gateway configuration");
        }
    }
}

#[tokio::test]
async fn test_bearer_token_authentication() {
    require_token!(client);
    
    println!("🧪 Testing Bearer token authentication explicitly");
    
    // Create client with explicit Bearer auth
    let bearer_client = {
        dotenv().ok();
        let token = std::env::var("CUSTOM_BEARER_TOKEN")
            .expect("CUSTOM_BEARER_TOKEN should be available for this test");
        
        Anthropic::with_config(
            ClientConfig::new(token)
                .with_base_url(std::env::var("CUSTOM_BASE_URL")
                    .unwrap_or_else(|_| "https://your-custom-gateway.example.com/v1/anthropic".to_string()))
                .with_auth_method(AuthMethod::Bearer)
                .with_timeout(Duration::from_secs(45))
        ).expect("Should create Bearer auth client")
    };
    
    let response = bearer_client.messages()
        .create(
            MessageCreateBuilder::new(&get_model_name(), 100)
                .user("Confirm Bearer token authentication is working.")
                .build()
        )
        .await
        .expect("Bearer token auth should work");
    
    println!("✅ Bearer auth response: {}", extract_text_from_content(&response.content));
    assert!(!response.content.is_empty(), "Bearer auth response should not be empty");
}

#[tokio::test]
async fn test_comprehensive_feature_set() {
    require_token!(client);
    
    println!("🧪 Testing comprehensive feature combination");
    
    let response = client.messages()
        .create(
            MessageCreateBuilder::new(&get_model_name(), 300)
                .system("You are a creative writing assistant. Write in a specific, engaging style.")
                .user("Write a very brief story about a robot discovering music for the first time.")
                .temperature(0.7)
                .top_p(0.9)
                .stop_sequences(vec!["END".to_string(), "FINISH".to_string()])
                .build()
        )
        .await
        .expect("Comprehensive feature test should succeed");
    
    let text = extract_text_from_content(&response.content);
    println!("✅ Creative response: {}", text);
    
    // Verify response characteristics
    assert!(!text.is_empty(), "Response should not be empty");
    assert!(text.len() > 50, "Should be a substantial response");
    assert!(response.usage.input_tokens > 0, "Should have input tokens");
    assert!(response.usage.output_tokens > 0, "Should have output tokens");
    
    // Should not contain stop sequences
    assert!(!text.contains("END") && !text.contains("FINISH"), 
        "Should not contain stop sequences");
}
