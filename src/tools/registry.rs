//! Tool registry for managing and executing registered tools.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use crate::types::{Tool, ToolUse, ToolResult};
use super::{ToolFunction, ToolError, ToolOperationResult};

/// Registry for managing tool definitions and their implementations.
///
/// The registry stores tool definitions and their corresponding execution functions,
/// provides validation, and handles tool execution with proper error handling.
pub struct ToolRegistry {
    /// Map of tool names to their definitions and implementations.
    tools: HashMap<String, ToolEntry>,
}

/// Internal structure for storing tool information.
struct ToolEntry {
    /// The tool definition with schema.
    definition: Tool,
    /// The tool implementation.
    implementation: Box<dyn ToolFunction>,
}

impl ToolRegistry {
    /// Create a new empty tool registry.
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool with its definition and implementation.
    ///
    /// # Arguments
    /// * `name` - Unique name for the tool
    /// * `definition` - Tool definition with JSON schema
    /// * `implementation` - Tool implementation
    ///
    /// # Returns
    /// `Ok(())` if successful, `Err(ToolError)` if the tool name already exists.
    ///
    /// # Example
    /// ```rust
    /// use anthropic_sdk::tools::{ToolRegistry, ToolFunction};
    /// use anthropic_sdk::types::{Tool, ToolResult};
    /// use serde_json::Value;
    /// use async_trait::async_trait;
    ///
    /// struct WeatherTool;
    ///
    /// #[async_trait]
    /// impl ToolFunction for WeatherTool {
    ///     async fn execute(&self, input: Value) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
    ///         let location = input["location"].as_str().unwrap_or("Unknown");
    ///         Ok(ToolResult::success("id", format!("Weather in {}: Sunny, 72°F", location)))
    ///     }
    /// }
    ///
    /// let mut registry = ToolRegistry::new();
    /// let tool_def = Tool::new("get_weather", "Get weather information")
    ///     .parameter("location", "string", "Location to get weather for")
    ///     .required("location")
    ///     .build();
    ///
    /// registry.register("get_weather", tool_def, Box::new(WeatherTool)).unwrap();
    /// ```
    pub fn register(
        &mut self,
        name: impl Into<String>,
        definition: Tool,
        implementation: Box<dyn ToolFunction>,
    ) -> ToolOperationResult<()> {
        let tool_name = name.into();

        if self.tools.contains_key(&tool_name) {
            return Err(ToolError::RegistryError {
                message: format!("Tool '{}' is already registered", tool_name),
            });
        }

        self.tools.insert(
            tool_name,
            ToolEntry {
                definition,
                implementation,
            },
        );

        Ok(())
    }

    /// Get tool definitions for all registered tools.
    ///
    /// Returns a vector of tool definitions that can be sent to Claude.
    pub fn get_tool_definitions(&self) -> Vec<Tool> {
        self.tools.values().map(|entry| entry.definition.clone()).collect()
    }

