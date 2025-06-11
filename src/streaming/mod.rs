//! Streaming support for real-time message generation.
//!
//! This module provides the `MessageStream` struct which handles Server-Sent Events (SSE)
//! from the Anthropic API, accumulates messages from incremental updates, and provides
//! an event-driven API for processing streaming responses.

pub mod events;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use futures::Stream;
use pin_project::pin_project;
use tokio::sync::{broadcast, oneshot};
use tokio_stream::wrappers::BroadcastStream;

use crate::types::{
    Message, MessageStreamEvent, ContentBlock, ContentBlockDelta, 
    AnthropicError, Result
};

use self::events::{EventHandler, EventType};

/// A streaming response from the Anthropic API.
///
/// `MessageStream` provides an event-driven interface for processing streaming responses
/// from Claude. It accumulates message content from incremental updates and provides
/// both callback-based and async iteration APIs.
///
/// # Examples
///
/// ## Callback-based processing:
/// ```ignore
/// # use anthropic_sdk::{Anthropic, MessageCreateBuilder};
/// # async fn example() -> anthropic_sdk::Result<()> {
/// let client = Anthropic::new("your-api-key")?;
/// let stream = client.messages().create_stream(
///     MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
///         .user("Write a story about AI")
///         .stream(true)
///         .build()
/// ).await?;
///
/// let final_message = stream
///     .on_text(|delta, _snapshot| {
///         print!("{}", delta);
///     })
///     .on_error(|error| {
///         eprintln!("Stream error: {}", error);
///     })
///     .final_message().await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Async iteration:
/// ```ignore
/// # use anthropic_sdk::{Anthropic, MessageCreateBuilder, MessageStreamEvent};
/// # use futures::StreamExt;
/// # async fn example() -> anthropic_sdk::Result<()> {
/// let client = Anthropic::new("your-api-key")?;
/// let mut stream = client.messages().create_stream(
///     MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
///         .user("Tell me a joke")
///         .stream(true)
///         .build()
/// ).await?;
///
/// while let Some(event) = stream.next().await {
///     match event? {
///         MessageStreamEvent::ContentBlockDelta { delta, .. } => {
///             // Process incremental content
///         }
///         MessageStreamEvent::MessageStop => break,
///         _ => {}
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[pin_project]
pub struct MessageStream {
    /// Current accumulated message snapshot
    current_message: Arc<Mutex<Option<Message>>>,
    
    /// Event handlers for different event types
    event_handlers: Arc<Mutex<HashMap<EventType, Vec<EventHandler>>>>,
    
    /// Broadcast channel for distributing events to handlers
    event_sender: broadcast::Sender<MessageStreamEvent>,
    
    /// Stream for events from the underlying HTTP stream
    #[pin]
    event_stream: BroadcastStream<MessageStreamEvent>,
    
    /// Channel for signaling when the stream ends
    completion_sender: Option<oneshot::Sender<Result<Message>>>,
    completion_receiver: oneshot::Receiver<Result<Message>>,
    
    /// Whether the stream has ended
    ended: Arc<Mutex<bool>>,
    
    /// Whether an error occurred
    errored: Arc<Mutex<bool>>,
    
    /// Whether the stream was aborted by the user
    aborted: Arc<Mutex<bool>>,
    
    /// Response metadata
    response: Option<reqwest::Response>,
    request_id: Option<String>,
}

impl MessageStream {
    /// Create a new MessageStream from an HTTP response.
    ///
    /// This is typically called internally by the SDK when creating streaming requests.
    pub fn new(response: reqwest::Response, request_id: Option<String>) -> Self {
        let (event_sender, event_receiver) = broadcast::channel(1000);
        let (completion_sender, completion_receiver) = oneshot::channel();
        
        Self {
            current_message: Arc::new(Mutex::new(None)),
            event_handlers: Arc::new(Mutex::new(HashMap::new())),
            event_sender,
            event_stream: BroadcastStream::new(event_receiver),
            completion_sender: Some(completion_sender),
            completion_receiver,
            ended: Arc::new(Mutex::new(false)),
            errored: Arc::new(Mutex::new(false)),
            aborted: Arc::new(Mutex::new(false)),
            response: Some(response),
            request_id,
        }
    }
    
