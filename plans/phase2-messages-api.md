# Phase 2: Core Messages API Implementation Plan

## Objective
Implement the core Messages API functionality, providing the ability to create messages with Claude models, handle responses, and manage token usage - achieving feature parity with the TypeScript SDK's primary use case.

## TypeScript SDK Reference Analysis

Based on the TypeScript SDK structure (`anthropic-sdk-typescript/src/resources/messages/`), the key components are:

### Core Message Types:
- **Message**: Response type with content, role, usage, etc.
- **MessageCreateParams**: Request parameters 
- **ContentBlock**: Text/image content types
- **Usage**: Token counting information
- **Role**: User/Assistant message roles
- **Model**: Supported Claude model variants

### API Endpoints:
- **POST /v1/messages** - Create a message
- **Streaming**: Server-sent events support (Phase 3)

## Phase 2 Tasks Breakdown

### Task 1: Message Type Definitions 🎯 **READY**

**Implementation Details:**
```rust
// src/types/messages.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub type_: String, // "message"  
    pub role: Role,
    pub content: Vec<ContentBlock>,
    pub model: String,
    pub stop_reason: Option<StopReason>,
    pub stop_sequence: Option<String>,
    pub usage: Usage,
    #[serde(skip)]
    pub request_id: Option<RequestId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { 
        source: ImageSource,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StopReason {
    #[serde(rename = "end_turn")]
    EndTurn,
    #[serde(rename = "max_tokens")]
    MaxTokens,
    #[serde(rename = "stop_sequence")]
    StopSequence,
    #[serde(rename = "tool_use")]
    ToolUse,
}
```

### Task 2: Message Request Builder 🎯 **READY**

**Implementation Details:**
```rust
// src/types/messages.rs (continued)
#[derive(Debug, Clone, Serialize)]
pub struct MessageCreateParams {
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<MessageParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageParam {
    pub role: Role,
    pub content: MessageContent,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

// Builder pattern for ergonomic API
#[derive(Debug, Clone)]
pub struct MessageCreateBuilder {
    params: MessageCreateParams,
}

impl MessageCreateBuilder {
    pub fn new(model: impl Into<String>, max_tokens: u32) -> Self { ... }
    pub fn message(mut self, role: Role, content: impl Into<MessageContent>) -> Self { ... }
    pub fn system(mut self, system: impl Into<String>) -> Self { ... }
    pub fn temperature(mut self, temperature: f32) -> Self { ... }
    pub fn build(self) -> MessageCreateParams { ... }
}
```

### Task 3: Models and Constants 🎯 **READY**

**Implementation Details:**
```rust
// src/types/models.rs
/// Supported Claude model variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Model {
    #[serde(rename = "claude-3-5-sonnet-latest")]
    Claude3_5SonnetLatest,
    #[serde(rename = "claude-3-5-sonnet-20241022")]
    Claude3_5Sonnet20241022,
    #[serde(rename = "claude-3-5-haiku-latest")]
    Claude3_5HaikuLatest,
    #[serde(rename = "claude-3-5-haiku-20241022")]
    Claude3_5Haiku20241022,
    #[serde(rename = "claude-3-opus-latest")]
    Claude3OpusLatest,
    #[serde(rename = "claude-3-opus-20240229")]
    Claude3Opus20240229,
}

impl Model {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Claude3_5SonnetLatest => "claude-3-5-sonnet-latest",
            Self::Claude3_5Sonnet20241022 => "claude-3-5-sonnet-20241022",
            Self::Claude3_5HaikuLatest => "claude-3-5-haiku-latest",
            Self::Claude3_5Haiku20241022 => "claude-3-5-haiku-20241022",
            Self::Claude3OpusLatest => "claude-3-opus-latest",
            Self::Claude3Opus20240229 => "claude-3-opus-20240229",
        }
    }
}

impl std::fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<Model> for String {
    fn from(model: Model) -> String {
        model.as_str().to_string()
    }
}
```

### Task 4: Messages Resource Implementation 🎯 **READY**

**Implementation Details:**
```rust
// src/resources/messages.rs
use crate::client::Anthropic;
use crate::types::messages::*;
use crate::types::errors::Result;
use crate::types::shared::RequestId;

pub struct MessagesResource<'a> {
    client: &'a Anthropic,
}

impl<'a> MessagesResource<'a> {
    pub fn new(client: &'a Anthropic) -> Self {
        Self { client }
    }
    
    /// Create a message with Claude
    pub async fn create(&self, params: MessageCreateParams) -> Result<Message> {
        let url = self.client.http_client().build_url("/v1/messages");
        
        let request = self.client.http_client()
            .post(&url)
            .json(&params)
            .build()
            .map_err(|e| AnthropicError::Connection { source: e })?;
        
        let response = self.client.http_client().send(request).await?;
        
        // Extract request ID from headers
        let request_id = self.client.http_client().extract_request_id(&response);
        
        let mut message: Message = response.json().await
            .map_err(|e| AnthropicError::Connection { source: e })?;
            
        message.request_id = request_id;
        
        Ok(message)
    }
}

// Convenience methods for the main client
impl Anthropic {
    /// Access to the Messages API
    pub fn messages(&self) -> MessagesResource {
        MessagesResource::new(self)
    }
}
```

### Task 5: Integration & Testing 🎯 **READY**

**Implementation Details:**
- Update main client to include messages resource
- Create comprehensive unit tests
- Create integration tests with mock server
- Add examples demonstrating usage
- Update documentation

## File Structure Changes

```
src/
├── types/
│   ├── mod.rs              # Updated exports
│   ├── messages.rs         # NEW - Message types
│   └── models.rs           # NEW - Model definitions
├── resources/              # NEW - API resources
│   ├── mod.rs              # NEW - Resource exports  
│   └── messages.rs         # NEW - Messages API
├── client.rs               # Updated with messages() method
└── lib.rs                  # Updated exports
```

## Testing Strategy for Phase 2

### Unit Tests:
1. **Message Type Serialization** - JSON round-trip tests
2. **Builder Pattern** - Ergonomic API construction
3. **Model Validation** - Enum serialization
4. **Error Handling** - API error parsing

### Integration Tests:
1. **Mock Server Tests** - Simulate API responses
2. **Request Validation** - Ensure proper JSON structure
3. **Response Parsing** - Handle various API response formats
4. **Error Scenarios** - Test error handling paths

### Examples:
1. **Basic Message Creation** - Simple text message
2. **Multi-message Conversation** - Back-and-forth chat
3. **Advanced Parameters** - System prompts, temperature, etc.
4. **Error Handling** - Graceful error management

## Acceptance Criteria

✅ **Phase 2 Complete When:**
1. All message types defined and properly serialized
2. MessageCreateBuilder provides ergonomic API
3. MessagesResource implements create() method
4. Request/response handling works end-to-end
5. Comprehensive test coverage (>90%)
6. Examples demonstrate real usage patterns
7. Error handling covers all API error cases
8. Integration with Phase 1 foundation seamless

## Next Phase Dependencies

**Phase 3 Requirements:**
- Message creation API ✅ (from Phase 2)
- HTTP client streaming ✅ (from Phase 1)
- **NEW**: Server-sent events parsing
- **NEW**: Stream event types
- **NEW**: Message accumulation logic

---

**Attention: Engineer** - Begin implementing Phase 2 tasks in order. Start with message type definitions and work systematically through each component.

---
*Phase 2 Plan Created: 2024*  
*Estimated Duration: 3-4 days*  
*Dependencies: Phase 1 Foundation ✅* 