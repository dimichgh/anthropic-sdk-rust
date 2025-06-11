//! # Anthropic SDK for Rust
//!
//! This crate provides a Rust SDK for the Anthropic API, offering feature parity
//! with the official TypeScript SDK.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use anthropic_sdk::Anthropic;
//!
//! // Create client from environment variable ANTHROPIC_API_KEY
//! let client = Anthropic::from_env()?;
//!
//! // Or create with explicit API key
//! let client = Anthropic::new("your-api-key")?;
//!
//! // Test the connection
//! client.test_connection().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//!
//! - **Messages API**: Create and stream Claude conversations
//! - **Authentication**: Automatic API key management
//! - **Error Handling**: Comprehensive error types
//! - **Logging**: Configurable tracing support
//! - **Async/Await**: Built on tokio for high performance
//!

pub mod client;
pub mod config;
pub mod types;
pub mod http;
pub mod utils;
pub mod resources;
pub mod streaming;
pub mod tools;
pub mod files;
pub mod tokens;

// Re-exports for public API
pub use client::Anthropic;
pub use config::{ClientConfig, LogLevel};
pub use types::{
    AnthropicError, Result, RequestId, Usage,
    Message, Role, ContentBlock, ImageSource, StopReason,
    MessageCreateParams, MessageParam, MessageContent, ContentBlockParam,
    MessageCreateBuilder, Model,
    // Streaming types
    MessageStreamEvent, MessageDelta, MessageDeltaUsage,
    ContentBlockDelta, TextCitation,
    // Tool types
    Tool, ToolBuilder, ToolChoice, ToolUse, ToolResult, ToolResultContent,
    ToolValidationError, ServerTool, WebSearchParameters,
    // Batch types (Beta)
    MessageBatch, BatchStatus, BatchRequestCounts, BatchRequest, BatchRequestBuilder,
    BatchResult, BatchResponse, BatchResponseBody, BatchError,
    BatchCreateParams, BatchListParams, BatchList,
    // Files API types (Beta)
    FileObject, FilePurpose, FileStatus, FileUploadParams, FileListParams, FileList,
    FileOrder, UploadProgress, StorageInfo, FileDownload,
    // Models API types
    ModelObject, ModelListParams, ModelList, ModelCapabilities, ModelCapability,
    ModelPricing, PricingTier, ModelComparison, ModelPerformance, ComparisonSummary,
    ModelRequirements, ModelUsageRecommendations, ModelRecommendation,
    RecommendedParameters, PerformanceExpectations, CostRange, QualityLevel,
    CostEstimation, CostBreakdown,
};
pub use tools::{
    ToolRegistry, ToolExecutor, ToolConversation, ToolFunction,
    ToolExecutionConfig, ConversationConfig, ToolError,
};
pub use resources::{
    MessagesResource, BatchesResource, FilesResource, ModelsResource,
};
pub use files::{
    File, FileData, FileSource, FileConstraints, FileBuilder, FileError, to_file,
};
pub use tokens::{
    TokenCounter, UsageStats, ModelUsage, RequestUsage, ModelPrice,
    UsageSummary,
};
pub use http::{
    RetryPolicy, RetryCondition, RetryExecutor, RetryResult, default_retry, api_retry,
};
pub use streaming::MessageStream;
pub use http::auth::AuthMethod;

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// User-Agent string for HTTP requests
pub const USER_AGENT: &str = concat!("anthropic-sdk-rust/", env!("CARGO_PKG_VERSION"));

// Convenient type aliases
pub type Error = AnthropicError;

// Module re-exports for organized access
pub use types as types_module; 