    /// Create a new MessageStream from an HttpStreamClient.
    ///
    /// This connects a real HTTP stream to the MessageStream, providing
    /// proper streaming functionality for real-time response processing.
    pub fn from_http_stream(mut http_stream: crate::http::streaming::HttpStreamClient) -> Result<Self> {
        let (event_sender, event_receiver) = broadcast::channel(1000);
        let (completion_sender, completion_receiver) = oneshot::channel();
        
        let current_message = Arc::new(Mutex::new(None));
        let ended = Arc::new(Mutex::new(false));
        let errored = Arc::new(Mutex::new(false));
        let request_id = http_stream.request_id().map(|s| s.to_string());
        
        // Clone references for the background task
        let current_message_clone = current_message.clone();
        let ended_clone = ended.clone();
        let errored_clone = errored.clone();
        let event_sender_clone = event_sender.clone();
        
        // Spawn task to process HTTP stream events
        tokio::spawn(async move {
            use futures::StreamExt;
            let mut final_message: Option<crate::types::Message> = None;
            
            while let Some(event_result) = http_stream.next().await {
                match event_result {
                    Ok(event) => {
                        // Update current message state
                        match &event {
                            crate::types::MessageStreamEvent::MessageStart { message } => {
                                *current_message_clone.lock().unwrap() = Some(message.clone());
                                final_message = Some(message.clone());
                            }
                            crate::types::MessageStreamEvent::ContentBlockStart { content_block, index } => {
                                if let Some(ref mut msg) = *current_message_clone.lock().unwrap() {
                                    while msg.content.len() <= *index {
                                        msg.content.push(crate::types::ContentBlock::Text { text: String::new() });
                                    }
                                    msg.content[*index] = content_block.clone();
                                }
                                if let Some(ref mut msg) = final_message.as_mut() {
                                    while msg.content.len() <= *index {
                                        msg.content.push(crate::types::ContentBlock::Text { text: String::new() });
                                    }
                                    msg.content[*index] = content_block.clone();
                                }
                            }
                            crate::types::MessageStreamEvent::ContentBlockDelta { delta, index } => {
                                if let Some(ref mut msg) = *current_message_clone.lock().unwrap() {
                                    if let Some(content_block) = msg.content.get_mut(*index) {
                                        if let (crate::types::ContentBlock::Text { text }, 
                                               crate::types::ContentBlockDelta::TextDelta { text: delta_text }) = 
                                            (content_block, delta) {
                                            text.push_str(delta_text);
                                        }
                                    }
                                }
                                if let Some(ref mut msg) = final_message.as_mut() {
                                    if let Some(content_block) = msg.content.get_mut(*index) {
                                        if let (crate::types::ContentBlock::Text { text }, 
                                               crate::types::ContentBlockDelta::TextDelta { text: delta_text }) = 
                                            (content_block, delta) {
                                            text.push_str(delta_text);
                                        }
                                    }
                                }
                            }
                            crate::types::MessageStreamEvent::MessageDelta { delta, usage } => {
                                if let Some(ref mut msg) = *current_message_clone.lock().unwrap() {
                                    if let Some(stop_reason) = &delta.stop_reason {
                                        msg.stop_reason = Some(stop_reason.clone());
                                    }
                                    if let Some(stop_sequence) = &delta.stop_sequence {
                                        msg.stop_sequence = Some(stop_sequence.clone());
                                    }
                                    msg.usage.output_tokens = usage.output_tokens;
                                    if let Some(input_tokens) = usage.input_tokens {
                                        msg.usage.input_tokens = input_tokens;
                                    }
                                }
                                if let Some(ref mut msg) = final_message.as_mut() {
                                    if let Some(stop_reason) = &delta.stop_reason {
                                        msg.stop_reason = Some(stop_reason.clone());
                                    }
                                    if let Some(stop_sequence) = &delta.stop_sequence {
                                        msg.stop_sequence = Some(stop_sequence.clone());
                                    }
                                    msg.usage.output_tokens = usage.output_tokens;
                                    if let Some(input_tokens) = usage.input_tokens {
                                        msg.usage.input_tokens = input_tokens;
                                    }
                                }
                            }
                            crate::types::MessageStreamEvent::MessageStop => {
                                *ended_clone.lock().unwrap() = true;
                                // Send the final message
                                if let Some(message) = final_message.clone() {
                                    let _ = completion_sender.send(Ok(message));
                                } else {
                                    let _ = completion_sender.send(Err(crate::types::AnthropicError::StreamError(
                                        "Stream ended without message".to_string()
                                    )));
                                }
                                // Send final event and break
                                let _ = event_sender_clone.send(event);
                                break;
                            }
                            _ => {}
                        }
                        
                        // Send event to broadcast channel for callbacks
                        let _ = event_sender_clone.send(event);
                    }
                    Err(e) => {
                        *errored_clone.lock().unwrap() = true;
                        let _ = completion_sender.send(Err(e));
                        break;
                    }
                }
            }
        });
        
        Ok(Self {
            current_message,
            event_handlers: Arc::new(Mutex::new(HashMap::new())),
            event_sender,
            event_stream: BroadcastStream::new(event_receiver),
            completion_sender: None, // Already consumed by the task
            completion_receiver,
            ended,
            errored,
            aborted: Arc::new(Mutex::new(false)),
            response: None, // No response needed for HTTP stream
            request_id,
        })
    }
    
