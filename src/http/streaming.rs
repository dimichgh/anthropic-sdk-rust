//! HTTP streaming client for Server-Sent Events (SSE) processing.
//!
//! This module handles the HTTP layer for streaming responses from the Anthropic API,
//! parsing SSE events and converting them into MessageStreamEvent objects.

use std::pin::Pin;
use std::task::{Context, Poll};
use futures::Stream;
use pin_project::pin_project;
use reqwest::Response;
use serde_json;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;

use crate::types::{MessageStreamEvent, AnthropicError, Result};

/// Configuration for SSE streaming requests.
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Buffer size for the event channel
    pub buffer_size: usize,
    /// Timeout for individual events (in seconds)
    pub event_timeout: Option<u64>,
    /// Whether to retry on connection errors
    pub retry_on_error: bool,
    /// Maximum retry attempts
    pub max_retries: Option<u32>,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            event_timeout: Some(30),
            retry_on_error: true,
            max_retries: Some(3),
        }
    }
}

/// HTTP streaming client for processing Server-Sent Events.
///
/// This client handles the low-level HTTP streaming and SSE parsing,
/// converting raw SSE data into structured MessageStreamEvent objects.
#[pin_project]
pub struct HttpStreamClient {
    /// The underlying SSE event stream
    #[pin]
    event_stream: Pin<Box<dyn Stream<Item = Result<MessageStreamEvent>> + Send>>,
    
    /// Broadcast sender for distributing events
    event_sender: broadcast::Sender<MessageStreamEvent>,
    
    /// Configuration for the stream
    config: StreamConfig,
    
    /// Whether the stream has ended
    ended: bool,
    
    /// Request ID from response headers
    request_id: Option<String>,
}

impl HttpStreamClient {
    /// Create a new HTTP stream client from a response.
    ///
    /// This method takes a reqwest Response (which should be from a streaming endpoint)
    /// and converts it into a stream of MessageStreamEvent objects.
    pub async fn from_response(response: Response, config: StreamConfig) -> Result<Self> {
        let request_id = response.headers()
            .get("request-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Create the event channel
        let (event_sender, _) = broadcast::channel(config.buffer_size);

        // Convert the HTTP response into an SSE stream
        let event_stream = Self::create_event_stream(response).await?;

        Ok(Self {
            event_stream: Box::pin(event_stream),
            event_sender,
            config,
            ended: false,
            request_id,
        })
    }

    /// Create a stream of MessageStreamEvent from an HTTP response.
    async fn create_event_stream(
        response: Response,
    ) -> Result<impl Stream<Item = Result<MessageStreamEvent>>> {
        // Check that we got a successful response
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AnthropicError::from_status(status.as_u16(), text));
        }

        // Convert the response into a byte stream
        let byte_stream = response.bytes_stream();

        // Use eventsource-stream to parse SSE events
        use eventsource_stream::Eventsource;
        
