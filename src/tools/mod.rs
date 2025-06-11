//! Tool execution and management for the Anthropic SDK.
//!
//! This module provides the infrastructure for registering, validating, and executing
//! tools (functions) that Claude can call during conversations.
//!
//! # Examples
//!
//! ## Basic Tool Registration and Execution
//!
//! ```rust
//! use anthropic_sdk::tools::{ToolRegistry, ToolFunction};
//! use anthropic_sdk::types::{Tool, ToolUse, ToolResult};
//! use serde_json::{json, Value};
//! use async_trait::async_trait;
//!
//! // Define a tool function
//! struct WeatherTool;
//!
//! #[async_trait]
//! impl ToolFunction for WeatherTool {
//!     async fn execute(&self, input: Value) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
//!         let location = input["location"].as_str().unwrap_or("Unknown");
//!         let weather_data = format!("The weather in {} is 72°F and sunny", location);
//!         Ok(ToolResult::success("tool_use_id", weather_data))
//!     }
//! }
//!
//! // Register and use the tool
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut registry = ToolRegistry::new();
//!
//! let weather_tool_def = Tool::new("get_weather", "Get the current weather in a given location")
//!     .parameter("location", "string", "The city and state")
//!     .required("location")
//!     .build();
//!
//! registry.register("get_weather", weather_tool_def, Box::new(WeatherTool))?;
//!
//! // Execute a tool call
//! let tool_use = ToolUse {
//!     id: "toolu_123".to_string(),
//!     name: "get_weather".to_string(),
//!     input: json!({"location": "San Francisco, CA"}),
//! };
//!
//! let result = registry.execute(&tool_use).await?;
//! # Ok(())
//! # }
//! ```

pub mod executor;
pub mod registry;
pub mod conversation;

use async_trait::async_trait;
use serde_json::Value;
use std::error::Error;

pub use executor::{ToolExecutor, ToolExecutionConfig, ToolExecutionConfigBuilder};
pub use registry::{ToolRegistry, SharedToolRegistry};
pub use conversation::{ToolConversation, ConversationConfig, ConversationConfigBuilder};

use crate::types::ToolResult;

/// Trait for implementing tool functions.
///
/// This trait allows you to define custom tools that Claude can call.
/// Each tool receives structured input and returns a structured result.
#[async_trait]
pub trait ToolFunction: Send + Sync {
    /// Execute the tool with the given input.
    ///
    /// # Arguments
    /// * `input` - JSON input parameters from Claude
    ///
    /// # Returns
    /// A `ToolResult` containing the execution result, or an error if execution fails.
    async fn execute(&self, input: Value) -> Result<ToolResult, Box<dyn Error + Send + Sync>>;
    
    /// Validate input before execution (optional).
    ///
    /// Override this method to provide custom validation logic beyond
    /// the JSON schema validation.
    fn validate_input(&self, _input: &Value) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
    
    /// Get the tool's timeout in seconds (optional).
    ///
    /// Override to set a custom timeout for this tool.
    /// Default is 30 seconds.
    fn timeout_seconds(&self) -> u64 {
        30
    }
}

/// Simple function wrapper for tool functions.
///
/// This allows you to register simple async functions as tools without
/// implementing the full `ToolFunction` trait.
pub struct SimpleTool<F>
where
    F: Fn(Value) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ToolResult, Box<dyn Error + Send + Sync>>> + Send>> + Send + Sync,
{
    function: F,
}

impl<F> SimpleTool<F>
where
    F: Fn(Value) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ToolResult, Box<dyn Error + Send + Sync>>> + Send>> + Send + Sync,
{
    /// Create a new simple tool from a function.
    pub fn new(function: F) -> Self {
        Self { function }
    }
}

#[async_trait]
impl<F> ToolFunction for SimpleTool<F>
where
    F: Fn(Value) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<ToolResult, Box<dyn Error + Send + Sync>>> + Send>> + Send + Sync,
{
    async fn execute(&self, input: Value) -> Result<ToolResult, Box<dyn Error + Send + Sync>> {
        (self.function)(input).await
    }
}

/// Tool execution errors.
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    /// Tool not found in registry.
    #[error("Tool '{name}' not found")]
    NotFound { name: String },
    
    /// Tool validation failed.
    #[error("Tool validation failed: {message}")]
    ValidationFailed { message: String },
    
    /// Tool execution failed.
    #[error("Tool execution failed: {source}")]
    ExecutionFailed { 
        #[source]
        source: Box<dyn Error + Send + Sync> 
    },
    
    /// Tool execution timed out.
    #[error("Tool execution timed out after {seconds} seconds")]
    Timeout { seconds: u64 },
    
    /// Tool registry error.
    #[error("Tool registry error: {message}")]
    RegistryError { message: String },
}

/// Result type for tool operations.
pub type ToolOperationResult<T> = Result<T, ToolError>;

/// Helper macro for creating simple tool functions.
///
/// # Example
///
/// ```rust
/// use anthropic_sdk::tool_function;
/// use serde_json::{json, Value};
///
/// let weather_tool = tool_function!(|input: Value| async move {
///     let location = input["location"].as_str().unwrap_or("Unknown");
///     let result = format!("Weather in {}: 72°F and sunny", location);
///     Ok(anthropic_sdk::ToolResult::success("tool_id", result))
/// });
/// ```
#[macro_export]
macro_rules! tool_function {
    (|$input:ident: Value| $body:expr) => {
        $crate::tools::SimpleTool::new(move |$input: Value| {
            Box::pin($body)
        })
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Tool, ToolResult};
    use serde_json::json;

    struct TestTool;

    #[async_trait]
    impl ToolFunction for TestTool {
        async fn execute(&self, input: Value) -> Result<ToolResult, Box<dyn Error + Send + Sync>> {
            let message = input["message"].as_str().unwrap_or("Hello");
            Ok(ToolResult::success("test_id", format!("Echo: {}", message)))
        }
    }

    #[tokio::test]
    async fn test_tool_function_execution() {
        let tool = TestTool;
        let input = json!({"message": "Hello, World!"});
        
        let result = tool.execute(input).await.unwrap();
        
        if let crate::types::ToolResultContent::Text(content) = result.content {
            assert_eq!(content, "Echo: Hello, World!");
        } else {
            panic!("Expected text content");
        }
    }

    #[tokio::test]
    async fn test_simple_tool() {
        let tool = SimpleTool::new(|input: Value| {
            Box::pin(async move {
                let number = input["number"].as_f64().unwrap_or(0.0);
                let result = number * 2.0;
                Ok(ToolResult::success("test_id", format!("Result: {}", result)))
            })
        });

        let input = json!({"number": 21.0});
        let result = tool.execute(input).await.unwrap();
        
        if let crate::types::ToolResultContent::Text(content) = result.content {
            assert_eq!(content, "Result: 42");
        } else {
            panic!("Expected text content");
        }
    }

    #[test]
    fn test_tool_function_macro() {
        let _tool = tool_function!(|input: Value| async move {
            let value = input["test"].as_str().unwrap_or("default");
            Ok(ToolResult::success("macro_test", format!("Processed: {}", value)))
        });
        
        // Test compilation of the macro
        assert!(true);
    }
} 