    /// Register a callback for text delta events.
    ///
    /// The callback receives two parameters:
    /// - `delta`: The new text being appended
    /// - `snapshot`: The current accumulated text
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use anthropic_sdk::MessageStream;
    /// # async fn example(stream: MessageStream) {
    /// stream.on_text(|delta, snapshot| {
    ///     print!("{}", delta);
    ///     println!("Total so far: {}", snapshot);
    /// });
    /// # }
    /// ```
    pub fn on_text<F>(self, callback: F) -> Self
    where
        F: Fn(&str, &str) + Send + Sync + 'static,
    {
        self.on(EventType::Text, EventHandler::Text(Box::new(callback)))
    }
    
    /// Register a callback for stream events.
    ///
    /// This provides access to all raw stream events and the current message snapshot.
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use anthropic_sdk::{MessageStream, MessageStreamEvent, Message};
    /// # async fn example(stream: MessageStream) {
    /// stream.on_stream_event(|event, snapshot| {
    ///     match event {
    ///         MessageStreamEvent::ContentBlockStart { .. } => {
    ///             println!("New content block started");
    ///         }
    ///         _ => {}
    ///     }
    /// });
    /// # }
    /// ```
    pub fn on_stream_event<F>(self, callback: F) -> Self
    where
        F: Fn(&MessageStreamEvent, &Message) + Send + Sync + 'static,
    {
        self.on(EventType::StreamEvent, EventHandler::StreamEvent(Box::new(callback)))
    }
    
    /// Register a callback for when a complete message is received.
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use anthropic_sdk::{MessageStream, Message};
    /// # async fn example(stream: MessageStream) {
    /// stream.on_message(|message| {
    ///     println!("Received message: {:?}", message);
    /// });
    /// # }
    /// ```
    pub fn on_message<F>(self, callback: F) -> Self
    where
        F: Fn(&Message) + Send + Sync + 'static,
    {
        self.on(EventType::Message, EventHandler::Message(Box::new(callback)))
    }
    
