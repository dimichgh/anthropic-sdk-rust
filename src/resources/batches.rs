use crate::types::{
    MessageBatch, BatchCreateParams, BatchListParams, BatchList, BatchResult,
    AnthropicError, Result,
};
use crate::http::HttpClient;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Resource for managing message batches
#[derive(Debug, Clone)]
pub struct BatchesResource {
    http_client: Arc<HttpClient>,
}

impl BatchesResource {
    /// Create a new batches resource
    pub fn new(http_client: Arc<HttpClient>) -> Self {
        Self { http_client }
    }

    /// Create a new message batch
    /// 
    /// # Arguments
    /// * `params` - Parameters for creating the batch
    /// 
    /// # Returns
    /// A new `MessageBatch` object with status information
    /// 
    /// # Errors
    /// Returns an error if the request fails or if the batch parameters are invalid
    pub async fn create(&self, params: BatchCreateParams) -> Result<MessageBatch> {
        let response = self
            .http_client
            .post("/v1/messages/batches")
            .json(&params)
            .send()
            .await?;

        let batch: MessageBatch = response.json().await?;
        Ok(batch)
    }

    /// Retrieve a specific message batch by ID
    /// 
    /// # Arguments
    /// * `batch_id` - The ID of the batch to retrieve
    /// 
    /// # Returns
    /// The `MessageBatch` object with current status
    /// 
    /// # Errors
    /// Returns an error if the batch is not found or if the request fails
    pub async fn get(&self, batch_id: &str) -> Result<MessageBatch> {
        let response = self
            .http_client
            .get(&format!("/v1/messages/batches/{}", batch_id))
            .send()
            .await?;

        let batch: MessageBatch = response.json().await?;
        Ok(batch)
    }

    /// List message batches
    /// 
    /// # Arguments
    /// * `params` - Optional parameters for pagination and filtering
    /// 
    /// # Returns
    /// A `BatchList` containing batches and pagination information
    /// 
    /// # Errors
    /// Returns an error if the request fails
    pub async fn list(&self, params: Option<BatchListParams>) -> Result<BatchList> {
        let mut request = self.http_client.get("/v1/messages/batches");

        if let Some(params) = params {
            if let Some(after) = params.after {
                request = request.query(&[("after", after)]);
            }
            if let Some(limit) = params.limit {
                request = request.query(&[("limit", limit.to_string())]);
            }
        }

        let response = request.send().await?;
        let batch_list: BatchList = response.json().await?;
        Ok(batch_list)
    }

    /// Cancel a message batch
    /// 
    /// # Arguments
    /// * `batch_id` - The ID of the batch to cancel
    /// 
    /// # Returns
    /// The updated `MessageBatch` object with cancellation status
    /// 
    /// # Errors
    /// Returns an error if the batch cannot be cancelled or if the request fails
    pub async fn cancel(&self, batch_id: &str) -> Result<MessageBatch> {
        let response = self
            .http_client
            .post(&format!("/v1/messages/batches/{}/cancel", batch_id))
            .send()
            .await?;

        let batch: MessageBatch = response.json().await?;
        Ok(batch)
    }

