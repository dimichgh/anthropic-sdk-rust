use crate::config::ClientConfig;
use crate::http::HttpClient;
use crate::types::errors::Result;
use crate::utils::init_logging;
use crate::resources::{MessagesResource, BatchesResource, FilesResource, ModelsResource};

/// Main Anthropic API client
pub struct Anthropic {
    config: ClientConfig,
    http_client: HttpClient,
}

impl Anthropic {
    /// Create a new Anthropic client with the provided API key
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        let config = ClientConfig::new(api_key);
        let http_client = HttpClient::new(config.clone())?;
        
        // Initialize logging
        init_logging(&config.log_level);
        
        Ok(Self {
            config,
            http_client,
        })
    }
    
    /// Create a new Anthropic client from environment variables
    pub fn from_env() -> Result<Self> {
        let config = ClientConfig::from_env()?;
        let http_client = HttpClient::new(config.clone())?;
        
        // Initialize logging
        init_logging(&config.log_level);
        
        Ok(Self {
            config,
            http_client,
        })
    }
    
    /// Create a new Anthropic client with custom configuration
    pub fn with_config(config: ClientConfig) -> Result<Self> {
        let http_client = HttpClient::new(config.clone())?;
        
        // Initialize logging
        init_logging(&config.log_level);
        
        Ok(Self {
            config,
            http_client,
        })
    }
    
    /// Get the current configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }
    
    /// Get a reference to the HTTP client for internal use
    pub(crate) fn http_client(&self) -> &HttpClient {
        &self.http_client
    }
    
    /// Test the connection by making a simple API call
    pub async fn test_connection(&self) -> Result<()> {
        // This will be implemented once we have the messages API
        // For now, we'll just validate the configuration
        self.config.validate()?;
        tracing::info!("Anthropic client initialized successfully");
        Ok(())
    }
    
    /// Access to the Messages API
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
    pub fn messages(&self) -> MessagesResource {
        MessagesResource::new(self)
    }

    /// Access to the Message Batches API (Beta)
    /// 
    /// # Example
    /// 
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use anthropic_sdk::{Anthropic, types::{BatchRequest, BatchCreateParams}};
    /// 
    /// let client = Anthropic::from_env()?;
    /// 
    /// let requests = vec![
    ///     BatchRequest::new("req1", "claude-3-5-sonnet-latest", 1024)
    ///         .user("Hello, world!")
    ///         .build(),
    /// ];
    /// 
    /// let batch = client.batches().create(
    ///     BatchCreateParams::new(requests)
    /// ).await?;
    /// 
    /// println!("Created batch: {}", batch.id);
    /// # Ok(())
    /// # }
    /// ```
    pub fn batches(&self) -> BatchesResource {
        BatchesResource::new(std::sync::Arc::new(self.http_client.clone()))
    }

    /// Access to the Files API (Beta)
    /// 
    /// # Example
    /// 
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use anthropic_sdk::{Anthropic, FileUploadParams, FilePurpose};
    /// 
    /// let client = Anthropic::from_env()?;
    /// 
    /// let upload_params = FileUploadParams::new(
    ///     std::fs::read("document.pdf")?,
    ///     "document.pdf",
    ///     "application/pdf",
    ///     FilePurpose::Document,
    /// );
    /// 
    /// let file_object = client.files().upload(upload_params).await?;
    /// println!("Uploaded file: {}", file_object.id);
    /// # Ok(())
    /// # }
    /// ```
    pub fn files(&self) -> FilesResource {
        FilesResource::new(std::sync::Arc::new(self.http_client.clone()))
    }

    /// Access to the Models API
    /// 
    /// # Example
    /// 
    /// ```rust,no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use anthropic_sdk::{Anthropic, ModelListParams, ModelRequirements, ModelCapability};
    /// 
    /// let client = Anthropic::from_env()?;
    /// 
    /// // List all models
    /// let models = client.models().list(None).await?;
    /// println!("Found {} models", models.data.len());
    /// 
    /// // Get specific model
    /// let model = client.models().get("claude-3-5-sonnet-latest").await?;
    /// println!("Model: {} ({})", model.display_name, model.id);
    /// 
    /// // Find best model for requirements
    /// let requirements = ModelRequirements::new()
    ///     .require_vision()
    ///     .min_context_length(100000);
    /// let best_model = client.models().find_best_model(&requirements).await?;
    /// 
    /// // Compare models
    /// let comparison = client.models().compare_models(&[
    ///     "claude-3-5-sonnet-latest",
    ///     "claude-3-5-haiku-latest"
    /// ]).await?;
    /// 
    /// // Estimate costs
    /// let cost = client.models().estimate_cost("claude-3-5-sonnet-latest", 1000, 500).await?;
    /// println!("Estimated cost: ${:.4}", cost.final_cost_usd);
    /// # Ok(())
    /// # }
    /// ```
    pub fn models(&self) -> ModelsResource {
        ModelsResource::new(self)
    }
}

impl std::fmt::Debug for Anthropic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Anthropic")
            .field("base_url", &self.config.base_url)
            .field("timeout", &self.config.timeout)
            .field("max_retries", &self.config.max_retries)
            .field("log_level", &self.config.log_level)
            .finish_non_exhaustive()
    }
} 