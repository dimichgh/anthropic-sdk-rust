# Anthropic Rust SDK - Architecture Plan

## Project Overview

**Goal**: Create a Rust SDK for Anthropic API with complete feature parity to the TypeScript SDK (`@anthropic-ai/sdk` v0.53.0).

**Reference Implementation**: `/anthropic-sdk-typescript/` - Official TypeScript SDK
**Target**: Production-ready Rust crate with idiomatic Rust patterns and modern ecosystem integration.

## Core Features Analysis (from TypeScript SDK)

### 1. **Core Client & Configuration**
- **Main Client**: `Anthropic` struct with configuration options
- **Authentication**: API key management (`ANTHROPIC_API_KEY` env var support)
- **Base Configuration**: Timeout, retry logic, custom headers, logging
- **HTTP Client**: Built-in HTTP client with connection management

### 2. **Messages API** (Primary Feature)
- **Standard Messages**: Create, retrieve messages 
- **Streaming Support**: Server-sent events (SSE) for real-time responses
- **Message Types**: User/Assistant role messages with content blocks
- **Models**: Support for Claude 3.5 Sonnet and other model variants
- **Token Counting**: Input/output token usage tracking
- **Tool Use**: Function calling capabilities
- **Content Types**: Text, images, documents

### 3. **Beta Features**
- **Message Batches**: Batch processing API for multiple requests
- **Files API**: File upload and management
- **Models API**: Model information and capabilities

### 4. **Additional SDKs**
- **Bedrock Integration**: AWS Bedrock Claude API support
- **Vertex AI Integration**: Google Cloud Vertex AI support

### 5. **Developer Experience Features**
- **Type Safety**: Complete type definitions for all API parameters/responses
- **Error Handling**: Structured error types (400, 401, 403, 404, 422, 429, 5xx)
- **Pagination**: Auto-pagination for list endpoints
- **Request/Response Logging**: Debug logging with customizable levels
- **Retries**: Exponential backoff retry logic
- **Timeouts**: Dynamic timeout calculation based on token limits
- **Request IDs**: Request tracking for debugging

### 6. **Streaming Helpers**
- **Event Handlers**: `.on('text', callback)` patterns
- **Message Accumulation**: Build final message from stream events  
- **Stream Control**: Cancellation and abort capabilities

## Rust SDK Architecture Design

### 1. **Crate Structure**
```
anthropic-sdk/
├── Cargo.toml                 # Main crate manifest
├── src/
│   ├── lib.rs                 # Public API exports
│   ├── client.rs              # Main Anthropic client
│   ├── config.rs              # Configuration management
│   ├── types/                 # Type definitions
│   │   ├── mod.rs
│   │   ├── messages.rs        # Message types
│   │   ├── models.rs          # Model types
│   │   ├── errors.rs          # Error types
│   │   └── shared.rs          # Common types
│   ├── resources/             # API resource modules
│   │   ├── mod.rs
│   │   ├── messages.rs        # Messages API
│   │   ├── models.rs          # Models API
│   │   └── beta/              # Beta features
│   │       ├── mod.rs
│   │       ├── batches.rs     # Message batches
│   │       └── files.rs       # Files API
│   ├── streaming/             # Streaming functionality
│   │   ├── mod.rs
│   │   ├── events.rs          # SSE event handling
│   │   └── helpers.rs         # Streaming utilities
│   ├── http/                  # HTTP layer
│   │   ├── mod.rs
│   │   ├── client.rs          # HTTP client wrapper
│   │   ├── retry.rs           # Retry logic
│   │   └── auth.rs            # Authentication
│   └── utils/                 # Utilities
│       ├── mod.rs
│       ├── pagination.rs      # Auto-pagination
│       └── logging.rs         # Logging setup
├── examples/                  # Usage examples
├── tests/                     # Integration tests
└── README.md                  # Documentation
```

### 2. **Key Dependencies**
```toml
[dependencies]
# HTTP Client
reqwest = { version = "0.12", features = ["json", "stream"] }

# JSON Serialization  
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Async Runtime
tokio = { version = "1.0", features = ["rt-multi-thread", "macros"] }
futures = "0.3"

# SSE Streaming
eventsource-stream = "0.2"

# Error Handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Environment Variables
dotenvy = "0.15"

# URL Building
url = "2.5"

# File Handling
mime = "0.3.17"
bytes = "1.5"

# Optional: AWS/GCP integrations
aws-sdk-bedrock = { version = "1.0", optional = true }
gcp-vertex-ai = { version = "0.1", optional = true }
```