    /// Get the results of a completed batch
    /// 
    /// # Arguments
    /// * `batch_id` - The ID of the completed batch
    /// 
    /// # Returns
    /// A vector of `BatchResult` objects containing the results
    /// 
    /// # Errors
    /// Returns an error if the batch is not completed or if the request fails
    pub async fn get_results(&self, batch_id: &str) -> Result<Vec<BatchResult>> {
        // First, get the batch to check status and get output file ID
        let batch = self.get(batch_id).await?;

        if !batch.is_complete() {
            return Err(AnthropicError::Other(
                "Batch is not yet completed".to_string(),
            ));
        }

        let output_file_id = batch.output_file_id.ok_or_else(|| {
            AnthropicError::Other("Batch has no output file".to_string())
        })?;

        // Download the results file
        let response = self
            .http_client
            .get(&format!("/v1/files/{}/content", output_file_id))
            .send()
            .await?;

        let content = response.text().await?;

        // Parse JSONL format (each line is a JSON object)
        let mut results = Vec::new();
        for line in content.lines() {
            if !line.trim().is_empty() {
                let result: BatchResult = serde_json::from_str(line)
                    .map_err(|e| AnthropicError::Other(format!("Failed to parse result: {}", e)))?;
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Wait for a batch to complete
    /// 
    /// # Arguments
    /// * `batch_id` - The ID of the batch to wait for
    /// * `poll_interval` - How often to check the status (default: 5 seconds)
    /// * `timeout` - Maximum time to wait (default: 1 hour)
    /// 
    /// # Returns
    /// The completed `MessageBatch` object
    /// 
    /// # Errors
    /// Returns an error if the batch fails, expires, or if the timeout is reached
    pub async fn wait_for_completion(
        &self,
        batch_id: &str,
        poll_interval: Option<Duration>,
        timeout: Option<Duration>,
    ) -> Result<MessageBatch> {
        let poll_interval = poll_interval.unwrap_or(Duration::from_secs(5));
        let timeout = timeout.unwrap_or(Duration::from_secs(3600)); // 1 hour

        let start_time = std::time::Instant::now();

        loop {
            let batch = self.get(batch_id).await?;

            if batch.is_complete() {
                return Ok(batch);
            }

            if batch.has_failed() {
                return Err(AnthropicError::Other(format!(
                    "Batch failed with status: {:?}",
                    batch.processing_status
                )));
            }

            if start_time.elapsed() > timeout {
                return Err(AnthropicError::Timeout);
            }

            sleep(poll_interval).await;
        }
    }

    /// Get the status and progress of a batch
    /// 
    /// # Arguments
    /// * `batch_id` - The ID of the batch to check
    /// 
    /// # Returns
    /// A tuple containing (status, completion_percentage, pending_requests)
    /// 
    /// # Errors
    /// Returns an error if the request fails
    pub async fn get_status(&self, batch_id: &str) -> Result<(crate::types::BatchStatus, f64, u32)> {
        let batch = self.get(batch_id).await?;
        Ok((
            batch.processing_status,
            batch.completion_percentage(),
            batch.pending_requests(),
        ))
    }
}

/// High-level batch processing utilities
impl BatchesResource {
    /// Create and monitor a batch until completion
    /// 
    /// # Arguments
    /// * `params` - Parameters for creating the batch
    /// * `poll_interval` - How often to check status (default: 5 seconds)
    /// 
    /// # Returns
    /// A tuple containing the completed batch and its results
    /// 
    /// # Errors
    /// Returns an error if batch creation, processing, or result retrieval fails
    pub async fn create_and_wait(
        &self,
        params: BatchCreateParams,
        poll_interval: Option<Duration>,
    ) -> Result<(MessageBatch, Vec<BatchResult>)> {
        // Create the batch
        let batch = self.create(params).await?;
        let batch_id = &batch.id;

        println!("Created batch {} with {} requests", batch_id, batch.request_counts.total);

        // Wait for completion
        let completed_batch = self.wait_for_completion(batch_id, poll_interval, None).await?;

        println!("Batch {} completed: {}/{} requests successful", 
            batch_id, 
            completed_batch.request_counts.completed,
            completed_batch.request_counts.total
        );

        // Get results
        let results = self.get_results(batch_id).await?;

        Ok((completed_batch, results))
    }

    /// Monitor batch progress with callbacks
    /// 
    /// # Arguments
    /// * `batch_id` - The ID of the batch to monitor
    /// * `progress_callback` - Called with progress updates
    /// * `poll_interval` - How often to check status
    /// 
    /// # Returns
    /// The completed batch
    /// 
    /// # Errors
    /// Returns an error if monitoring fails
    pub async fn monitor_progress<F>(
        &self,
        batch_id: &str,
        mut progress_callback: F,
        poll_interval: Option<Duration>,
    ) -> Result<MessageBatch>
    where
        F: FnMut(f64, u32, u32), // (percentage, completed, total)
    {
        let poll_interval = poll_interval.unwrap_or(Duration::from_secs(5));
        let mut last_percentage = -1.0;

        loop {
            let batch = self.get(batch_id).await?;
            let percentage = batch.completion_percentage();

            // Only call callback if progress changed
            if (percentage - last_percentage).abs() > 0.01 {
                progress_callback(
                    percentage,
                    batch.request_counts.completed,
                    batch.request_counts.total,
                );
                last_percentage = percentage;
            }

            if batch.is_complete() {
                return Ok(batch);
            }

            if batch.has_failed() {
                return Err(AnthropicError::Other(format!(
                    "Batch failed with status: {:?}",
                    batch.processing_status
                )));
            }

            sleep(poll_interval).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BatchRequest, BatchStatus, BatchRequestCounts};

    #[test]
    fn test_batch_completion_check() {
        let batch = MessageBatch {
            id: "batch_test".to_string(),
            object_type: "message_batch".to_string(),
            processing_status: BatchStatus::Completed,
            request_counts: BatchRequestCounts {
                total: 10,
                completed: 10,
                failed: 0,
            },
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now(),
            ended_at: Some(chrono::Utc::now()),
            input_file_id: "file_input".to_string(),
            output_file_id: Some("file_output".to_string()),
            error_file_id: None,
            metadata: std::collections::HashMap::new(),
        };

        assert!(batch.is_complete());
        assert!(!batch.has_failed());
        assert!(!batch.can_cancel());
        assert_eq!(batch.completion_percentage(), 100.0);
        assert_eq!(batch.pending_requests(), 0);
    }

    #[test]
    fn test_batch_request_creation() {
        let request = BatchRequest::new("test_req", "claude-3-5-sonnet-latest", 1024)
            .user("Hello, world!")
            .system("You are helpful")
            .temperature(0.7)
            .build();

        assert_eq!(request.custom_id, "test_req");
        assert_eq!(request.method, "POST");
        assert_eq!(request.url, "/v1/messages");
        assert!(request.body.messages.len() > 0);
        assert_eq!(request.body.system, Some("You are helpful".to_string()));
        assert_eq!(request.body.temperature, Some(0.7));
    }
} 