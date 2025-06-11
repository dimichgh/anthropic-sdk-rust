use crate::types::{Message, MessageCreateParams};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// A batch request for processing multiple messages efficiently
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBatch {
    /// Unique identifier for the batch
    pub id: String,
    
    /// The type of object (always "message_batch")
    #[serde(rename = "type")]
    pub object_type: String,
    
    /// Current processing status of the batch
    pub processing_status: BatchStatus,
    
    /// Total number of requests in the batch
    pub request_counts: BatchRequestCounts,
    
    /// When the batch was created
    pub created_at: DateTime<Utc>,
    
    /// When the batch processing will expire
    pub expires_at: DateTime<Utc>,
    
    /// When the batch processing was completed (if applicable)
    pub ended_at: Option<DateTime<Utc>>,
    
    /// File ID containing the batch requests
    pub input_file_id: String,
    
    /// File ID containing the batch results (if completed)
    pub output_file_id: Option<String>,
    
    /// File ID containing any errors (if applicable)
    pub error_file_id: Option<String>,
    
    /// Custom metadata for the batch
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Status of batch processing
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BatchStatus {
    /// Batch is validating inputs
    Validating,
    
    /// Batch is in the processing queue
    InProgress,
    
    /// Batch is being processed
    Finalizing,
    
    /// Batch processing completed successfully
    Completed,
    
    /// Batch processing expired before completion
    Expired,
    
    /// Batch processing was cancelled
    Cancelling,
    
    /// Batch processing was cancelled
    Cancelled,
    
    /// Batch processing failed
    Failed,
}

impl BatchStatus {
    /// Check if the batch is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            BatchStatus::Completed 
            | BatchStatus::Expired 
            | BatchStatus::Cancelled 
            | BatchStatus::Failed
        )
    }
    
    /// Check if the batch is still being processed
    pub fn is_processing(&self) -> bool {
        matches!(
            self,
            BatchStatus::Validating 
            | BatchStatus::InProgress 
            | BatchStatus::Finalizing
        )
    }
}

/// Count of requests in different states within a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequestCounts {
    /// Total number of requests in the batch
    pub total: u32,
    
    /// Number of requests completed successfully
    pub completed: u32,
    
    /// Number of requests that failed
    pub failed: u32,
}

impl BatchRequestCounts {
    /// Calculate the number of pending requests
    pub fn pending(&self) -> u32 {
        self.total.saturating_sub(self.completed + self.failed)
    }
    
    /// Calculate completion percentage
    pub fn completion_percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.completed as f64 / self.total as f64) * 100.0
        }
    }
}

/// Individual request within a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    /// Custom ID for this request (for result matching)
    pub custom_id: String,
    
    /// HTTP method (always "POST" for messages)
    pub method: String,
    
    /// API endpoint URL
    pub url: String,
    
    /// Request body containing message parameters
    pub body: MessageCreateParams,
}

impl BatchRequest {
    /// Create a new batch request
    pub fn new(custom_id: impl Into<String>, model: impl Into<String>, max_tokens: u32) -> BatchRequestBuilder {
        BatchRequestBuilder {
            custom_id: custom_id.into(),
            method: "POST".to_string(),
            url: "/v1/messages".to_string(),
            body: MessageCreateParams {
                model: model.into(),
                max_tokens,
                messages: Vec::new(),
                system: None,
                temperature: None,
                top_p: None,
                top_k: None,
                stop_sequences: None,
                stream: Some(false), // Batches don't support streaming
                tools: None,
                tool_choice: None,
                metadata: None,
            },
        }
    }
}

/// Builder for creating batch requests
#[derive(Debug, Clone)]
pub struct BatchRequestBuilder {
    custom_id: String,
    method: String,
    url: String,
    body: MessageCreateParams,
}

impl BatchRequestBuilder {
    /// Add a user message to the request
    pub fn user(mut self, content: impl Into<String>) -> Self {
        use crate::types::{MessageParam, Role, MessageContent};
        
        self.body.messages.push(MessageParam {
            role: Role::User,
            content: MessageContent::Text(content.into()),
        });
        self
    }
    
    /// Add an assistant message to the request
    pub fn assistant(mut self, content: impl Into<String>) -> Self {
        use crate::types::{MessageParam, Role, MessageContent};
        
        self.body.messages.push(MessageParam {
            role: Role::Assistant,
            content: MessageContent::Text(content.into()),
        });
        self
    }
    
    /// Set the system prompt for the request
    pub fn system(mut self, system: impl Into<String>) -> Self {
        self.body.system = Some(system.into());
        self
    }
    
    /// Set the temperature for the request
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.body.temperature = Some(temperature);
        self
    }
    
    /// Set the top_p for the request
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.body.top_p = Some(top_p);
        self
    }
    
    /// Set the top_k for the request
    pub fn top_k(mut self, top_k: u32) -> Self {
        self.body.top_k = Some(top_k);
        self
    }
    
    /// Add stop sequences for the request
    pub fn stop_sequences(mut self, stop_sequences: Vec<String>) -> Self {
        self.body.stop_sequences = Some(stop_sequences);
        self
    }
    
    /// Add tools to the request
    pub fn tools(mut self, tools: Vec<crate::types::Tool>) -> Self {
        self.body.tools = Some(tools);
        self
    }
    
    /// Set tool choice for the request
    pub fn tool_choice(mut self, tool_choice: crate::types::ToolChoice) -> Self {
        self.body.tool_choice = Some(tool_choice);
        self
    }
    
    /// Add metadata to the request
    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.body.metadata = Some(metadata);
        self
    }
    
    /// Build the batch request
    pub fn build(self) -> BatchRequest {
        BatchRequest {
            custom_id: self.custom_id,
            method: self.method,
            url: self.url,
            body: self.body,
        }
    }
}

