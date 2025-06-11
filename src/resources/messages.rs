use crate::client::Anthropic;
use crate::types::messages::*;
use crate::types::errors::{AnthropicError, Result};
use crate::streaming::MessageStream;
use crate::http::streaming::{StreamRequestBuilder, StreamConfig};

/// Messages API resource for interacting with Claude
pub struct MessagesResource<'a> {
    client: &'a Anthropic,
}

impl<'a> MessagesResource<'a> {
    /// Create a new Messages resource
    pub fn new(client: &'a Anthropic) -> Self {
        Self { client }
    }
    
    /// Create a message with Claude
    /// 
    /// Send a structured list of input messages with text and/or image content,
    /// and Claude will generate the next message in the conversation.
    /// 
    /// # Example
    /// 
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use anthropic_sdk::{Anthropic, types::MessageCreateBuilder};
    /// 
    /// let client = Anthropic::from_env()?;
    /// 
    /// let message = client.messages().create(
    ///     MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
    ///         .user("Hello, Claude!")
    ///         .build()
    /// ).await?;
    /// 
    /// println!("Claude responded: {:?}", message.content);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(&self, params: MessageCreateParams) -> Result<Message> {
        let url = self.client.http_client().build_url("/v1/messages");
        
        let request = self.client.http_client()
            .post(&url)
            .json(&params)
            .build()
            .map_err(|e| AnthropicError::Connection { message: e.to_string() })?;
        
        let response = self.client.http_client().send(request).await?;
        
        // Extract request ID from headers
        let request_id = self.client.http_client().extract_request_id(&response);
        
        let mut message: Message = response.json().await
            .map_err(|e| AnthropicError::Connection { message: e.to_string() })?;
            
        message.request_id = request_id;
        
        Ok(message)
    }
    
    /// Create a streaming message with Claude
    /// 
    /// Send a message request and receive a real-time stream of the response.
    /// This allows you to process Claude's response as it's being generated.
    /// 
    /// # Example
    /// 
    /// ```ignore
    /// use anthropic_sdk::{Anthropic, MessageCreateBuilder};
    /// use futures::StreamExt;
    /// 
    /// let client = Anthropic::from_env()?;
    /// 
    /// let stream = client.messages().create_stream(
    ///     MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
    ///         .user("Write a story about AI")
    ///         .stream(true)
    ///         .build()
    /// ).await?;
    /// 
    /// // Option 1: Use callbacks
    /// let final_message = stream
    ///     .on_text(|delta, _| print!("{}", delta))
    ///     .on_error(|error| eprintln!("Error: {}", error))
    ///     .final_message().await?;
    /// 
    /// // Option 2: Manual iteration
    /// while let Some(event) = stream.next().await {
    ///     // Process each event as needed
    /// }
    /// ```
    pub async fn create_stream(&self, mut params: MessageCreateParams) -> Result<MessageStream> {
        // Ensure streaming is enabled
        params.stream = Some(true);
        
        // Create authorization header - use Bearer for most cases including custom gateways
        let auth_header = format!("Bearer {}", self.client.config().api_key);
        
        // Build the streaming request with proper authentication
        let stream_builder = StreamRequestBuilder::new(
            self.client.http_client().client().clone(),
            self.client.config().base_url.clone(),
        )
        .header("Authorization", &auth_header)
        .header("Content-Type", "application/json")
        .header("anthropic-version", "2023-06-01")
        .config(StreamConfig::default());
        
        // Make the streaming request to get the real HTTP stream
        let http_stream = stream_builder
            .post_stream("v1/messages", &params)
            .await?;
        
        // Create MessageStream that processes the real HTTP stream events
        let message_stream = MessageStream::from_http_stream(http_stream)?;
        
        Ok(message_stream)
    }
    
    /// Create a streaming message using the builder pattern
    /// 
    /// This is a convenience method that provides an ergonomic API for creating streaming messages.
    /// 
    /// # Example
    /// 
    /// ```ignore
    /// use anthropic_sdk::Anthropic;
    /// 
    /// let client = Anthropic::from_env()?;
    /// 
    /// let final_message = client.messages()
    ///     .create_with_builder("claude-3-5-sonnet-latest", 1024)
    ///     .user("Write a poem about the ocean")
    ///     .system("You are a creative poet.")
    ///     .temperature(0.8)
    ///     .stream()
    ///     .await?
    ///     .on_text(|delta, _| print!("{}", delta))
    ///     .final_message()
    ///     .await?;
    /// ```
    pub async fn stream(&self, params: MessageCreateParams) -> Result<MessageStream> {
        self.create_stream(params).await
    }
    
    /// Create a message using the builder pattern
    /// 
    /// This is a convenience method that provides an ergonomic API for creating messages.
    /// 
    /// # Example
    /// 
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use anthropic_sdk::Anthropic;
    /// 
    /// let client = Anthropic::from_env()?;
    /// 
    /// let message = client.messages()
    ///     .create_with_builder("claude-3-5-sonnet-latest", 1024)
    ///     .user("What is the capital of France?")
    ///     .system("You are a helpful geography assistant.")
    ///     .temperature(0.3)
    ///     .send()
    ///     .await?;
    /// 
    /// println!("Response: {:?}", message.content);
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_with_builder(&'a self, model: impl Into<String>, max_tokens: u32) -> MessageCreateBuilderWithClient<'a> {
        MessageCreateBuilderWithClient {
            resource: self,
            builder: MessageCreateBuilder::new(model, max_tokens),
        }
    }
}