    /// Get tool definitions for specific tools by name.
    ///
    /// # Arguments
    /// * `names` - Iterator of tool names to retrieve
    ///
    /// # Returns
    /// Vector of tool definitions for the specified tools.
    pub fn get_specific_tools<I>(&self, names: I) -> Vec<Tool>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        names
            .into_iter()
            .filter_map(|name| {
                self.tools.get(name.as_ref()).map(|entry| entry.definition.clone())
            })
            .collect()
    }

    /// Check if a tool is registered.
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }

    /// Get the number of registered tools.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Execute a tool call.
    ///
    /// This method validates the input against the tool's schema and executes
    /// the tool with proper error handling and timeout management.
    ///
    /// # Arguments
    /// * `tool_use` - Tool use request from Claude
    ///
    /// # Returns
    /// `ToolResult` with the execution result or error information.
    ///
    /// # Example
    /// ```rust
    /// use anthropic_sdk::types::ToolUse;
    /// use serde_json::json;
    ///
    /// # async fn example(registry: &anthropic_sdk::ToolRegistry) -> Result<(), Box<dyn std::error::Error>> {
    /// let tool_use = ToolUse {
    ///     id: "toolu_123".to_string(),
    ///     name: "get_weather".to_string(),
    ///     input: json!({"location": "San Francisco, CA"}),
    /// };
    ///
    /// let result = registry.execute(&tool_use).await?;
    /// println!("Tool result: {:?}", result);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute(&self, tool_use: &ToolUse) -> ToolOperationResult<ToolResult> {
        // Find the tool
        let tool_entry = self.tools.get(&tool_use.name).ok_or_else(|| ToolError::NotFound {
            name: tool_use.name.clone(),
        })?;

        // Validate input against schema
        if let Err(validation_error) = tool_entry.definition.validate_input(&tool_use.input) {
            return Ok(ToolResult::error(
                tool_use.id.clone(),
                format!("Validation failed: {}", validation_error),
            ));
        }

        // Custom validation from the tool implementation
        if let Err(custom_error) = tool_entry.implementation.validate_input(&tool_use.input) {
            return Ok(ToolResult::error(
                tool_use.id.clone(),
                format!("Custom validation failed: {}", custom_error),
            ));
        }

        // Execute with timeout
        let execution_timeout = Duration::from_secs(tool_entry.implementation.timeout_seconds());
        
        match timeout(
            execution_timeout,
            tool_entry.implementation.execute(tool_use.input.clone()),
        ).await {
            Ok(Ok(mut result)) => {
                // Ensure the result has the correct tool_use_id
                result.tool_use_id = tool_use.id.clone();
                Ok(result)
            }
            Ok(Err(execution_error)) => {
                Err(ToolError::ExecutionFailed { 
                    source: execution_error 
                })
            }
            Err(_) => {
                Ok(ToolResult::error(
                    tool_use.id.clone(),
                    format!("Tool execution timed out after {} seconds", tool_entry.implementation.timeout_seconds()),
                ))
            }
        }
    }

    /// Execute multiple tool calls in parallel.
    ///
    /// # Arguments
    /// * `tool_uses` - Vector of tool use requests
    ///
    /// # Returns
    /// Vector of tool results in the same order as the input.
    pub async fn execute_parallel(&self, tool_uses: &[ToolUse]) -> Vec<ToolOperationResult<ToolResult>> {
        let futures = tool_uses.iter().map(|tool_use| self.execute(tool_use));
        futures::future::join_all(futures).await
    }

    /// Remove a tool from the registry.
    ///
    /// # Arguments
    /// * `name` - Name of the tool to remove
    ///
    /// # Returns
    /// `true` if the tool was removed, `false` if it wasn't found.
    pub fn unregister(&mut self, name: &str) -> bool {
        self.tools.remove(name).is_some()
    }

    /// Clear all tools from the registry.
    pub fn clear(&mut self) {
        self.tools.clear();
    }

    /// Get a list of all registered tool names.
    pub fn tool_names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    /// Get a tool definition by name.
    pub fn get_tool_definition(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name).map(|entry| &entry.definition)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared tool registry for use across multiple requests.