### 3. **Core Client Design**

```rust
pub struct Anthropic {
    config: ClientConfig,
    http_client: HttpClient,
    pub messages: MessagesResource,
    pub models: ModelsResource,
    pub beta: BetaResource,
}

pub struct ClientConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout: Duration,
    pub max_retries: u32,
    pub log_level: LogLevel,
}

impl Anthropic {
    pub fn new(api_key: impl Into<String>) -> Self { ... }
    pub fn from_env() -> Result<Self, Error> { ... }
    pub fn with_config(config: ClientConfig) -> Self { ... }
}
```

### 4. **Type System Design**
- **Request/Response Types**: Mirror TypeScript interfaces using `serde`
- **Builder Patterns**: For complex request construction
- **Enum Types**: For model names, message roles, content types
- **Error Types**: Structured error hierarchy with HTTP status mapping

### 5. **Streaming Architecture**
```rust
pub struct MessageStream {
    inner: EventStream,
    accumulated_message: Option<Message>,
}

impl MessageStream {
    pub async fn next_event(&mut self) -> Option<Result<MessageStreamEvent, Error>> { ... }
    pub async fn final_message(self) -> Result<Message, Error> { ... }
    pub fn on_text<F>(self, callback: F) -> Self where F: Fn(&str) { ... }
}
```

## Implementation Phases

### **Phase 1: Foundation** (2-3 days)
- [x] ✅ Project structure setup
- [ ] 🚧 Core client implementation
- [ ] 🚧 Configuration management
- [ ] 🚧 HTTP client wrapper
- [ ] 🚧 Basic authentication
- [ ] 🚧 Error type definitions

### **Phase 2: Core Messages API** (3-4 days)  
- [ ] ⏳ Message types and builders
- [ ] ⏳ Standard message creation
- [ ] ⏳ Response handling
- [ ] ⏳ Basic error handling
- [ ] ⏳ Unit tests for core functionality

### **Phase 3: Streaming Support** (2-3 days)
- [ ] ⏳ SSE event parsing
- [ ] ⏳ Stream management
- [ ] ⏳ Event handlers and callbacks
- [ ] ⏳ Message accumulation
- [ ] ⏳ Stream cancellation

### **Phase 4: Advanced Features** (3-4 days)
- [ ] ⏳ Tool use (function calling)
- [ ] ⏳ File uploads support
- [ ] ⏳ Token counting
- [ ] ⏳ Retry logic with exponential backoff
- [ ] ⏳ Request/response logging

### **Phase 5: Beta Features** (2-3 days)
- [ ] ⏳ Message batches API
- [ ] ⏳ Files API
- [ ] ⏳ Models API
- [ ] ⏳ Auto-pagination

### **Phase 6: Cloud Integrations** (3-4 days)
- [ ] ⏳ AWS Bedrock SDK
- [ ] ⏳ Google Vertex AI SDK
- [ ] ⏳ Feature flags and optional dependencies

### **Phase 7: Polish & Documentation** (2-3 days)
- [ ] ⏳ Comprehensive examples
- [ ] ⏳ Integration tests
- [ ] ⏳ Documentation and README
- [ ] ⏳ Performance optimization
- [ ] ⏳ Publish preparation

## Success Criteria

1. **Feature Parity**: All TypeScript SDK features implemented in Rust
2. **Type Safety**: Strong typing throughout the API surface
3. **Performance**: Efficient async/streaming operations
4. **Documentation**: Clear examples and API documentation
5. **Testing**: Comprehensive test coverage
6. **Ergonomics**: Idiomatic Rust patterns and ease of use

## Next Steps

**Attention: Engineer** - Please switch to **Act Mode** and begin implementing **Phase 1: Foundation**. Start with setting up the basic project structure, core client, and configuration management.

---
*Created by: Software Architect*  
*Date: 2024*  
*Status: 🎯 **APPROVED - Ready for Implementation*** 