/// A message builder with a client reference for sending requests
pub struct MessageCreateBuilderWithClient<'a> {
    resource: &'a MessagesResource<'a>,
    builder: MessageCreateBuilder,
}

impl<'a> MessageCreateBuilderWithClient<'a> {
    /// Add a message to the conversation
    pub fn message(mut self, role: Role, content: impl Into<MessageContent>) -> Self {
        self.builder = self.builder.message(role, content);
        self
    }
    
    /// Add a user message
    pub fn user(mut self, content: impl Into<MessageContent>) -> Self {
        self.builder = self.builder.user(content);
        self
    }
    
    /// Add an assistant message
    pub fn assistant(mut self, content: impl Into<MessageContent>) -> Self {
        self.builder = self.builder.assistant(content);
        self
    }
    
    /// Set the system prompt
    pub fn system(mut self, system: impl Into<String>) -> Self {
        self.builder = self.builder.system(system);
        self
    }
    
    /// Set the temperature
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.builder = self.builder.temperature(temperature);
        self
    }
    
    /// Set top_p
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.builder = self.builder.top_p(top_p);
        self
    }
    
    /// Set top_k
    pub fn top_k(mut self, top_k: u32) -> Self {
        self.builder = self.builder.top_k(top_k);
        self
    }
    
    /// Set custom stop sequences
    pub fn stop_sequences(mut self, stop_sequences: Vec<String>) -> Self {
        self.builder = self.builder.stop_sequences(stop_sequences);
        self
    }
    
    /// Enable streaming
    pub fn stream(mut self, stream: bool) -> Self {
        self.builder = self.builder.stream(stream);
        self
    }
    
    /// Send the message request
    pub async fn send(self) -> Result<Message> {
        self.resource.create(self.builder.build()).await
    }
    
    /// Send the message request as a stream
    /// 
    /// This enables streaming mode and returns a MessageStream for real-time processing.
    /// 
    /// # Example
    /// 
    /// ```ignore
    /// let stream = client.messages()
    ///     .create_with_builder("claude-3-5-sonnet-latest", 1024)
    ///     .user("Tell me a story")
    ///     .stream_send()
    ///     .await?;
    /// 
    /// let final_message = stream
    ///     .on_text(|delta, _| print!("{}", delta))
    ///     .final_message()
    ///     .await?;
    /// ```
    pub async fn stream_send(self) -> Result<MessageStream> {
        let params = self.builder.stream(true).build();
        self.resource.create_stream(params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::messages::{MessageContent, ContentBlockParam};

    #[test]
    fn test_message_create_params_serialization() {
        let params = MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
            .user("Hello, world!")
            .system("You are helpful")
            .temperature(0.7)
            .build();

        let json = serde_json::to_value(&params).unwrap();
        
        assert_eq!(json["model"], "claude-3-5-sonnet-latest");
        assert_eq!(json["max_tokens"], 1024);
        assert_eq!(json["messages"].as_array().unwrap().len(), 1);
        assert_eq!(json["system"], "You are helpful");
        
        // Handle floating point precision by checking if the value is close to 0.7
        let temperature = json["temperature"].as_f64().unwrap();
        assert!((temperature - 0.7).abs() < 0.001, "Temperature should be close to 0.7, got {}", temperature);
    }

    #[test]
    fn test_complex_message_content() {
        let content = MessageContent::Blocks(vec![
            ContentBlockParam::text("Here's an image:"),
            ContentBlockParam::image_base64("image/jpeg", "base64data"),
        ]);
        
        let params = MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
            .user(content)
            .build();

        let json = serde_json::to_value(&params).unwrap();
        let message_content = &json["messages"][0]["content"];
        
        assert!(message_content.is_array());
        assert_eq!(message_content.as_array().unwrap().len(), 2);
        assert_eq!(message_content[0]["type"], "text");
        assert_eq!(message_content[1]["type"], "image");
    }

    #[test]
    fn test_multi_message_conversation() {
        let params = MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
            .user("Hello!")
            .assistant("Hi there! How can I help you?")
            .user("What's the weather like?")
            .build();

        assert_eq!(params.messages.len(), 3);
        assert_eq!(params.messages[0].role, Role::User);
        assert_eq!(params.messages[1].role, Role::Assistant);
        assert_eq!(params.messages[2].role, Role::User);
    }
} 