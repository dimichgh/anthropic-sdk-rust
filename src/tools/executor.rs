//! High-level tool execution coordinator.
//!
//! This module provides the `ToolExecutor` which coordinates tool execution
//! across multiple tools, handles retries, and manages conversation flow.

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use crate::types::{ToolUse, ToolResult, Message, ContentBlock};
use super::{ToolRegistry, ToolError, ToolOperationResult};

/// Configuration for tool execution.
#[derive(Debug, Clone)]
pub struct ToolExecutionConfig {
    /// Maximum number of retry attempts for failed tools.
    pub max_retries: u32,
    
    /// Base delay between retry attempts.
    pub retry_delay: Duration,
    
    /// Whether to use exponential backoff for retries.
    pub exponential_backoff: bool,
    
    /// Maximum delay for exponential backoff.
    pub max_retry_delay: Duration,
    
    /// Whether to execute tools in parallel when possible.
    pub parallel_execution: bool,
    
    /// Maximum number of concurrent tool executions.
    pub max_concurrent_tools: usize,
}

impl Default for ToolExecutionConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay: Duration::from_millis(500),
            exponential_backoff: true,
            max_retry_delay: Duration::from_secs(10),
            parallel_execution: true,
            max_concurrent_tools: 4,
        }
    }
}

/// High-level tool executor that coordinates tool execution.
///
/// The executor handles multiple tool calls, retry logic, error recovery,
/// and provides higher-level abstractions for tool management.
pub struct ToolExecutor {
    /// The tool registry for executing tools.
    registry: Arc<ToolRegistry>,
    
    /// Configuration for tool execution.
    config: ToolExecutionConfig,
}

