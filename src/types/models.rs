use serde::{Deserialize, Serialize};

/// Supported Claude model variants
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Model {
    // Claude 3.5 Sonnet models
    #[serde(rename = "claude-3-5-sonnet-latest")]
    Claude3_5SonnetLatest,
    #[serde(rename = "claude-3-5-sonnet-20241022")]
    Claude3_5Sonnet20241022,
    #[serde(rename = "claude-3-5-sonnet-20240620")]
    Claude3_5Sonnet20240620,
    
    // Claude 3.5 Haiku models
    #[serde(rename = "claude-3-5-haiku-latest")]
    Claude3_5HaikuLatest,
    #[serde(rename = "claude-3-5-haiku-20241022")]
    Claude3_5Haiku20241022,
    
    // Claude 3 Opus models
    #[serde(rename = "claude-3-opus-latest")]
    Claude3OpusLatest,
    #[serde(rename = "claude-3-opus-20240229")]
    Claude3Opus20240229,
    
    // Claude 3 Sonnet models
    #[serde(rename = "claude-3-sonnet-20240229")]
    Claude3Sonnet20240229,
    
    // Claude 3 Haiku models
    #[serde(rename = "claude-3-haiku-20240307")]
    Claude3Haiku20240307,
    
    // Legacy Claude 2 models
    #[serde(rename = "claude-2.1")]
    Claude2_1,
    #[serde(rename = "claude-2.0")]
    Claude2_0,
}

impl Model {
    /// Get the string representation of the model
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Claude3_5SonnetLatest => "claude-3-5-sonnet-latest",
            Self::Claude3_5Sonnet20241022 => "claude-3-5-sonnet-20241022",
            Self::Claude3_5Sonnet20240620 => "claude-3-5-sonnet-20240620",
            Self::Claude3_5HaikuLatest => "claude-3-5-haiku-latest",
            Self::Claude3_5Haiku20241022 => "claude-3-5-haiku-20241022",
            Self::Claude3OpusLatest => "claude-3-opus-latest",
            Self::Claude3Opus20240229 => "claude-3-opus-20240229",
            Self::Claude3Sonnet20240229 => "claude-3-sonnet-20240229",
            Self::Claude3Haiku20240307 => "claude-3-haiku-20240307",
            Self::Claude2_1 => "claude-2.1",
            Self::Claude2_0 => "claude-2.0",
        }
    }
    
    /// Get the family of the model (e.g., "sonnet", "haiku", "opus")
    pub fn family(&self) -> &'static str {
        match self {
            Self::Claude3_5SonnetLatest | Self::Claude3_5Sonnet20241022 | Self::Claude3_5Sonnet20240620 |
            Self::Claude3Sonnet20240229 => "sonnet",
            Self::Claude3_5HaikuLatest | Self::Claude3_5Haiku20241022 | Self::Claude3Haiku20240307 => "haiku",
            Self::Claude3OpusLatest | Self::Claude3Opus20240229 => "opus",
            Self::Claude2_1 | Self::Claude2_0 => "claude-2",
        }
    }
    
    /// Check if this model supports vision (image inputs)
    pub fn supports_vision(&self) -> bool {
        match self {
            Self::Claude2_1 | Self::Claude2_0 => false,
            _ => true,
        }
    }
    
    /// Check if this model supports tool use
    pub fn supports_tools(&self) -> bool {
        match self {
            Self::Claude2_1 | Self::Claude2_0 => false,
            _ => true,
        }
    }
}

impl std::fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<Model> for String {
    fn from(model: Model) -> String {
        model.as_str().to_string()
    }
} 