        let sse_stream = byte_stream
            .eventsource()
            .map(|result| {
                match result {
                    Ok(event) => {
                        // Parse the SSE event data based on event type
                        match event.event.as_str() {
                            // Handle Anthropic API format (event type is "message", data contains the event)
                            "message" | "" => {
                                match serde_json::from_str::<MessageStreamEvent>(&event.data) {
                                    Ok(stream_event) => Ok(stream_event),
                                    Err(e) => Err(AnthropicError::StreamError(
                                        format!("Failed to parse SSE event: {}", e)
                                    )),
                                }
                            }
                            // Handle custom gateway format (event type IS the message event type)
                            "message_start" => {
                                // Parse the message data - handle both direct and nested formats
                                match serde_json::from_str::<crate::types::Message>(&event.data) {
                                    Ok(message) => Ok(MessageStreamEvent::MessageStart { message }),
                                    Err(_) => {
                                        // Try parsing as a wrapped message (custom gateway format)
                                        match serde_json::from_str::<serde_json::Value>(&event.data) {
                                            Ok(value) => {
                                                if let Some(message_value) = value.get("message") {
                                                    match serde_json::from_value::<crate::types::Message>(message_value.clone()) {
                                                        Ok(message) => Ok(MessageStreamEvent::MessageStart { message }),
                                                        Err(e) => Err(AnthropicError::StreamError(
                                                            format!("Failed to parse nested message: {}", e)
                                                        )),
                                                    }
                                                } else {
                                                    Err(AnthropicError::StreamError(
                                                        "message_start event missing message field".to_string()
                                                    ))
                                                }
                                            }
                                            Err(e) => Err(AnthropicError::StreamError(
                                                format!("Failed to parse message_start as JSON: {}", e)
                                            )),
                                        }
                                    }
                                }
                            }
                            "content_block_start" => {
                                // Parse as a generic JSON value first to extract index and content_block
                                match serde_json::from_str::<serde_json::Value>(&event.data) {
                                    Ok(value) => {
                                        let index = value["index"].as_u64().unwrap_or(0) as usize;
                                        match serde_json::from_value::<crate::types::ContentBlock>(value["content_block"].clone()) {
                                            Ok(content_block) => Ok(MessageStreamEvent::ContentBlockStart { content_block, index }),
                                            Err(e) => Err(AnthropicError::StreamError(
                                                format!("Failed to parse content_block in content_block_start: {}", e)
                                            )),
                                        }
                                    }
                                    Err(e) => Err(AnthropicError::StreamError(
                                        format!("Failed to parse content_block_start event: {}", e)
                                    )),
                                }
                            }
                            "content_block_delta" => {
                                // Parse as a generic JSON value first to extract index and delta
                                match serde_json::from_str::<serde_json::Value>(&event.data) {
                                    Ok(value) => {
                                        let index = value["index"].as_u64().unwrap_or(0) as usize;
                                        match serde_json::from_value::<crate::types::ContentBlockDelta>(value["delta"].clone()) {
                                            Ok(delta) => Ok(MessageStreamEvent::ContentBlockDelta { delta, index }),
                                            Err(e) => Err(AnthropicError::StreamError(
                                                format!("Failed to parse delta in content_block_delta: {}", e)
                                            )),
                                        }
                                    }
                                    Err(e) => Err(AnthropicError::StreamError(
                                        format!("Failed to parse content_block_delta event: {}", e)
                                    )),
                                }
                            }
                            "content_block_stop" => {
                                // Parse as a generic JSON value to extract index
                                match serde_json::from_str::<serde_json::Value>(&event.data) {
                                    Ok(value) => {
                                        let index = value["index"].as_u64().unwrap_or(0) as usize;
                                        Ok(MessageStreamEvent::ContentBlockStop { index })
                                    }
                                    Err(e) => Err(AnthropicError::StreamError(
                                        format!("Failed to parse content_block_stop event: {}", e)
                                    )),
                                }
                            }
                            "message_delta" => {
                                // Parse as a generic JSON value to extract delta and usage
                                match serde_json::from_str::<serde_json::Value>(&event.data) {
                                    Ok(value) => {
                                        let delta = serde_json::from_value::<crate::types::MessageDelta>(value["delta"].clone())
                                            .map_err(|e| AnthropicError::StreamError(format!("Failed to parse delta: {}", e)))?;
                                        let usage = serde_json::from_value::<crate::types::MessageDeltaUsage>(value["usage"].clone())
                                            .map_err(|e| AnthropicError::StreamError(format!("Failed to parse usage: {}", e)))?;
                                        Ok(MessageStreamEvent::MessageDelta { delta, usage })
                                    }
                                    Err(e) => Err(AnthropicError::StreamError(
                                        format!("Failed to parse message_delta event: {}", e)
                                    )),
                                }
                            }
                            "message_stop" => {
                                // Message stop doesn't need data parsing
                                Ok(MessageStreamEvent::MessageStop)
                            }
                            // Handle other event types
                            "ping" => {
                                // Ignore ping events
                                Err(AnthropicError::StreamError("ping".to_string()))
                            }
                            event_type => {
                                // Log unknown event types but don't fail
                                tracing::debug!("Unknown SSE event type: {}", event_type);
                                Err(AnthropicError::StreamError(
                                    format!("Unknown event type: {}", event_type)
                                ))
                            }
                        }
                    }
                    Err(e) => Err(AnthropicError::StreamError(
                        format!("SSE stream error: {}", e)
                    )),
                }
            })
            .filter_map(|result| {
                match result {
                    Ok(event) => Some(Ok(event)),
                    Err(e) if e.to_string().contains("ping") => None, // Filter out ping errors
                    Err(e) => Some(Err(e)),
                }
            });