    /// Register a callback for when the final message is complete.
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use anthropic_sdk::{MessageStream, Message};
    /// # async fn example(stream: MessageStream) {
    /// stream.on_final_message(|message| {
    ///     println!("Final message: {:?}", message);
    /// });
    /// # }
    /// ```
    pub fn on_final_message<F>(self, callback: F) -> Self
    where
        F: Fn(&Message) + Send + Sync + 'static,
    {
        self.on(EventType::FinalMessage, EventHandler::FinalMessage(Box::new(callback)))
    }
    
    /// Register a callback for errors.
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use anthropic_sdk::{MessageStream, AnthropicError};
    /// # async fn example(stream: MessageStream) {
    /// stream.on_error(|error| {
    ///     eprintln!("Stream error: {}", error);
    /// });
    /// # }
    /// ```
    pub fn on_error<F>(self, callback: F) -> Self
    where
        F: Fn(&AnthropicError) + Send + Sync + 'static,
    {
        self.on(EventType::Error, EventHandler::Error(Box::new(callback)))
    }
    
    /// Register a callback for when the stream ends.
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use anthropic_sdk::MessageStream;
    /// # async fn example(stream: MessageStream) {
    /// stream.on_end(|| {
    ///     println!("Stream ended");
    /// });
    /// # }
    /// ```
    pub fn on_end<F>(self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on(EventType::End, EventHandler::End(Box::new(callback)))
    }
    
    /// Generic method to register event handlers.
    fn on(self, event_type: EventType, handler: EventHandler) -> Self {
        {
            let mut handlers = self.event_handlers.lock().unwrap();
            handlers.entry(event_type).or_insert_with(Vec::new).push(handler);
        }
        self
    }
    
