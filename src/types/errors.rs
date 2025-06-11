use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum AnthropicError {
    #[error("Bad request: {message}")]
    BadRequest { message: String, status: u16 },
    
    #[error("Authentication failed: {message}")]
    Authentication { message: String, status: u16 },
    
    #[error("Permission denied: {message}")]
    PermissionDenied { message: String, status: u16 },
    
    #[error("Resource not found: {message}")]
    NotFound { message: String, status: u16 },
    
    #[error("Unprocessable entity: {message}")]
    UnprocessableEntity { message: String, status: u16 },
    
    #[error("Rate limit exceeded: {message}")]
    RateLimit { message: String, status: u16 },
    
    #[error("Internal server error: {message}")]
    InternalServer { message: String, status: u16 },
    
    #[error("API connection error: {message}")]
    Connection { message: String },
    
    #[error("API connection timeout")]
    ConnectionTimeout,
    
    #[error("User aborted request")]
    UserAbort,
    
    #[error("Streaming error: {0}")]
    StreamError(String),
    
    #[error("Invalid configuration: {message}")]
    Configuration { message: String },
    
    #[error("Invalid API key")]
    InvalidApiKey,
    
    #[error("Request timeout")]
    Timeout,
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("HTTP error: {status} - {message}")]
    HttpError { status: u16, message: String },
    
    #[error("Service unavailable: {message}")]
    ServiceUnavailable { message: String },
    
    #[error("{0}")]
    Other(String),
}

impl AnthropicError {
    /// Create an error from HTTP response status and message
    pub fn from_status(status: u16, message: String) -> Self {
        match status {
            400 => Self::BadRequest { message, status },
            401 => Self::Authentication { message, status },
            403 => Self::PermissionDenied { message, status },
            404 => Self::NotFound { message, status },
            422 => Self::UnprocessableEntity { message, status },
            429 => Self::RateLimit { message, status },
            500..=599 => Self::InternalServer { message, status },
            _ => Self::InternalServer { message, status },
        }
    }
    
    /// Get the HTTP status code if available
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::BadRequest { status, .. } |
            Self::Authentication { status, .. } |
            Self::PermissionDenied { status, .. } |
            Self::NotFound { status, .. } |
            Self::UnprocessableEntity { status, .. } |
            Self::RateLimit { status, .. } |
            Self::InternalServer { status, .. } => Some(*status),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for AnthropicError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::Timeout
        } else if err.is_connect() {
            Self::Connection { 
                message: err.to_string() 
            }
        } else if err.is_request() {
            Self::HttpError { 
                status: err.status().map(|s| s.as_u16()).unwrap_or(0), 
                message: err.to_string() 
            }
        } else {
            Self::NetworkError(err.to_string())
        }
    }
}

impl From<serde_json::Error> for AnthropicError {
    fn from(err: serde_json::Error) -> Self {
        Self::Other(format!("JSON serialization/deserialization error: {}", err))
    }
}

impl From<chrono::OutOfRangeError> for AnthropicError {
    fn from(err: chrono::OutOfRangeError) -> Self {
        Self::Other(format!("Date/time out of range error: {}", err))
    }
}

pub type Result<T> = std::result::Result<T, AnthropicError>; 