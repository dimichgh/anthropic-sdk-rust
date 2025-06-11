use reqwest::header::{HeaderValue, HeaderMap};
use crate::types::errors::{AnthropicError, Result};

/// Authentication method for different API gateways
#[derive(Debug, Clone)]
pub enum AuthMethod {
    /// Standard Anthropic API authentication with x-api-key header
    Anthropic,
    /// Bearer token authentication for custom gateways and third-party services
    Bearer,
    /// Custom token header (for compatibility with various gateways)
    Token,
}

/// Authentication handler for Anthropic API and compatible gateways
#[derive(Debug, Clone)]
pub struct AuthHandler {
    api_key: String,
    auth_method: AuthMethod,
}

impl AuthHandler {
    /// Create a new auth handler with standard Anthropic authentication
    pub fn new(api_key: String) -> Self {
        Self { 
            api_key,
            auth_method: AuthMethod::Anthropic,
        }
    }
    
    /// Create a new auth handler with Bearer token authentication
    pub fn new_bearer(api_key: String) -> Self {
        Self {
            api_key,
            auth_method: AuthMethod::Bearer,
        }
    }
    
    /// Create a new auth handler with custom token header
    pub fn new_token(api_key: String) -> Self {
        Self {
            api_key,
            auth_method: AuthMethod::Token,
        }
    }
    
    /// Create a new auth handler with specified method
    pub fn with_method(api_key: String, auth_method: AuthMethod) -> Self {
        Self {
            api_key,
            auth_method,
        }
    }
    
    /// Add authentication headers to the request
    pub fn add_auth_headers(&self, headers: &mut HeaderMap) -> Result<()> {
        match self.auth_method {
            AuthMethod::Anthropic => {
                let api_key_header = HeaderValue::from_str(&self.api_key)
                    .map_err(|_| AnthropicError::Configuration {
                        message: "Invalid API key format".to_string(),
                    })?;
                    
                headers.insert("x-api-key", api_key_header);
                headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
            }
            AuthMethod::Bearer => {
                let bearer_token = format!("Bearer {}", self.api_key);
                let auth_header = HeaderValue::from_str(&bearer_token)
                    .map_err(|_| AnthropicError::Configuration {
                        message: "Invalid API key format for Bearer token".to_string(),
                    })?;
                    
                headers.insert("authorization", auth_header);
                headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
            }
            AuthMethod::Token => {
                let token_header = HeaderValue::from_str(&self.api_key)
                    .map_err(|_| AnthropicError::Configuration {
                        message: "Invalid API key format for token header".to_string(),
                    })?;
                    
                headers.insert("token", token_header);
                headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
            }
        }
        
        headers.insert("content-type", HeaderValue::from_static("application/json"));
        
        Ok(())
    }
} 