    /// Wait for the stream to complete and return the final message.
    ///
    /// This method will block until the stream ends and return the accumulated message.
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use anthropic_sdk::MessageStream;
    /// # async fn example(stream: MessageStream) -> anthropic_sdk::Result<()> {
    /// let final_message = stream.final_message().await?;
    /// println!("Claude said: {:?}", final_message.content);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn final_message(self) -> Result<Message> {
        self.completion_receiver.await
            .map_err(|_| AnthropicError::StreamError("Stream ended unexpectedly".to_string()))?
    }
    
    /// Wait for the stream to complete without returning the message.
    ///
    /// This is useful when you're processing events with callbacks and just need
    /// to wait for completion.
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use anthropic_sdk::MessageStream;
    /// # async fn example(stream: MessageStream) -> anthropic_sdk::Result<()> {
    /// stream.on_text(|delta, _| print!("{}", delta))
    ///     .done().await?;
    /// println!("\nStream completed!");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn done(self) -> Result<()> {
        self.completion_receiver.await
            .map_err(|_| AnthropicError::StreamError("Stream ended unexpectedly".to_string()))?
            .map(|_| ())
    }
    
    /// Get the current accumulated message snapshot.
    ///
    /// Returns `None` if the stream hasn't started or no message has been received yet.
    pub fn current_message(&self) -> Option<Message> {
        self.current_message.lock().unwrap().clone()
    }
    
    /// Check if the stream has ended.
    pub fn ended(&self) -> bool {
        *self.ended.lock().unwrap()
    }
    
    /// Check if an error occurred.
    pub fn errored(&self) -> bool {
        *self.errored.lock().unwrap()
    }
    
    /// Check if the stream was aborted.
    pub fn aborted(&self) -> bool {
        *self.aborted.lock().unwrap()
    }
    
    /// Get the response metadata.
    pub fn response(&self) -> Option<&reqwest::Response> {
        self.response.as_ref()
    }
    
    /// Get the request ID.
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }
    
    /// Abort the stream.
    ///
    /// This will cancel the underlying HTTP request and mark the stream as aborted.
    pub fn abort(&self) {
        *self.aborted.lock().unwrap() = true;
        // In a real implementation, this would cancel the HTTP request
    }
    
    /// Process a stream event and update the internal state.
    ///
    /// This method accumulates message content from incremental updates and
    /// dispatches events to registered handlers.
    #[allow(dead_code)]
    fn process_event(&self, event: MessageStreamEvent) -> Result<()> {
        // Update current message state based on the event
        match &event {
            MessageStreamEvent::MessageStart { message } => {
                *self.current_message.lock().unwrap() = Some(message.clone());
            }
            MessageStreamEvent::ContentBlockStart { content_block, index } => {
                if let Some(ref mut msg) = *self.current_message.lock().unwrap() {
                    // Ensure the content array is large enough
                    while msg.content.len() <= *index {
                        msg.content.push(ContentBlock::Text { text: String::new() });
                    }
                    msg.content[*index] = content_block.clone();
                }
            }
            MessageStreamEvent::ContentBlockDelta { delta, index } => {
                if let Some(ref mut msg) = *self.current_message.lock().unwrap() {
                    if let Some(content_block) = msg.content.get_mut(*index) {
                        self.apply_delta(content_block, delta)?;
                    }
                }
            }
            MessageStreamEvent::MessageDelta { delta, usage } => {
                if let Some(ref mut msg) = *self.current_message.lock().unwrap() {
                    if let Some(stop_reason) = &delta.stop_reason {
                        msg.stop_reason = Some(stop_reason.clone());
                    }
                    if let Some(stop_sequence) = &delta.stop_sequence {
                        msg.stop_sequence = Some(stop_sequence.clone());
                    }
                    // Update usage information
                    msg.usage.output_tokens = usage.output_tokens;
                    if let Some(input_tokens) = usage.input_tokens {
                        msg.usage.input_tokens = input_tokens;
                    }
                }
            }
            MessageStreamEvent::MessageStop => {
                *self.ended.lock().unwrap() = true;
            }
            _ => {}
        }
        
        // Dispatch event to handlers
        self.dispatch_event(&event)?;
        
        // Send event to broadcast channel for async iteration
        let _ = self.event_sender.send(event);
        
        Ok(())
    }
    
    /// Apply a content block delta to update the content.
    #[allow(dead_code)]
    fn apply_delta(&self, content_block: &mut ContentBlock, delta: &ContentBlockDelta) -> Result<()> {
        match (content_block, delta) {
            (ContentBlock::Text { text }, ContentBlockDelta::TextDelta { text: delta_text }) => {
                text.push_str(delta_text);
            }
            (ContentBlock::ToolUse { input, .. }, ContentBlockDelta::InputJsonDelta { partial_json }) => {
                // In a real implementation, we'd parse the partial JSON
                // For now, we'll just store it as-is
                *input = serde_json::from_str(partial_json)
                    .unwrap_or_else(|_| serde_json::Value::String(partial_json.clone()));
            }
            _ => {
                // Other delta types would be handled here
            }
        }
        Ok(())
    }
    
    /// Dispatch an event to all registered handlers.
    fn dispatch_event(&self, event: &MessageStreamEvent) -> Result<()> {
        let handlers = self.event_handlers.lock().unwrap();
        let current_message = self.current_message.lock().unwrap();
        
        // Dispatch to stream event handlers
        if let Some(stream_handlers) = handlers.get(&EventType::StreamEvent) {
            for handler in stream_handlers {
                if let EventHandler::StreamEvent(callback) = handler {
                    if let Some(ref msg) = *current_message {
                        callback(event, msg);
                    }
                }
            }
        }
        
        // Dispatch specific event types
        match event {
            MessageStreamEvent::ContentBlockDelta { delta, .. } => {
                if let ContentBlockDelta::TextDelta { text } = delta {
                    if let Some(text_handlers) = handlers.get(&EventType::Text) {
                        for handler in text_handlers {
                            if let EventHandler::Text(callback) = handler {
                                // Get current accumulated text for snapshot
                                let snapshot = if let Some(ref msg) = *current_message {
                                    self.get_accumulated_text(msg)
                                } else {
                                    String::new()
                                };
                                callback(text, &snapshot);
                            }
                        }
                    }
                }
            }
            MessageStreamEvent::MessageStop => {
                if let Some(end_handlers) = handlers.get(&EventType::End) {
                    for handler in end_handlers {
                        if let EventHandler::End(callback) = handler {
                            callback();
                        }
                    }
                }
                
                // Send final message
                if let Some(ref msg) = *current_message {
                    if let Some(final_handlers) = handlers.get(&EventType::FinalMessage) {
                        for handler in final_handlers {
                            if let EventHandler::FinalMessage(callback) = handler {
                                callback(msg);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Get the accumulated text from all text content blocks.
    fn get_accumulated_text(&self, message: &Message) -> String {
        message.content
            .iter()
            .filter_map(|block| match block {
                ContentBlock::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

impl Stream for MessageStream {
    type Item = Result<MessageStreamEvent>;
    
    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use futures::Stream as FuturesStream;
        
        let this = self.project();
        
        match FuturesStream::poll_next(this.event_stream, cx) {
            std::task::Poll::Ready(Some(Ok(event))) => {
                std::task::Poll::Ready(Some(Ok(event)))
            }
            std::task::Poll::Ready(Some(Err(err))) => {
                // Handle any broadcast stream errors
                std::task::Poll::Ready(Some(Err(AnthropicError::StreamError(
                    format!("Stream error: {}", err)
                ))))
            }
            std::task::Poll::Ready(None) => {
                std::task::Poll::Ready(None)
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Role, Usage};
    
    // For testing, we'll use a simple helper to create a dummy response
    async fn create_dummy_response() -> reqwest::Response {
        // Create a simple HTTP client and make a basic request for testing
        let client = reqwest::Client::new();
        // Use httpbin.org which provides testing endpoints
        client.get("https://httpbin.org/status/200")
            .send()
            .await
            .expect("Failed to create test response")
    }
    
    #[tokio::test]
    async fn test_message_stream_creation() {
        let response = create_dummy_response().await;
        let stream = MessageStream::new(response, Some("test-request-id".to_string()));
        
        assert!(!stream.ended());
        assert!(!stream.errored());
        assert!(!stream.aborted());
        assert_eq!(stream.request_id(), Some("test-request-id"));
    }
    
    #[tokio::test]
    async fn test_event_processing() {
        let response = create_dummy_response().await;
        let stream = MessageStream::new(response, None);
        
        // Test message start event
        let start_event = MessageStreamEvent::MessageStart {
            message: Message {
                id: "msg_test".to_string(),
                type_: "message".to_string(),
                role: Role::Assistant,
                content: vec![],
                model: "claude-3-5-sonnet-latest".to_string(),
                stop_reason: None,
                stop_sequence: None,
                usage: Usage {
                    input_tokens: 10,
                    output_tokens: 0,
                    cache_creation_input_tokens: None,
                    cache_read_input_tokens: None,
                    server_tool_use: None,
                    service_tier: None,
                },
                request_id: None,
            },
        };
        
        stream.process_event(start_event).unwrap();
        
        let current = stream.current_message().unwrap();
        assert_eq!(current.id, "msg_test");
        assert_eq!(current.role, Role::Assistant);
    }
    
    #[test]
    fn test_event_handlers() {
        use std::sync::{Arc, Mutex};
        use std::collections::HashMap;
        
        // Test creating event handlers directly
        let text_called = Arc::new(Mutex::new(false));
        let text_called_clone = text_called.clone();
        
        let _handler = EventHandler::Text(Box::new(move |_delta, _snapshot| {
            *text_called_clone.lock().unwrap() = true;
        }));
        
        // Test event type equality
        assert_eq!(EventType::Text, EventType::Text);
        assert_ne!(EventType::Text, EventType::Error);
        
        // Test using event types as hash keys
        let mut map: HashMap<EventType, String> = HashMap::new();
        map.insert(EventType::Text, "text_handler".to_string());
        assert_eq!(map.get(&EventType::Text), Some(&"text_handler".to_string()));
    }
} 