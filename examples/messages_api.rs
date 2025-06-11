use anthropic_sdk::{
    Anthropic, MessageCreateBuilder, Role, ContentBlockParam, Model,
    MessageContent, Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🦀 Anthropic Rust SDK - Phase 2 Messages API Demo");
    println!("{}", "=".repeat(60));

    // Note: These examples use a demo API key and won't actually make real API calls
    // To test with real API calls, set your ANTHROPIC_API_KEY environment variable
    
    // Example 1: Basic message creation using the builder pattern
    println!("\n📝 Example 1: Basic Message Creation");
    demonstrate_basic_message().await?;

    // Example 2: Multi-turn conversation
    println!("\n💬 Example 2: Multi-turn Conversation");
    demonstrate_conversation().await?;

    // Example 3: Advanced parameters and system prompt
    println!("\n⚙️ Example 3: Advanced Parameters");
    demonstrate_advanced_params().await?;

    // Example 4: Using the ergonomic builder API
    println!("\n🔧 Example 4: Ergonomic Builder API");
    demonstrate_builder_api().await?;

    // Example 5: Image content (vision capabilities)
    println!("\n🖼️ Example 5: Vision - Image Content");
    demonstrate_vision().await?;

    // Example 6: Model selection and capabilities
    println!("\n🤖 Example 6: Model Selection");
    demonstrate_models().await?;

    // Example 7: JSON serialization inspection
    println!("\n📄 Example 7: JSON Serialization");
    demonstrate_serialization().await?;

    println!("\n🎯 Phase 2 Messages API Complete!");
    println!("✅ All message creation patterns implemented");
    println!("✅ Type-safe builder pattern");
    println!("✅ Vision and multimodal support");
    println!("✅ Comprehensive model selection");
    
    Ok(())
}

async fn demonstrate_basic_message() -> Result<()> {
    println!("   Creating a simple message request...");
    
    let client = Anthropic::new("demo-api-key")?;
    
    let params = MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
        .user("Hello, Claude! How are you today?")
        .build();
    
    println!("   ✅ Message parameters created:");
    println!("      Model: {}", params.model);
    println!("      Max tokens: {}", params.max_tokens);
    println!("      Messages: {} message(s)", params.messages.len());
    println!("      First message role: {:?}", params.messages[0].role);
    
    // Note: We can't actually call the API without a real key, but the structure is ready
    println!("   📋 Ready for API call: client.messages().create(params).await");
    
    Ok(())
}

async fn demonstrate_conversation() -> Result<()> {
    println!("   Building a multi-turn conversation...");
    
    let params = MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
        .user("Hi there! What's your name?")
        .assistant("Hello! I'm Claude, an AI assistant created by Anthropic.")
        .user("Nice to meet you, Claude! Can you help me understand Rust?")
        .system("You are a helpful programming tutor specializing in Rust.")
        .temperature(0.3) // Lower temperature for more focused responses
        .build();
    
    println!("   ✅ Multi-turn conversation created:");
    println!("      Total messages: {}", params.messages.len());
    println!("      System prompt: {:?}", params.system.as_ref().map(|s| &s[..50]));
    println!("      Temperature: {:?}", params.temperature);
    
    for (i, msg) in params.messages.iter().enumerate() {
        println!("      Message {}: {:?} - {}", i + 1, msg.role, 
            match &msg.content {
                MessageContent::Text(text) => &text[..text.len().min(40)],
                MessageContent::Blocks(_) => "[complex content]",
            }
        );
    }
    
    Ok(())
}

async fn demonstrate_advanced_params() -> Result<()> {
    println!("   Configuring advanced generation parameters...");
    
    let params = MessageCreateBuilder::new("claude-3-5-sonnet-latest", 2048)
        .user("Write a creative short story about a robot learning to paint.")
        .system("You are a creative writing assistant. Write engaging, imaginative stories.")
        .temperature(0.8) // Higher temperature for creativity
        .top_p(0.9)       // Nucleus sampling
        .top_k(50)        // Top-k sampling
        .stop_sequences(vec!["THE END".to_string(), "[STORY COMPLETE]".to_string()])
        .build();
    
    println!("   ✅ Advanced parameters configured:");
    println!("      Max tokens: {}", params.max_tokens);
    println!("      Temperature: {:?} (creative)", params.temperature);
    println!("      Top-p: {:?}", params.top_p);
    println!("      Top-k: {:?}", params.top_k);
    println!("      Stop sequences: {:?}", params.stop_sequences);
    
    Ok(())
}

