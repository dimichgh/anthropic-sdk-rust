//! High-level tool conversation management.
//!
//! This module provides abstractions for managing multi-turn conversations
//! that involve tool use, handling the back-and-forth between Claude and tools.

use std::sync::Arc;
use crate::client::Anthropic;
use crate::types::{Message, ToolChoice, ToolResult, MessageCreateBuilder};
use super::{ToolRegistry, ToolExecutor, ToolExecutionConfig, ToolOperationResult, ToolError};

/// High-level tool conversation manager.
///
/// This provides a simplified interface for conducting conversations with Claude
/// that involve tool use, automatically handling tool execution and conversation flow.
pub struct ToolConversation {
    /// The Anthropic client for API calls.
    client: Arc<Anthropic>,
    
    /// Tool registry for executing tools.
    registry: Arc<ToolRegistry>,
    
    /// Tool executor for advanced execution features.
    executor: ToolExecutor,
    
    /// Configuration for the conversation.
    config: ConversationConfig,
}

/// Configuration for tool conversations.
#[derive(Debug, Clone)]
pub struct ConversationConfig {
    /// Maximum number of conversation turns.
    pub max_turns: usize,
    
    /// Model to use for the conversation.
    pub model: String,
    
    /// Maximum tokens per response.
    pub max_tokens: u32,
    
    /// Tool choice strategy.
    pub tool_choice: Option<ToolChoice>,
    
    /// Whether to automatically execute tools.
    pub auto_execute_tools: bool,
    
    /// Tool execution configuration.
    pub execution_config: ToolExecutionConfig,
}

impl Default for ConversationConfig {
    fn default() -> Self {
        Self {
            max_turns: 10,
            model: "claude-3-5-sonnet-latest".to_string(),
            max_tokens: 1024,
            tool_choice: Some(ToolChoice::Auto),
            auto_execute_tools: true,
            execution_config: ToolExecutionConfig::default(),
        }
    }
}

impl ToolConversation {
    /// Create a new tool conversation.
    pub fn new(client: Arc<Anthropic>, registry: Arc<ToolRegistry>) -> Self {
        let executor = ToolExecutor::new(registry.clone());
        Self {
            client,
            registry: registry.clone(),
            executor,
            config: ConversationConfig::default(),
        }
    }

    /// Create a new tool conversation with custom configuration.
    pub fn with_config(
        client: Arc<Anthropic>,
        registry: Arc<ToolRegistry>,
        config: ConversationConfig,
    ) -> Self {
        let executor = ToolExecutor::with_config(registry.clone(), config.execution_config.clone());
        Self {
            client,
            registry: registry.clone(),
            executor,
            config,
        }
    }

    /// Start a conversation with an initial user message.
    ///
    /// This method initiates a conversation and returns the first response from Claude.
    /// If Claude uses tools, they will be automatically executed if `auto_execute_tools` is enabled.
    pub async fn start(&self, user_message: impl Into<String>) -> ToolOperationResult<Message> {
        let tools = self.registry.get_tool_definitions();
        
        let mut builder = MessageCreateBuilder::new(&self.config.model, self.config.max_tokens)
            .user(user_message.into());

        // Add tools if available
        if !tools.is_empty() {
            builder = builder.tools(tools);
            
            if let Some(ref tool_choice) = self.config.tool_choice {
                builder = builder.tool_choice(tool_choice.clone());
            }
        }

        let message = self.client.messages()
            .create(builder.build())
            .await
            .map_err(|e| ToolError::ExecutionFailed { source: e.into() })?;

        Ok(message)
    }

    /// Continue a conversation by processing tool uses and getting the next response.
    ///
    /// This method takes a message that may contain tool use requests, executes the tools,
    /// and returns Claude's response incorporating the tool results.
    pub async fn continue_with_tools(&self, message: &Message) -> ToolOperationResult<Option<Message>> {
        let tool_uses = self.executor.extract_tool_uses(message);
        
        if tool_uses.is_empty() {
            return Ok(None);
        }

        if !self.config.auto_execute_tools {
            // Return without executing tools - let the caller handle execution
            return Ok(None);
        }

        // Execute all tools
        let tool_results = self.executor.execute_multiple(&tool_uses).await;
        
        // Convert execution results to tool results
        let mut results = Vec::new();
        for (tool_use, result) in tool_uses.iter().zip(tool_results.iter()) {
            match result {
                Ok(tool_result) => results.push(tool_result.clone()),
                Err(error) => {
                    results.push(ToolResult::error(
                        tool_use.id.clone(),
                        format!("Tool execution failed: {}", error),
                    ));
                }
            }
        }

        // Create a follow-up message with tool results
        use crate::types::messages::{MessageContent, ContentBlockParam};
        
        // Convert tool results to content blocks
        let tool_result_blocks: Vec<ContentBlockParam> = results.into_iter().map(|result| {
            // Convert ToolResultContent to String for ContentBlockParam::ToolResult
            let content_string = match result.content {
                crate::types::ToolResultContent::Text(text) => Some(text),
                crate::types::ToolResultContent::Json(json) => Some(json.to_string()),
                crate::types::ToolResultContent::Blocks(blocks) => {
                    // Convert blocks to a simple text representation
                    let text_parts: Vec<String> = blocks.into_iter().map(|block| {
                        match block {
                            crate::types::ToolResultBlock::Text { text } => text,
                            crate::types::ToolResultBlock::Image { .. } => "[Image]".to_string(),
                        }
                    }).collect();
                    Some(text_parts.join("\n"))
                }
            };
            
            ContentBlockParam::ToolResult {
                tool_use_id: result.tool_use_id,
                content: content_string,
                is_error: result.is_error,
            }
        }).collect();
        
        let mut builder = MessageCreateBuilder::new(&self.config.model, self.config.max_tokens)
            .user(MessageContent::Blocks(tool_result_blocks));
            
        // Add tools again for potential follow-up tool use
        let tools = self.registry.get_tool_definitions();
        if !tools.is_empty() {
            builder = builder.tools(tools);
            
            if let Some(ref tool_choice) = self.config.tool_choice {
                builder = builder.tool_choice(tool_choice.clone());
            }
        }
        
        let next_message = self.client.messages()
            .create(builder.build())
            .await
            .map_err(|e| ToolError::ExecutionFailed { source: e.into() })?;

        Ok(Some(next_message))
    }

