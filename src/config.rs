use std::time::Duration;
use dotenvy::dotenv;
use crate::types::errors::{AnthropicError, Result};
use crate::http::auth::AuthMethod;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub api_key: String,
    pub base_url: String,
    pub timeout: Duration,
    pub max_retries: u32,
    pub log_level: LogLevel,
    pub auth_method: AuthMethod,
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Off,
}

impl ClientConfig {
    /// Create a new client configuration with the provided API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.anthropic.com".to_string(),
            timeout: Duration::from_secs(600), // 10 minutes default
            max_retries: 2,
            log_level: LogLevel::Warn,
            auth_method: AuthMethod::Anthropic,
        }
    }
    
    /// Create a client configuration from environment variables
    pub fn from_env() -> Result<Self> {
        dotenv().ok(); // Load .env file if present
        
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| AnthropicError::Configuration {
                message: "ANTHROPIC_API_KEY environment variable not set".to_string(),
            })?;
            
        let mut config = Self::new(api_key);
        
        // Optional environment variable overrides
        if let Ok(base_url) = std::env::var("ANTHROPIC_BASE_URL") {
            config.base_url = base_url;
        }
        
        if let Ok(timeout_str) = std::env::var("ANTHROPIC_TIMEOUT") {
            if let Ok(timeout_secs) = timeout_str.parse::<u64>() {
                config.timeout = Duration::from_secs(timeout_secs);
            }
        }
        
        if let Ok(retries_str) = std::env::var("ANTHROPIC_MAX_RETRIES") {
            if let Ok(retries) = retries_str.parse::<u32>() {
                config.max_retries = retries;
            }
        }
        
        // Check for authentication method override
        if let Ok(auth_method) = std::env::var("ANTHROPIC_AUTH_METHOD") {
            match auth_method.to_lowercase().as_str() {
                "bearer" => config.auth_method = AuthMethod::Bearer,
                "token" => config.auth_method = AuthMethod::Token,
                "anthropic" => config.auth_method = AuthMethod::Anthropic,
                _ => {} // Keep default
            }
        }
        
        Ok(config)
    }
    
    /// Set the request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
    
    /// Set the maximum number of retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
    
    /// Set the log level
    pub fn with_log_level(mut self, log_level: LogLevel) -> Self {
        self.log_level = log_level;
        self
    }
    
    /// Set the base URL
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
    
    /// Set the authentication method
    pub fn with_auth_method(mut self, auth_method: AuthMethod) -> Self {
        self.auth_method = auth_method;
        self
    }
    
    /// Configure for custom gateway (Bearer token + base URL)
    pub fn for_custom_gateway(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self.auth_method = AuthMethod::Bearer;
        self
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.api_key.is_empty() {
            return Err(AnthropicError::Configuration {
                message: "API key cannot be empty".to_string(),
            });
        }
        
        if self.base_url.is_empty() {
            return Err(AnthropicError::Configuration {
                message: "Base URL cannot be empty".to_string(),
            });
        }
        
        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            return Err(AnthropicError::Configuration {
                message: "Base URL must start with http:// or https://".to_string(),
            });
        }
        
        Ok(())
    }
} 