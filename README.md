# Anthropic Rust SDK

[![Crates.io](https://img.shields.io/crates/v/anthropic-sdk-rust.svg)](https://crates.io/crates/anthropic-sdk-rust)
[![Documentation](https://docs.rs/anthropic-sdk-rust/badge.svg)](https://docs.rs/anthropic-sdk-rust)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

An unofficial, comprehensive, type-safe Rust SDK for the Anthropic API with full feature parity to the TypeScript SDK. Built with modern async/await patterns, extensive error handling, and ergonomic builder APIs.

## ✨ Features

- **🚀 Full API Coverage**: Messages, Streaming, Tools, Vision, Files, Batches, and Models APIs
- **🛡️ Type Safety**: Comprehensive type system with serde integration
- **🔄 Async/Await**: Built on tokio with efficient async operations
- **🔧 Builder Patterns**: Ergonomic fluent APIs for easy usage
- **📡 Streaming Support**: Real-time streaming responses with proper backpressure
- **🛠️ Tool Integration**: Complete tool use system with custom tool support
- **👁️ Vision Support**: Image analysis with base64 and URL inputs
- **📁 File Management**: Upload, manage, and process files
- **📊 Batch Processing**: Efficient batch operations for large workloads
- **🤖 Model Intelligence**: Smart model selection and comparison utilities
- **⚡ Production Ready**: Comprehensive error handling, retries, and logging

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
anthropic-sdk-rust = "0.1.0"
tokio = { version = "1.0", features = ["rt-multi-thread", "macros"] }
```

## 🚀 Quick Start

### Basic Setup

```rust
use anthropic_sdk::{Anthropic, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Create client with API key from environment or directly
    let client = Anthropic::new("your-api-key")?;
    
    // Or use environment variable ANTHROPIC_API_KEY
    let client = Anthropic::from_env()?;
    
    Ok(())
}
```

### Simple Message

```rust
use anthropic_sdk::{Anthropic, MessageCreateBuilder, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Anthropic::from_env()?;
    
    let response = client.messages()
        .create(
            MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
                .user("Hello, Claude! How are you today?")
                .build()
        )
        .await?;
    
    println!("Claude: {}", response.content.get_text());
    Ok(())
}
```

## 📖 Comprehensive Usage Guide

### 🔧 Client Configuration

```rust
use anthropic_sdk::{Anthropic, ClientConfig, LogLevel};
use std::time::Duration;

let config = ClientConfig::new("your-api-key")
    .with_timeout(Duration::from_secs(30))
    .with_max_retries(3)
    .with_log_level(LogLevel::Info)
    .with_base_url("https://api.anthropic.com");

let client = Anthropic::with_config(config)?;
```

### 💬 Messages API

#### Basic Conversation
```rust
let response = client.messages()
    .create(
        MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
            .user("What's the capital of France?")
            .system("You are a helpful geography assistant.")
            .build()
    )
    .await?;
```

#### Multi-turn Conversation
```rust
let response = client.messages()
    .create(
        MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
            .user("Hi! What's your name?")
            .assistant("Hello! I'm Claude, an AI assistant.")
            .user("Nice to meet you! Can you help me with math?")
            .system("You are a helpful math tutor.")
            .temperature(0.3)
            .build()
    )
    .await?;
```

#### Advanced Parameters
```rust
let response = client.messages()
    .create(
        MessageCreateBuilder::new("claude-3-5-sonnet-latest", 2048)
            .user("Write a creative story about space exploration.")
            .temperature(0.8)  // More creative
            .top_p(0.9)        // Nucleus sampling
            .top_k(50)         // Top-k sampling
            .stop_sequences(vec!["THE END".to_string()])
            .build()
    )
    .await?;
```

### 🔄 Streaming Responses

```rust
use anthropic_sdk::{MessageCreateBuilder, StreamEvent};
use futures::StreamExt;

let mut stream = client.messages()
    .stream(
        MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
            .user("Tell me a story")
            .build()
    )
    .await?;

while let Some(event) = stream.next().await {
    match event? {
        StreamEvent::ContentBlockDelta { delta, .. } => {
            if let Some(text) = delta.text {
                print!("{}", text);
            }
        }
        StreamEvent::MessageStop => break,
        _ => {}
    }
}
```

### 👁️ Vision (Image Analysis)

```rust
use anthropic_sdk::{ContentBlockParam, MessageContent};

// Using base64 image
let response = client.messages()
    .create(
        MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
            .user(MessageContent::Blocks(vec![
                ContentBlockParam::text("What do you see in this image?"),
                ContentBlockParam::image_base64("image/jpeg", base64_image_data),
            ]))
            .build()
    )
    .await?;

// Using image URL
let response = client.messages()
    .create(
        MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
            .user(MessageContent::Blocks(vec![
                ContentBlockParam::text("Describe this image"),
                ContentBlockParam::image_url("https://example.com/image.jpg"),
            ]))
            .build()
    )
    .await?;
```

### 🛠️ Tool Use

```rust
use anthropic_sdk::{Tool, ToolFunction, MessageCreateBuilder};

// Define a weather tool
let weather_tool = Tool {
    name: "get_weather".to_string(),
    description: "Get current weather for a location".to_string(),
    input_schema: serde_json::json!({
        "type": "object",
        "properties": {
            "location": {
                "type": "string",
                "description": "City name"
            }
        },
        "required": ["location"]
    }),
};

let response = client.messages()
    .create(
        MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
            .user("What's the weather like in Paris?")
            .tools(vec![weather_tool])
            .build()
    )
    .await?;

// Handle tool use in response
if let Some(tool_use) = response.content.get_tool_use() {
    println!("Claude wants to use tool: {}", tool_use.name);
    println!("With input: {}", tool_use.input);
}
```

### 📁 File Management

```rust
// Upload a file
let file = client.files()
    .upload("path/to/document.pdf", "assistants")
    .await?;

println!("Uploaded file: {} ({})", file.filename, file.id);

// List files
let files = client.files()
    .list(Some("assistants"), None, None)
    .await?;

// Use file in conversation
let response = client.messages()
    .create(
        MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
            .user(MessageContent::Blocks(vec![
                ContentBlockParam::text("Summarize this document"),
                ContentBlockParam::document(file.id, "Document to analyze"),
            ]))
            .build()
    )
    .await?;
```

### 📊 Batch Processing

```rust
use anthropic_sdk::{BatchCreateBuilder, BatchRequestBuilder};

// Create batch requests
let requests = vec![
    BatchRequestBuilder::new("req_1")
        .messages(
            MessageCreateBuilder::new("claude-3-5-sonnet-latest", 100)
                .user("What is 2+2?")
                .build()
        )
        .build(),
    BatchRequestBuilder::new("req_2")
        .messages(
            MessageCreateBuilder::new("claude-3-5-sonnet-latest", 100)
                .user("What is 3+3?")
                .build()
        )
        .build(),
];

// Create and submit batch
let batch = client.batches()
    .create(
        BatchCreateBuilder::new(requests)
            .completion_window("24h")
            .build()
    )
    .await?;

println!("Batch created: {}", batch.id);

// Check batch status
let status = client.batches().retrieve(&batch.id).await?;
println!("Batch status: {:?}", status.processing_status);
```

### 🤖 Model Intelligence

```rust
use anthropic_sdk::{ModelRequirements, ModelCapability};

// List available models
let models = client.models().list(None).await?;
for model in models.data {
    println!("Model: {} ({})", model.display_name, model.id);
}

// Find best model for requirements
let requirements = ModelRequirements::new()
    .max_cost_per_million_tokens(15.0)
    .min_context_length(100000)
    .required_capabilities(vec![
        ModelCapability::Vision,
        ModelCapability::ToolUse,
    ])
    .build();

let best_model = client.models()
    .find_best_model(&requirements)
    .await?;

println!("Best model: {}", best_model.display_name);

// Compare models
let comparison = client.models()
    .compare_models(&["claude-3-5-sonnet-latest", "claude-3-5-haiku-latest"])
    .await?;

println!("Comparison: {}", comparison.summary.recommendation);
```

## 🏗️ Architecture & Design

### Type System
The SDK uses a comprehensive type system with:
- **Builders**: Fluent APIs for easy construction
- **Enums**: Type-safe model and parameter selection
- **Validation**: Compile-time and runtime validation
- **Serialization**: Automatic JSON handling with serde

### Error Handling
```rust
use anthropic_sdk::{AnthropicError, Result};

match client.messages().create(params).await {
    Ok(response) => println!("Success: {}", response.content.get_text()),
    Err(AnthropicError::ApiError { status, message, .. }) => {
        eprintln!("API Error {}: {}", status, message);
    }
    Err(AnthropicError::NetworkError { source }) => {
        eprintln!("Network Error: {}", source);
    }
    Err(e) => eprintln!("Other Error: {}", e),
}
```

### Async Patterns
Built on tokio with:
- Efficient connection pooling
- Automatic retries with exponential backoff
- Proper timeout handling
- Stream processing with backpressure

## 🔧 Configuration Options

### Environment Variables
```bash
ANTHROPIC_API_KEY=your-api-key
ANTHROPIC_BASE_URL=https://api.anthropic.com  # Optional
ANTHROPIC_TIMEOUT=30                          # Seconds
ANTHROPIC_MAX_RETRIES=3                       # Retry attempts
```

### Custom Configuration
```rust
use anthropic_sdk::{ClientConfig, LogLevel, Anthropic};
use std::time::Duration;

let config = ClientConfig::new("api-key")
    .with_base_url("https://api.anthropic.com")
    .with_timeout(Duration::from_secs(60))
    .with_max_retries(5)
    .with_log_level(LogLevel::Debug)
    .with_user_agent("MyApp/1.0");

let client = Anthropic::with_config(config)?;
```

## 📊 Examples

The `examples/` directory contains comprehensive demonstrations:

- **`basic_client.rs`** - Basic client setup and configuration
- **`messages_api.rs`** - Complete Messages API usage
- **`streaming_example.rs`** - Real-time streaming responses
- **`comprehensive_tool_use.rs`** - Advanced tool integration
- **`comprehensive_file_upload.rs`** - File management workflows
- **`phase5_1_batches.rs`** - Batch processing examples
- **`phase5_3_models_api.rs`** - Model selection and comparison
- **`production_patterns.rs`** - Production-ready patterns

Run examples:
```bash
cargo run --example basic_client
cargo run --example messages_api
cargo run --example streaming_example
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Test specific module
cargo test messages

# Run integration tests
cargo test --test integration
```

## 📚 Documentation

Generate and view documentation:
```bash
# Generate docs
cargo doc --open --no-deps

# View online documentation
# https://docs.rs/anthropic-sdk-rust
```

## 🚀 Performance & Production

### Connection Pooling
The SDK automatically manages HTTP connections with:
- Keep-alive connections
- Connection pooling via reqwest
- Automatic retries with exponential backoff

### Memory Management
- Streaming responses to handle large outputs
- Efficient JSON parsing with serde
- Minimal allocations in hot paths

### Error Recovery
- Automatic retries for transient failures
- Circuit breaker patterns for reliability
- Comprehensive error types for debugging

## 🛡️ Security

- API keys handled securely (never logged)
- HTTPS-only communication
- Input validation and sanitization
- Rate limiting awareness

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and add tests
4. Run tests: `cargo test`
5. Commit: `git commit -m 'Add amazing feature'`
6. Push: `git push origin feature/amazing-feature`
7. Open a Pull Request

## 📋 Roadmap

- [x] **Phase 1**: Foundation & Client Setup
- [x] **Phase 2**: Messages API
- [x] **Phase 3**: Streaming & Tools
- [x] **Phase 4**: Vision & Advanced Features  
- [x] **Phase 5**: Files, Batches & Models APIs
- [ ] **Phase 6**: Cloud Integrations (AWS Bedrock, GCP Vertex AI)
- [ ] **Phase 7**: Advanced Streaming & WebSockets
- [ ] **Phase 8**: Caching & Performance Optimization

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙋‍♂️ Support

- **Documentation**: [docs.rs/anthropic-sdk-rust](https://docs.rs/anthropic-sdk-rust)
- **Issues**: [GitHub Issues](https://github.com/dimichgh/anthropic-sdk-rust/issues)
- **Discussions**: [GitHub Discussions](https://github.com/dimichgh/anthropic-sdk-rust/discussions)

## 🎯 Examples Gallery

### Real-world Use Cases

```rust
// Customer Support Bot
let response = client.messages()
    .create(
        MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
            .system("You are a helpful customer support agent.")
            .user("I need help with my order #12345")
            .temperature(0.1) // Consistent responses
            .build()
    )
    .await?;

// Code Review Assistant
let response = client.messages()
    .create(
        MessageCreateBuilder::new("claude-3-5-sonnet-latest", 2048)
            .system("You are an expert code reviewer.")
            .user("Please review this Rust code for potential issues")
            .tools(vec![code_analysis_tool])
            .build()
    )
    .await?;

// Document Analysis
let response = client.messages()
    .create(
        MessageCreateBuilder::new("claude-3-5-sonnet-latest", 4096)
            .user(MessageContent::Blocks(vec![
                ContentBlockParam::text("Analyze this financial report"),
                ContentBlockParam::document(uploaded_file.id, "Q3 Report"),
            ]))
            .build()
    )
    .await?;
```

---

**Ready to build amazing AI-powered applications with Rust? Get started with the Anthropic SDK today!** 🦀✨
