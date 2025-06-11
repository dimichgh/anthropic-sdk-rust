# Phase 1: Foundation Implementation Plan

## Objective
Set up the foundational structure for the Anthropic Rust SDK, including project configuration, core client architecture, and essential infrastructure components.

## Phase 1 Tasks Breakdown

### Task 1: Project Structure Setup ✅ **COMPLETED**
- [x] Architecture plan created (`plans/rust-sdk-architecture.md`)
- [x] Status tracking document created (`plans/status.md`)
- [x] Phase 1 plan created (this document)

### Task 2: Core Cargo.toml Setup 🎯 **READY**

**Implementation Details:**
```toml
[package]
name = "anthropic-sdk"
version = "0.1.0"
edition = "2021"
authors = ["Anthropic SDK Team"]
description = "Official Rust SDK for the Anthropic API"
repository = "https://github.com/dimichgh/anthropic-sdk-rust"
license = "MIT"
keywords = ["anthropic", "claude", "ai", "api", "sdk"]
categories = ["api-bindings", "web-programming::http-client"]
readme = "README.md"

[lib]
name = "anthropic_sdk"
path = "src/lib.rs"

# Core dependencies for HTTP, JSON, async, and error handling
[dependencies]
reqwest = { version = "0.12", features = ["json", "stream"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["rt-multi-thread", "macros"] }
futures = "0.3"
thiserror = "1.0"
tracing = "0.1"
dotenvy = "0.15"
url = "2.5"
bytes = "1.5"
mime = "0.3.17"

# Optional cloud integrations
[features]
default = []
bedrock = ["aws-sdk-bedrock"]
vertex = ["gcp-vertex-ai"]
all = ["bedrock", "vertex"]

[dependencies.aws-sdk-bedrock]
version = "1.0"
optional = true

[dependencies.gcp-vertex-ai]
version = "0.1"
optional = true

[dev-dependencies]
tokio-test = "0.4"
```

**Files to Create:**
- `Cargo.toml` (root)

### Task 3: Library Structure Setup 🎯 **READY**

**Implementation Details:**
Create the basic source code structure with module organization:

```
src/
├── lib.rs              # Public API exports
├── client.rs           # Main Anthropic client
├── config.rs           # Configuration management  
├── types/              # Type definitions
│   ├── mod.rs
│   ├── errors.rs       # Error types
│   └── shared.rs       # Common types
├── http/               # HTTP layer
│   ├── mod.rs
│   ├── client.rs       # HTTP client wrapper
│   └── auth.rs         # Authentication
└── utils/              # Utilities
    ├── mod.rs
    └── logging.rs      # Logging setup
```

**Files to Create:**
- `src/lib.rs`
- `src/client.rs`
- `src/config.rs`
- `src/types/mod.rs`
- `src/types/errors.rs`
- `src/types/shared.rs`
- `src/http/mod.rs`
- `src/http/client.rs`
- `src/http/auth.rs`
- `src/utils/mod.rs`
- `src/utils/logging.rs`

### Task 4: Error Type Definitions 🎯 **READY**

**Implementation Details:**
Define comprehensive error types matching TypeScript SDK error patterns:

```rust
// src/types/errors.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnthropicError {
    #[error("Bad request: {message}")]
    BadRequest { message: String, status: u16 },
    
    #[error("Authentication failed: {message}")]
    Authentication { message: String, status: u16 },
    
    #[error("Permission denied: {message}")]
    PermissionDenied { message: String, status: u16 },
    
    #[error("Resource not found: {message}")]
    NotFound { message: String, status: u16 },
    
    #[error("Unprocessable entity: {message}")]
    UnprocessableEntity { message: String, status: u16 },
    
    #[error("Rate limit exceeded: {message}")]
    RateLimit { message: String, status: u16 },
    
    #[error("Internal server error: {message}")]
    InternalServer { message: String, status: u16 },
    
    #[error("API connection error: {source}")]
    Connection { source: reqwest::Error },
    
    #[error("API connection timeout")]
    ConnectionTimeout,
    
    #[error("User aborted request")]
    UserAbort,
    
    #[error("Invalid configuration: {message}")]
    Configuration { message: String },
}

pub type Result<T> = std::result::Result<T, AnthropicError>;
```

### Task 5: Configuration Management Implementation 🎯 **READY**

**Implementation Details:**
```rust
// src/config.rs
use std::time::Duration;
use dotenvy::dotenv;
use crate::types::errors::{AnthropicError, Result};

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout: Duration,
    pub max_retries: u32,
    pub log_level: LogLevel,
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Off,
}

impl ClientConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.anthropic.com".to_string(),
            timeout: Duration::from_secs(600), // 10 minutes default
            max_retries: 2,
            log_level: LogLevel::Warn,
        }
    }
    
    pub fn from_env() -> Result<Self> {
        dotenv().ok(); // Load .env file if present
        
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| AnthropicError::Configuration {
                message: "ANTHROPIC_API_KEY environment variable not set".to_string(),
            })?;
            
        let mut config = Self::new(api_key);
        
        // Optional environment variable overrides
        if let Ok(base_url) = std::env::var("ANTHROPIC_BASE_URL") {
            config.base_url = base_url;
        }
        
        Ok(config)
    }
    
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
    
    pub fn with_log_level(mut self, log_level: LogLevel) -> Self {
        self.log_level = log_level;
        self
    }
}
```