        Ok(sse_stream)
    }

    /// Get the request ID from the response headers.
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }

    /// Get the stream configuration.
    pub fn config(&self) -> &StreamConfig {
        &self.config
    }

    /// Check if the stream has ended.
    pub fn ended(&self) -> bool {
        self.ended
    }

    /// Get a receiver for the broadcast channel.
    ///
    /// This allows multiple consumers to receive the same stream events.
    pub fn subscribe(&self) -> broadcast::Receiver<MessageStreamEvent> {
        self.event_sender.subscribe()
    }
}

impl Stream for HttpStreamClient {
    type Item = Result<MessageStreamEvent>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        match this.event_stream.poll_next(cx) {
            Poll::Ready(Some(Ok(event))) => {
                // Broadcast the event to all subscribers
                let _ = this.event_sender.send(event.clone());
                
                // Check if this is a terminal event
                if matches!(event, MessageStreamEvent::MessageStop) {
                    *this.ended = true;
                }
                
                Poll::Ready(Some(Ok(event)))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => {
                *this.ended = true;
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Builder for creating HTTP streaming requests.
#[derive(Debug, Clone)]
pub struct StreamRequestBuilder {
    /// HTTP client for making requests
    client: reqwest::Client,
    /// Base URL for the API
    base_url: String,
    /// Request headers
    headers: reqwest::header::HeaderMap,
    /// Stream configuration
    config: StreamConfig,
}

impl StreamRequestBuilder {
    /// Create a new stream request builder.
    pub fn new(client: reqwest::Client, base_url: String) -> Self {
        Self {
            client,
            base_url,
            headers: reqwest::header::HeaderMap::new(),
            config: StreamConfig::default(),
        }
    }

    /// Add a header to the request.
    pub fn header(mut self, key: &str, value: &str) -> Self {
        if let (Ok(key), Ok(value)) = (
            reqwest::header::HeaderName::from_bytes(key.as_bytes()),
            reqwest::header::HeaderValue::from_str(value),
        ) {
            self.headers.insert(key, value);
        }
        self
    }

    /// Set the stream configuration.
    pub fn config(mut self, config: StreamConfig) -> Self {
        self.config = config;
        self
    }

    /// Make a streaming POST request.
    pub async fn post_stream<T: serde::Serialize>(
        self,
        endpoint: &str,
        body: &T,
    ) -> Result<HttpStreamClient> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        
        let mut headers = self.headers;
        headers.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static("text/event-stream"),
        );
        headers.insert(
            reqwest::header::CACHE_CONTROL,
            reqwest::header::HeaderValue::from_static("no-cache"),
        );

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(body)
            .send()
            .await
            .map_err(|e| AnthropicError::Connection { message: e.to_string() })?;

        HttpStreamClient::from_response(response, self.config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_config_default() {
        let config = StreamConfig::default();
        assert_eq!(config.buffer_size, 1000);
        assert_eq!(config.event_timeout, Some(30));
        assert_eq!(config.retry_on_error, true);
        assert_eq!(config.max_retries, Some(3));
    }

    #[test]
    fn test_stream_request_builder() {
        let client = reqwest::Client::new();
        let builder = StreamRequestBuilder::new(client, "https://api.anthropic.com".to_string())
            .header("Authorization", "Bearer test-key")
            .config(StreamConfig {
                buffer_size: 500,
                ..Default::default()
            });

        assert_eq!(builder.base_url, "https://api.anthropic.com");
        assert_eq!(builder.config.buffer_size, 500);
        assert!(builder.headers.contains_key("authorization"));
    }

    #[tokio::test]
    async fn test_sse_event_parsing() {
        // Test that we can parse a sample SSE event
        let event_data = r#"{"type":"message_start","message":{"id":"msg_123","type":"message","role":"assistant","content":[],"model":"claude-3-5-sonnet-latest","stop_reason":null,"stop_sequence":null,"usage":{"input_tokens":10,"output_tokens":0,"cache_creation_input_tokens":null,"cache_read_input_tokens":null,"server_tool_use":null,"service_tier":null}}}"#;
        
        let parsed: std::result::Result<MessageStreamEvent, _> = serde_json::from_str(event_data);
        assert!(parsed.is_ok());
        
        if let Ok(MessageStreamEvent::MessageStart { message }) = parsed {
            assert_eq!(message.id, "msg_123");
            assert_eq!(message.usage.input_tokens, 10);
        } else {
            panic!("Expected MessageStart event");
        }
    }
} 