async fn demonstrate_builder_api() -> Result<()> {
    println!("   Using the ergonomic builder API...");
    
    let client = Anthropic::new("demo-api-key")?;
    
    // This creates a builder that's ready to send
    let builder = client.messages()
        .create_with_builder("claude-3-5-sonnet-latest", 1024)
        .user("What's the weather like today?")
        .system("You are a helpful weather assistant.")
        .temperature(0.1);
    
    println!("   ✅ Builder created and configured");
    println!("   📋 Ready to send: builder.send().await");
    println!("   🔧 This provides a fluent API for message creation");
    
    Ok(())
}

async fn demonstrate_vision() -> Result<()> {
    println!("   Creating message with image content...");
    
    // Create a message with both text and image content
    let image_content = MessageContent::Blocks(vec![
        ContentBlockParam::text("What do you see in this image?"),
        ContentBlockParam::image_base64(
            "image/jpeg",
            "/9j/4AAQSkZJRg..." // Would be actual base64 data
        ),
    ]);
    
    let params = MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
        .user(image_content)
        .system("You are a helpful vision assistant. Describe images accurately.")
        .build();
    
    println!("   ✅ Vision message created:");
    println!("      Content blocks: {}", 
        match &params.messages[0].content {
            MessageContent::Blocks(blocks) => blocks.len(),
            MessageContent::Text(_) => 1,
        }
    );
    println!("      ✅ Text content block");
    println!("      ✅ Image content block (base64)");
    
    Ok(())
}

async fn demonstrate_models() -> Result<()> {
    println!("   Exploring available models...");
    
    let models = vec![
        Model::Claude3_5SonnetLatest,
        Model::Claude3_5HaikuLatest,
        Model::Claude3OpusLatest,
        Model::Claude3Sonnet20240229,
    ];
    
    for model in models {
        println!("   🤖 Model: {}", model);
        println!("      Family: {}", model.family());
        println!("      Vision support: {}", model.supports_vision());
        println!("      Tool support: {}", model.supports_tools());
        
        // Create a message with this model
        let _params = MessageCreateBuilder::new(model.as_str(), 1024)
            .user("Hello!")
            .build();
            
        println!("      ✅ Compatible with MessageCreateBuilder");
        println!();
    }
    
    Ok(())
}

async fn demonstrate_serialization() -> Result<()> {
    println!("   Inspecting JSON serialization...");
    
    let params = MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
        .user("Serialize me!")
        .system("You are helpful")
        .temperature(0.5)
        .build();
    
    let json = serde_json::to_string_pretty(&params)
        .map_err(|e| anthropic_sdk::AnthropicError::Configuration {
            message: format!("Serialization error: {}", e),
        })?;
    
    println!("   ✅ JSON serialization successful:");
    println!("{}", json);
    
    // Verify it matches expected structure
    let json_value: serde_json::Value = serde_json::from_str(&json)
        .map_err(|e| anthropic_sdk::AnthropicError::Configuration {
            message: format!("Deserialization error: {}", e),
        })?;
    
    println!("\n   🔍 Verification:");
    println!("      Has 'model' field: {}", json_value.get("model").is_some());
    println!("      Has 'max_tokens' field: {}", json_value.get("max_tokens").is_some());
    println!("      Has 'messages' array: {}", json_value.get("messages").and_then(|v| v.as_array()).is_some());
    println!("      Has 'system' field: {}", json_value.get("system").is_some());
    println!("      Has 'temperature' field: {}", json_value.get("temperature").is_some());
    
    Ok(())
} 