///
/// This wraps the registry in an Arc for thread-safe sharing.
pub type SharedToolRegistry = Arc<ToolRegistry>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::ToolFunction;
    use async_trait::async_trait;
    use serde_json::json;

    struct TestEchoTool;

    #[async_trait]
    impl ToolFunction for TestEchoTool {
        async fn execute(&self, input: serde_json::Value) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
            let message = input["message"].as_str().unwrap_or("No message");
            Ok(ToolResult::success("test_id", format!("Echo: {}", message)))
        }
    }

    struct TestMathTool;

    #[async_trait]
    impl ToolFunction for TestMathTool {
        async fn execute(&self, input: serde_json::Value) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
            let a = input["a"].as_f64().unwrap_or(0.0);
            let b = input["b"].as_f64().unwrap_or(0.0);
            let operation = input["operation"].as_str().unwrap_or("add");
            
            let result = match operation {
                "add" => a + b,
                "subtract" => a - b,
                "multiply" => a * b,
                "divide" => if b != 0.0 { a / b } else { return Ok(ToolResult::error("test_id", "Division by zero")); },
                _ => return Ok(ToolResult::error("test_id", "Unknown operation")),
            };
            
            Ok(ToolResult::success("test_id", result.to_string()))
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_tool_registration() {
        let mut registry = ToolRegistry::new();
        
        let tool_def = Tool::new("echo", "Echo a message")
            .parameter("message", "string", "Message to echo")
            .required("message")
            .build();

        let result = registry.register("echo", tool_def, Box::new(TestEchoTool));
        assert!(result.is_ok());
        assert_eq!(registry.len(), 1);
        assert!(registry.has_tool("echo"));
    }

    #[test]
    fn test_duplicate_tool_registration() {
        let mut registry = ToolRegistry::new();
        
        let tool_def = Tool::new("echo", "Echo a message")
            .build();

        registry.register("echo", tool_def.clone(), Box::new(TestEchoTool)).unwrap();
        
        // Try to register the same tool again
        let result = registry.register("echo", tool_def, Box::new(TestEchoTool));
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_tool_execution() {
        let mut registry = ToolRegistry::new();
        
        let tool_def = Tool::new("echo", "Echo a message")
            .parameter("message", "string", "Message to echo")
            .required("message")
            .build();

        registry.register("echo", tool_def, Box::new(TestEchoTool)).unwrap();

        let tool_use = ToolUse {
            id: "test_123".to_string(),
            name: "echo".to_string(),
            input: json!({"message": "Hello, World!"}),
        };

        let result = registry.execute(&tool_use).await.unwrap();
        
        if let crate::types::ToolResultContent::Text(content) = result.content {
            assert_eq!(content, "Echo: Hello, World!");
        } else {
            panic!("Expected text content");
        }
        
        assert_eq!(result.tool_use_id, "test_123");
    }

    #[tokio::test]
    async fn test_tool_not_found() {
        let registry = ToolRegistry::new();
        
        let tool_use = ToolUse {
            id: "test_123".to_string(),
            name: "nonexistent".to_string(),
            input: json!({}),
        };

        let result = registry.execute(&tool_use).await;
        assert!(result.is_err());
        
        if let Err(ToolError::NotFound { name }) = result {
            assert_eq!(name, "nonexistent");
        } else {
            panic!("Expected NotFound error");
        }
    }

    #[tokio::test]
    async fn test_parallel_execution() {
        let mut registry = ToolRegistry::new();
        
        let math_tool_def = Tool::new("math", "Perform math operations")
            .parameter("a", "number", "First number")
            .parameter("b", "number", "Second number")
            .parameter("operation", "string", "Operation to perform")
            .required("a")
            .required("b")
            .required("operation")
            .build();

        registry.register("math", math_tool_def, Box::new(TestMathTool)).unwrap();

        let tool_uses = vec![
            ToolUse {
                id: "test_1".to_string(),
                name: "math".to_string(),
                input: json!({"a": 10, "b": 5, "operation": "add"}),
            },
            ToolUse {
                id: "test_2".to_string(),
                name: "math".to_string(),
                input: json!({"a": 10, "b": 5, "operation": "multiply"}),
            },
        ];

        let results = registry.execute_parallel(&tool_uses).await;
        
        assert_eq!(results.len(), 2);
        
        for result in results {
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_get_tool_definitions() {
        let mut registry = ToolRegistry::new();
        
        let tool1 = Tool::new("tool1", "First tool")
            .build();
            
        let tool2 = Tool::new("tool2", "Second tool")
            .build();

        registry.register("tool1", tool1, Box::new(TestEchoTool)).unwrap();
        registry.register("tool2", tool2, Box::new(TestEchoTool)).unwrap();

        let definitions = registry.get_tool_definitions();
        assert_eq!(definitions.len(), 2);
        
        let names: Vec<&str> = definitions.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"tool1"));
        assert!(names.contains(&"tool2"));
    }

    #[test]
    fn test_get_specific_tools() {
        let mut registry = ToolRegistry::new();
        
        let tool1 = Tool::new("tool1", "First tool").build();
        let tool2 = Tool::new("tool2", "Second tool").build();
        let tool3 = Tool::new("tool3", "Third tool").build();

        registry.register("tool1", tool1, Box::new(TestEchoTool)).unwrap();
        registry.register("tool2", tool2, Box::new(TestEchoTool)).unwrap();
        registry.register("tool3", tool3, Box::new(TestEchoTool)).unwrap();

        let specific_tools = registry.get_specific_tools(["tool1", "tool3"]);
        assert_eq!(specific_tools.len(), 2);
        
        let names: Vec<&str> = specific_tools.iter().map(|t| t.name.as_str()).collect();
        assert!(names.contains(&"tool1"));
        assert!(names.contains(&"tool3"));
        assert!(!names.contains(&"tool2"));
    }
} 