impl ToolExecutor {
    /// Create a new tool executor with the given registry.
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self {
            registry,
            config: ToolExecutionConfig::default(),
        }
    }

    /// Create a new tool executor with custom configuration.
    pub fn with_config(registry: Arc<ToolRegistry>, config: ToolExecutionConfig) -> Self {
        Self { registry, config }
    }

    /// Execute a single tool with retry logic.
    ///
    /// # Arguments
    /// * `tool_use` - The tool use request from Claude
    ///
    /// # Returns
    /// The tool result after execution (with retries if needed).
    pub async fn execute_with_retry(&self, tool_use: &ToolUse) -> ToolOperationResult<ToolResult> {
        let mut last_error = None;
        let mut delay = self.config.retry_delay;

        for attempt in 0..=self.config.max_retries {
            match self.registry.execute(tool_use).await {
                Ok(result) => {
                    // Check if the result indicates an error that should be retried
                    if let Some(true) = result.is_error {
                        if attempt < self.config.max_retries && self.should_retry_error(&result) {
                            last_error = Some(ToolError::ExecutionFailed {
                                source: format!("Tool returned error: {:?}", result.content).into(),
                            });
                            
                            if attempt < self.config.max_retries {
                                sleep(delay).await;
                                if self.config.exponential_backoff {
                                    delay = std::cmp::min(delay * 2, self.config.max_retry_delay);
                                }
                            }
                            continue;
                        }
                    }
                    return Ok(result);
                }
                Err(err) => {
                    if attempt < self.config.max_retries && self.should_retry_error_type(&err) {
                        last_error = Some(err);
                        sleep(delay).await;
                        if self.config.exponential_backoff {
                            delay = std::cmp::min(delay * 2, self.config.max_retry_delay);
                        }
                    } else {
                        return Err(err);
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| ToolError::ExecutionFailed {
            source: "Maximum retries exceeded".to_string().into(),
        }))
    }

    /// Execute multiple tools, potentially in parallel.
    ///
    /// # Arguments
    /// * `tool_uses` - Vector of tool use requests
    ///
    /// # Returns
    /// Vector of tool results in the same order as input.
    pub async fn execute_multiple(&self, tool_uses: &[ToolUse]) -> Vec<ToolOperationResult<ToolResult>> {
        if self.config.parallel_execution && tool_uses.len() > 1 {
            self.execute_parallel_with_concurrency(tool_uses).await
        } else {
            let mut results = Vec::with_capacity(tool_uses.len());
            for tool_use in tool_uses {
                results.push(self.execute_with_retry(tool_use).await);
            }
            results
        }
    }

    /// Execute tools in parallel with concurrency control.
    async fn execute_parallel_with_concurrency(&self, tool_uses: &[ToolUse]) -> Vec<ToolOperationResult<ToolResult>> {
        use futures::stream::{self, StreamExt};

        // Use a semaphore to limit concurrent executions
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.config.max_concurrent_tools));
        
        let futures = tool_uses.iter().enumerate().map(|(index, tool_use)| {
            let registry = self.registry.clone();
            let semaphore = semaphore.clone();
            let tool_use = tool_use.clone();
            let config = self.config.clone();
            
            async move {
                let _permit = semaphore.acquire().await.unwrap();
                let executor = ToolExecutor::with_config(registry, config);
                (index, executor.execute_with_retry(&tool_use).await)
            }
        });

        let mut results: Vec<(usize, ToolOperationResult<ToolResult>)> = stream::iter(futures)
            .buffer_unordered(self.config.max_concurrent_tools)
            .collect()
            .await;

        // Sort results by original index to maintain order
        results.sort_by_key(|(index, _)| *index);
        results.into_iter().map(|(_, result)| result).collect()
    }

    /// Extract tool use requests from a message.
    ///
    /// # Arguments
    /// * `message` - Message from Claude that may contain tool use requests
    ///
    /// # Returns
    /// Vector of tool use requests found in the message.
    pub fn extract_tool_uses(&self, message: &Message) -> Vec<ToolUse> {
        message
            .content
            .iter()
            .filter_map(|block| {
                if let ContentBlock::ToolUse { id, name, input } = block {
                    Some(ToolUse {
                        id: id.clone(),
                        name: name.clone(),
                        input: input.clone(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check if a tool should be retried based on the error in the result.
    fn should_retry_error(&self, _result: &ToolResult) -> bool {
        // Add logic to determine if specific error types should be retried
        // For now, we'll be conservative and not retry errors in results
        false
    }

    /// Check if a tool execution error should be retried.
    fn should_retry_error_type(&self, error: &ToolError) -> bool {
        match error {
            ToolError::ExecutionFailed { .. } => true,
            ToolError::Timeout { .. } => true,
            ToolError::ValidationFailed { .. } => false, // Don't retry validation errors
            ToolError::NotFound { .. } => false,         // Don't retry missing tools
            ToolError::RegistryError { .. } => false,    // Don't retry registry errors
        }
    }

    /// Get the underlying tool registry.
    pub fn registry(&self) -> &Arc<ToolRegistry> {
        &self.registry
    }

    /// Get the current execution configuration.
    pub fn config(&self) -> &ToolExecutionConfig {
        &self.config
    }

    /// Update the execution configuration.
    pub fn set_config(&mut self, config: ToolExecutionConfig) {
        self.config = config;
    }
}

/// Builder for creating tool execution configurations.
pub struct ToolExecutionConfigBuilder {
    config: ToolExecutionConfig,
}

impl ToolExecutionConfigBuilder {
    /// Create a new configuration builder with defaults.
    pub fn new() -> Self {
        Self {
            config: ToolExecutionConfig::default(),
        }
    }

    /// Set the maximum number of retry attempts.
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.config.max_retries = max_retries;
        self
    }

    /// Set the base retry delay.
    pub fn retry_delay(mut self, delay: Duration) -> Self {
        self.config.retry_delay = delay;
        self
    }

    /// Enable or disable exponential backoff.
    pub fn exponential_backoff(mut self, enabled: bool) -> Self {
        self.config.exponential_backoff = enabled;
        self
    }

    /// Set the maximum retry delay for exponential backoff.
    pub fn max_retry_delay(mut self, delay: Duration) -> Self {
        self.config.max_retry_delay = delay;
        self
    }

    /// Enable or disable parallel execution.
    pub fn parallel_execution(mut self, enabled: bool) -> Self {
        self.config.parallel_execution = enabled;
        self
    }

    /// Set the maximum number of concurrent tool executions.
    pub fn max_concurrent_tools(mut self, max: usize) -> Self {
        self.config.max_concurrent_tools = max;
        self
    }

    /// Build the configuration.
    pub fn build(self) -> ToolExecutionConfig {
        self.config
    }
}

impl Default for ToolExecutionConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::{ToolFunction, ToolRegistry};
    use crate::types::{Tool, ToolResult};
    use async_trait::async_trait;
    use serde_json::{json, Value};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestRetryTool {
        attempts: Arc<AtomicUsize>,
        fail_count: usize,
    }

    #[async_trait]
    impl ToolFunction for TestRetryTool {
        async fn execute(&self, _input: Value) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
            let attempt = self.attempts.fetch_add(1, Ordering::SeqCst);
            if attempt < self.fail_count {
                Err("Simulated failure".into())
            } else {
                Ok(ToolResult::success("test_id", format!("Success on attempt {}", attempt + 1)))
            }
        }
    }

    struct TestSlowTool {
        delay: Duration,
    }

    #[async_trait]
    impl ToolFunction for TestSlowTool {
        async fn execute(&self, _input: Value) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
            sleep(self.delay).await;
            Ok(ToolResult::success("test_id", "Slow tool completed"))
        }
    }

    #[tokio::test]
    async fn test_successful_execution() {
        let mut registry = ToolRegistry::new();
        let tool_def = Tool::new("test_tool", "Test tool")
            .build();

        let attempts = Arc::new(AtomicUsize::new(0));
        registry
            .register(
                "test_tool",
                tool_def,
                Box::new(TestRetryTool {
                    attempts,
                    fail_count: 0, // Don't fail
                }),
            )
            .unwrap();

        let executor = ToolExecutor::new(Arc::new(registry));
        let tool_use = ToolUse {
            id: "test_id".to_string(),
            name: "test_tool".to_string(),
            input: json!({}),
        };

        let result = executor.execute_with_retry(&tool_use).await.unwrap();
        if let crate::types::ToolResultContent::Text(content) = result.content {
            assert_eq!(content, "Success on attempt 1");
        } else {
            panic!("Expected text content");
        }
    }

    #[tokio::test]
    async fn test_retry_logic() {
        let mut registry = ToolRegistry::new();
        let tool_def = Tool::new("retry_tool", "Tool that fails then succeeds")
            .build();

        let attempts = Arc::new(AtomicUsize::new(0));
        registry
            .register(
                "retry_tool",
                tool_def,
                Box::new(TestRetryTool {
                    attempts,
                    fail_count: 2, // Fail first 2 attempts
                }),
            )
            .unwrap();

        let config = ToolExecutionConfigBuilder::new()
            .max_retries(3)
            .retry_delay(Duration::from_millis(10))
            .exponential_backoff(false)
            .build();

        let executor = ToolExecutor::with_config(Arc::new(registry), config);
        let tool_use = ToolUse {
            id: "test_id".to_string(),
            name: "retry_tool".to_string(),
            input: json!({}),
        };

        let result = executor.execute_with_retry(&tool_use).await.unwrap();
        if let crate::types::ToolResultContent::Text(content) = result.content {
            assert_eq!(content, "Success on attempt 3");
        } else {
            panic!("Expected text content");
        }
    }

    #[tokio::test]
    async fn test_parallel_execution() {
        let mut registry = ToolRegistry::new();
        let tool_def = Tool::new("slow_tool", "Slow tool for testing parallelism")
            .build();

        registry
            .register(
                "slow_tool",
                tool_def,
                Box::new(TestSlowTool {
                    delay: Duration::from_millis(100),
                }),
            )
            .unwrap();

        let config = ToolExecutionConfigBuilder::new()
            .parallel_execution(true)
            .max_concurrent_tools(3)
            .build();

        let executor = ToolExecutor::with_config(Arc::new(registry), config);

        let tool_uses = vec![
            ToolUse {
                id: "test_1".to_string(),
                name: "slow_tool".to_string(),
                input: json!({}),
            },
            ToolUse {
                id: "test_2".to_string(),
                name: "slow_tool".to_string(),
                input: json!({}),
            },
            ToolUse {
                id: "test_3".to_string(),
                name: "slow_tool".to_string(),
                input: json!({}),
            },
        ];

        let start = std::time::Instant::now();
        let results = executor.execute_multiple(&tool_uses).await;
        let duration = start.elapsed();

        // Should complete in roughly 100ms (parallel) rather than 300ms (sequential)
        assert!(duration < Duration::from_millis(200));
        assert_eq!(results.len(), 3);
        
        for result in results {
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_config_builder() {
        let config = ToolExecutionConfigBuilder::new()
            .max_retries(5)
            .retry_delay(Duration::from_millis(100))
            .exponential_backoff(true)
            .max_retry_delay(Duration::from_secs(5))
            .parallel_execution(false)
            .max_concurrent_tools(2)
            .build();

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.retry_delay, Duration::from_millis(100));
        assert!(config.exponential_backoff);
        assert_eq!(config.max_retry_delay, Duration::from_secs(5));
        assert!(!config.parallel_execution);
        assert_eq!(config.max_concurrent_tools, 2);
    }
} 