/// Result of a single request within a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    /// Custom ID from the original request
    pub custom_id: String,
    
    /// HTTP response for this request
    pub response: BatchResponse,
}

/// HTTP response for a batch request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse {
    /// HTTP status code
    pub status_code: u16,
    
    /// Response headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
    
    /// Response body (success or error)
    pub body: BatchResponseBody,
}

/// Response body for a batch request
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BatchResponseBody {
    /// Successful message response
    Success(Message),
    
    /// Error response
    Error(BatchError),
}

/// Error response for a failed batch request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchError {
    /// Error type
    #[serde(rename = "type")]
    pub error_type: String,
    
    /// Error message
    pub message: String,
    
    /// Additional error details
    #[serde(default)]
    pub details: HashMap<String, serde_json::Value>,
}

/// Parameters for creating a new batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreateParams {
    /// Array of individual requests
    pub requests: Vec<BatchRequest>,
    
    /// Custom metadata for the batch
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, String>,
    
    /// Completion window for the batch (in hours, default 24)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_window: Option<u32>,
}

impl BatchCreateParams {
    /// Create new batch parameters
    pub fn new(requests: Vec<BatchRequest>) -> Self {
        Self {
            requests,
            metadata: HashMap::new(),
            completion_window: None,
        }
    }
    
    /// Add metadata to the batch
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }
    
    /// Set completion window in hours
    pub fn with_completion_window(mut self, hours: u32) -> Self {
        self.completion_window = Some(hours);
        self
    }
}

/// Parameters for listing batches
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BatchListParams {
    /// A cursor for use in pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    
    /// Number of items to return (1-100, default 20)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

impl BatchListParams {
    /// Create new list parameters
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set pagination cursor
    pub fn after(mut self, after: impl Into<String>) -> Self {
        self.after = Some(after.into());
        self
    }
    
    /// Set result limit
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit.clamp(1, 100));
        self
    }
}

/// Response containing a list of batches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchList {
    /// List of batch objects
    pub data: Vec<MessageBatch>,
    
    /// Whether there are more items available
    pub has_more: bool,
    
    /// First ID in the current page
    pub first_id: Option<String>,
    
    /// Last ID in the current page
    pub last_id: Option<String>,
}

impl MessageBatch {
    /// Check if the batch is complete
    pub fn is_complete(&self) -> bool {
        self.processing_status == BatchStatus::Completed
    }
    
    /// Check if the batch has failed
    pub fn has_failed(&self) -> bool {
        matches!(
            self.processing_status,
            BatchStatus::Failed | BatchStatus::Expired
        )
    }
    
    /// Check if the batch can be cancelled
    pub fn can_cancel(&self) -> bool {
        self.processing_status.is_processing()
    }
    
    /// Get completion percentage
    pub fn completion_percentage(&self) -> f64 {
        self.request_counts.completion_percentage()
    }
    
    /// Get the number of pending requests
    pub fn pending_requests(&self) -> u32 {
        self.request_counts.pending()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_status_terminal() {
        assert!(BatchStatus::Completed.is_terminal());
        assert!(BatchStatus::Failed.is_terminal());
        assert!(BatchStatus::Cancelled.is_terminal());
        assert!(BatchStatus::Expired.is_terminal());
        assert!(BatchStatus::InProgress.is_processing());
    }

    #[test]
    fn test_batch_request_builder() {
        let request = BatchRequest::new("test1", "claude-3-5-sonnet-latest", 1024)
            .user("Hello, world!")
            .system("You are a helpful assistant")
            .temperature(0.7)
            .build();

        assert_eq!(request.custom_id, "test1");
        assert_eq!(request.method, "POST");
        assert_eq!(request.url, "/v1/messages");
        assert_eq!(request.body.model, "claude-3-5-sonnet-latest");
        assert_eq!(request.body.max_tokens, 1024);
        assert_eq!(request.body.messages.len(), 1);
        assert_eq!(request.body.system, Some("You are a helpful assistant".to_string()));
        assert_eq!(request.body.temperature, Some(0.7));
    }

    #[test]
    fn test_request_counts() {
        let counts = BatchRequestCounts {
            total: 100,
            completed: 75,
            failed: 10,
        };

        assert_eq!(counts.pending(), 15);
        assert_eq!(counts.completion_percentage(), 75.0);
    }

    #[test]
    fn test_batch_create_params() {
        let requests = vec![
            BatchRequest::new("req1", "claude-3-5-sonnet-latest", 1024)
                .user("Hello")
                .build(),
        ];

        let params = BatchCreateParams::new(requests)
            .with_completion_window(12);

        assert_eq!(params.requests.len(), 1);
        assert_eq!(params.completion_window, Some(12));
    }
} 