    /// Execute a complete conversation until completion or max turns reached.
    ///
    /// This method manages the entire conversation flow, automatically executing tools
    /// and continuing the conversation until Claude provides a final response.
    pub async fn execute_until_complete(&self, initial_message: impl Into<String>) -> ToolOperationResult<Message> {
        let mut current_message = self.start(initial_message).await?;
        let mut turn_count = 1;

        while turn_count < self.config.max_turns {
            match self.continue_with_tools(&current_message).await? {
                Some(next_message) => {
                    current_message = next_message;
                    turn_count += 1;
                }
                None => {
                    // No more tools to execute, conversation is complete
                    break;
                }
            }
        }

        if turn_count >= self.config.max_turns {
            return Err(ToolError::ExecutionFailed {
                source: "Conversation exceeded maximum turns".to_string().into(),
            });
        }

        Ok(current_message)
    }

    /// Get the tool registry.
    pub fn registry(&self) -> &Arc<ToolRegistry> {
        &self.registry
    }

    /// Get the tool executor.
    pub fn executor(&self) -> &ToolExecutor {
        &self.executor
    }

    /// Get the conversation configuration.
    pub fn config(&self) -> &ConversationConfig {
        &self.config
    }

    /// Update the conversation configuration.
    pub fn set_config(&mut self, config: ConversationConfig) {
        self.config = config;
        self.executor.set_config(self.config.execution_config.clone());
    }


}

/// Builder for creating conversation configurations.
pub struct ConversationConfigBuilder {
    config: ConversationConfig,
}

impl ConversationConfigBuilder {
    /// Create a new configuration builder.
    pub fn new() -> Self {
        Self {
            config: ConversationConfig::default(),
        }
    }

    /// Set the maximum number of conversation turns.
    pub fn max_turns(mut self, max_turns: usize) -> Self {
        self.config.max_turns = max_turns;
        self
    }

    /// Set the model to use.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }

    /// Set the maximum tokens per response.
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.config.max_tokens = max_tokens;
        self
    }

    /// Set the tool choice strategy.
    pub fn tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.config.tool_choice = Some(tool_choice);
        self
    }

    /// Enable or disable automatic tool execution.
    pub fn auto_execute_tools(mut self, enabled: bool) -> Self {
        self.config.auto_execute_tools = enabled;
        self
    }

    /// Set the tool execution configuration.
    pub fn execution_config(mut self, config: ToolExecutionConfig) -> Self {
        self.config.execution_config = config;
        self
    }

    /// Build the configuration.
    pub fn build(self) -> ConversationConfig {
        self.config
    }
}

impl Default for ConversationConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_config_builder() {
        let config = ConversationConfigBuilder::new()
            .max_turns(5)
            .model("claude-3-5-sonnet-latest")
            .max_tokens(2048)
            .tool_choice(ToolChoice::Any)
            .auto_execute_tools(false)
            .build();

        assert_eq!(config.max_turns, 5);
        assert_eq!(config.model, "claude-3-5-sonnet-latest");
        assert_eq!(config.max_tokens, 2048);
        assert_eq!(config.tool_choice, Some(ToolChoice::Any));
        assert!(!config.auto_execute_tools);
    }

    #[test]
    fn test_default_config() {
        let config = ConversationConfig::default();
        assert_eq!(config.max_turns, 10);
        assert_eq!(config.model, "claude-3-5-sonnet-latest");
        assert_eq!(config.max_tokens, 1024);
        assert_eq!(config.tool_choice, Some(ToolChoice::Auto));
        assert!(config.auto_execute_tools);
    }
} 