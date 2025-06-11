use serde::{Deserialize, Serialize};

/// Request ID for tracking API requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Usage information for API requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Usage {
    /// The number of input tokens which were used
    pub input_tokens: u32,
    
    /// The number of output tokens which were used
    pub output_tokens: u32,
    
    /// The number of input tokens used to create the cache entry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,
    
    /// The number of input tokens read from the cache
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,
    
    /// Server tool usage statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_tool_use: Option<ServerToolUsage>,
    
    /// Service tier used for the request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
}

/// Server tool usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServerToolUsage {
    /// Number of web search tool requests made
    pub web_search_requests: u32,
}

impl Usage {
    /// Get the total number of tokens used
    pub fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
    
    /// Get the total input tokens including cache tokens
    pub fn total_input_tokens(&self) -> u32 {
        self.input_tokens 
            + self.cache_creation_input_tokens.unwrap_or(0)
            + self.cache_read_input_tokens.unwrap_or(0)
    }
}

/// Base trait for responses that include request IDs
pub trait HasRequestId {
    fn request_id(&self) -> Option<&RequestId>;
} 