### Task 6: HTTP Client Wrapper 🎯 **READY**

**Implementation Details:**
```rust
// src/http/client.rs
use reqwest::{Client, Request, Response};
use crate::config::ClientConfig;
use crate::types::errors::{AnthropicError, Result};

pub struct HttpClient {
    client: Client,
    config: ClientConfig,
}

impl HttpClient {
    pub fn new(config: ClientConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|e| AnthropicError::Connection { source: e })?;
            
        Ok(Self { client, config })
    }
    
    pub async fn send(&self, mut request: Request) -> Result<Response> {
        // Add authentication headers
        let headers = request.headers_mut();
        headers.insert(
            "x-api-key", 
            self.config.api_key.parse()
                .map_err(|_| AnthropicError::Configuration {
                    message: "Invalid API key format".to_string(),
                })?
        );
        headers.insert("anthropic-version", "2023-06-01".parse().unwrap());
        
        let response = self.client.send(request).await
            .map_err(|e| AnthropicError::Connection { source: e })?;
            
        self.handle_response_status(&response)?;
        Ok(response)
    }
    
    fn handle_response_status(&self, response: &Response) -> Result<()> {
        let status = response.status();
        if status.is_success() {
            return Ok(());
        }
        
        let status_code = status.as_u16();
        let message = format!("HTTP {}: {}", status_code, status.canonical_reason().unwrap_or("Unknown"));
        
        let error = match status_code {
            400 => AnthropicError::BadRequest { message, status: status_code },
            401 => AnthropicError::Authentication { message, status: status_code },
            403 => AnthropicError::PermissionDenied { message, status: status_code },
            404 => AnthropicError::NotFound { message, status: status_code },
            422 => AnthropicError::UnprocessableEntity { message, status: status_code },
            429 => AnthropicError::RateLimit { message, status: status_code },
            500..=599 => AnthropicError::InternalServer { message, status: status_code },
            _ => AnthropicError::Connection { 
                source: reqwest::Error::from(reqwest::ErrorKind::Request) 
            },
        };
        
        Err(error)
    }
}
```

### Task 7: Core Client Implementation 🎯 **READY**

**Implementation Details:**
```rust
// src/client.rs
use crate::config::ClientConfig;
use crate::http::HttpClient;
use crate::types::errors::{AnthropicError, Result};

pub struct Anthropic {
    config: ClientConfig,
    http_client: HttpClient,
}

impl Anthropic {
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        let config = ClientConfig::new(api_key);
        let http_client = HttpClient::new(config.clone())?;
        
        Ok(Self {
            config,
            http_client,
        })
    }
    
    pub fn from_env() -> Result<Self> {
        let config = ClientConfig::from_env()?;
        let http_client = HttpClient::new(config.clone())?;
        
        Ok(Self {
            config,
            http_client,
        })
    }
    
    pub fn with_config(config: ClientConfig) -> Result<Self> {
        let http_client = HttpClient::new(config.clone())?;
        
        Ok(Self {
            config,
            http_client,
        })
    }
}
```

### Task 8: Public API Exports 🎯 **READY**

**Implementation Details:**
```rust
// src/lib.rs
//! # Anthropic SDK for Rust
//!
//! This crate provides a Rust SDK for the Anthropic API, offering feature parity
//! with the official TypeScript SDK.

pub mod client;
pub mod config;
pub mod types;
pub mod http;
pub mod utils;

// Re-exports for public API
pub use client::Anthropic;
pub use config::{ClientConfig, LogLevel};
pub use types::errors::{AnthropicError, Result};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
```

## Testing Strategy for Phase 1

### Unit Tests to Implement:
1. **Configuration Tests**
   - Environment variable loading
   - Default configuration values
   - Configuration validation

2. **Error Handling Tests**
   - Error type creation and formatting
   - HTTP status code mapping

3. **HTTP Client Tests**
   - Request header injection
   - Response status handling
   - Timeout configuration

4. **Client Construction Tests**
   - Various client creation patterns
   - Configuration validation

## Acceptance Criteria

✅ **Phase 1 Complete When:**
1. All source files created with proper module structure
2. Cargo.toml configured with all required dependencies  
3. Error types defined and properly structured
4. Configuration management working with environment variables
5. HTTP client wrapper handles authentication and basic errors
6. Core client can be instantiated successfully
7. Basic unit tests pass
8. Project compiles without warnings

## Next Phase Dependencies

**Phase 2 Requirements:**
- HTTP client foundation ✅ (from Phase 1)
- Error handling system ✅ (from Phase 1)  
- Configuration management ✅ (from Phase 1)
- **NEW**: Message type definitions
- **NEW**: API endpoint implementations

---

**Attention: Engineer** - Begin implementing Phase 1 tasks in the order listed above. Start with Cargo.toml setup and work through each task systematically.

---
*Phase 1 Plan Created: 2024*  
*Estimated Duration: 2-3 days*  
*Dependencies: None* 