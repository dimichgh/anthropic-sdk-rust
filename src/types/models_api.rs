use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model object returned by the API
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelObject {
    /// Unique model identifier
    pub id: String,
    
    /// Human-readable name for the model
    pub display_name: String,
    
    /// RFC 3339 datetime string representing when the model was released
    pub created_at: DateTime<Utc>,
    
    /// Object type, always "model" for models
    #[serde(rename = "type")]
    pub object_type: String,
}

/// Parameters for listing models
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelListParams {
    /// ID of the object to use as a cursor for pagination (before)
    pub before_id: Option<String>,
    
    /// ID of the object to use as a cursor for pagination (after)
    pub after_id: Option<String>,
    
    /// Number of items to return per page (1-1000, default 20)
    pub limit: Option<u32>,
}

/// Paginated list of models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelList {
    /// Array of model objects
    pub data: Vec<ModelObject>,
    
    /// First ID in the data list
    pub first_id: Option<String>,
    
    /// Last ID in the data list  
    pub last_id: Option<String>,
    
    /// Indicates if there are more results available
    pub has_more: bool,
}

/// Model capabilities and limitations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    /// Maximum context length in tokens
    pub max_context_length: u64,
    
    /// Maximum output tokens per request
    pub max_output_tokens: u64,
    
    /// Supported capabilities
    pub capabilities: Vec<ModelCapability>,
    
    /// Model family (e.g., "claude-3", "claude-3-5")
    pub family: String,
    
    /// Model generation/version
    pub generation: String,
    
    /// Whether the model supports vision (image input)
    pub supports_vision: bool,
    
    /// Whether the model supports tool use/function calling
    pub supports_tools: bool,
    
    /// Whether the model supports system messages
    pub supports_system_messages: bool,
    
    /// Whether the model supports streaming
    pub supports_streaming: bool,
    
    /// Supported languages (ISO codes)
    pub supported_languages: Vec<String>,
    
    /// Training data cutoff date
    pub training_cutoff: Option<DateTime<Utc>>,
}

/// Individual model capability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModelCapability {
    /// Text generation and conversation
    TextGeneration,
    /// Vision and image understanding
    Vision,
    /// Tool use and function calling
    ToolUse,
    /// Code generation and analysis
    CodeGeneration,
    /// Mathematical reasoning
    Mathematical,
    /// Creative writing
    Creative,
    /// Analysis and reasoning
    Analysis,
    /// Summarization
    Summarization,
    /// Translation between languages
    Translation,
    /// Long context handling
    LongContext,
}

/// Pricing information for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Model ID this pricing applies to
    pub model_id: String,
    
    /// Input token price in USD per 1M tokens
    pub input_price_per_million: f64,
    
    /// Output token price in USD per 1M tokens  
    pub output_price_per_million: f64,
    
    /// Batch input token price in USD per 1M tokens (if available)
    pub batch_input_price_per_million: Option<f64>,
    
    /// Batch output token price in USD per 1M tokens (if available)
    pub batch_output_price_per_million: Option<f64>,
    
    /// Cache write price in USD per 1M tokens (if available)
    pub cache_write_price_per_million: Option<f64>,
    
    /// Cache read price in USD per 1M tokens (if available)
    pub cache_read_price_per_million: Option<f64>,
    
    /// Pricing tier or category
    pub tier: PricingTier,
    
    /// Currency code (usually "USD")
    pub currency: String,
    
    /// When this pricing was last updated
    pub updated_at: DateTime<Utc>,
}

/// Pricing tier categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PricingTier {
    /// Premium/flagship models with highest capabilities
    Premium,
    /// Standard models with balanced price/performance
    Standard,
    /// Fast/efficient models optimized for speed
    Fast,
    /// Legacy models with older pricing
    Legacy,
}

/// Model comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelComparison {
    /// Models being compared
    pub models: Vec<ModelObject>,
    
    /// Capabilities comparison
    pub capabilities: Vec<ModelCapabilities>,
    
    /// Pricing comparison
    pub pricing: Vec<ModelPricing>,
    
    /// Performance characteristics
    pub performance: Vec<ModelPerformance>,
    
    /// Comparison summary and recommendations
    pub summary: ComparisonSummary,
}

/// Performance characteristics for a model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPerformance {
    /// Model ID
    pub model_id: String,
    
    /// Relative speed score (1-10, higher is faster)
    pub speed_score: u8,
    
    /// Relative quality score (1-10, higher is better)
    pub quality_score: u8,
    
    /// Average response time in milliseconds
    pub avg_response_time_ms: Option<u64>,
    
    /// Tokens per second throughput
    pub tokens_per_second: Option<f64>,
    
    /// Cost efficiency score (1-10, higher is more cost effective)
    pub cost_efficiency_score: u8,
}

/// Summary of model comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonSummary {
    /// Best model for speed
    pub fastest_model: String,
    
    /// Best model for quality
    pub highest_quality_model: String,
    
    /// Most cost-effective model
    pub most_cost_effective_model: String,
    
    /// Best overall balanced model
    pub best_overall_model: String,
    
    /// Key differences and trade-offs
    pub key_differences: Vec<String>,
    
    /// Recommendations by use case
    pub use_case_recommendations: HashMap<String, String>,
}

/// Requirements for model selection
#[derive(Debug, Clone, Default)]
pub struct ModelRequirements {
    /// Maximum cost per input token
    pub max_input_cost_per_token: Option<f64>,
    
    /// Maximum cost per output token
    pub max_output_cost_per_token: Option<f64>,
    
    /// Minimum context length required
    pub min_context_length: Option<u64>,
    
    /// Required capabilities
    pub required_capabilities: Vec<ModelCapability>,
    
    /// Preferred model family
    pub preferred_family: Option<String>,
    
    /// Minimum speed score
    pub min_speed_score: Option<u8>,
    
    /// Minimum quality score
    pub min_quality_score: Option<u8>,
    
    /// Whether vision support is required
    pub requires_vision: Option<bool>,
    
    /// Whether tool use support is required
    pub requires_tools: Option<bool>,
    
    /// Maximum acceptable response time in milliseconds
    pub max_response_time_ms: Option<u64>,
    
    /// Preferred languages (ISO codes)
    pub preferred_languages: Vec<String>,
}

/// Usage recommendations for specific use cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsageRecommendations {
    /// Use case category
    pub use_case: String,
    
    /// Recommended models in order of preference
    pub recommended_models: Vec<ModelRecommendation>,
    
    /// General guidelines for this use case
    pub guidelines: Vec<String>,
    
    /// Recommended parameters
    pub recommended_parameters: RecommendedParameters,
    
    /// Common pitfalls to avoid
    pub pitfalls: Vec<String>,
    
    /// Expected performance characteristics
    pub expected_performance: PerformanceExpectations,
}

/// Individual model recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecommendation {
    /// Model ID
    pub model_id: String,
    
    /// Reason for recommendation
    pub reason: String,
    
    /// Confidence score (1-10)
    pub confidence_score: u8,
    
    /// Expected cost range for typical usage
    pub cost_range: CostRange,
    
    /// Specific strengths for this use case
    pub strengths: Vec<String>,
    
    /// Potential limitations
    pub limitations: Vec<String>,
}

/// Recommended parameters for a use case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedParameters {
    /// Recommended temperature range
    pub temperature_range: (f32, f32),
    
    /// Recommended max_tokens range
    pub max_tokens_range: (u32, u32),
    
    /// Recommended top_p range
    pub top_p_range: Option<(f32, f32)>,
    
    /// Whether to use streaming
    pub use_streaming: Option<bool>,
    
    /// Recommended system message patterns
    pub system_message_patterns: Vec<String>,
}

/// Expected performance for a use case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceExpectations {
    /// Expected response time range in milliseconds
    pub response_time_range_ms: (u64, u64),
    
    /// Expected cost range for typical request
    pub cost_range: CostRange,
    
    /// Expected quality level
    pub quality_level: QualityLevel,
    
    /// Success rate expectations
    pub success_rate_percentage: f32,
}

/// Cost range information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRange {
    /// Minimum cost in USD
    pub min_cost_usd: f64,
    
    /// Maximum cost in USD
    pub max_cost_usd: f64,
    
    /// Typical/average cost in USD
    pub typical_cost_usd: f64,
}

/// Quality level categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum QualityLevel {
    /// Highest quality, most accurate
    Excellent,
    /// Good quality, reliable
    Good,
    /// Acceptable quality, some limitations
    Acceptable,
    /// Lower quality, may need refinement
    Basic,
}

/// Cost estimation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimation {
    /// Model ID
    pub model_id: String,
    
    /// Input tokens
    pub input_tokens: u64,
    
    /// Output tokens
    pub output_tokens: u64,
    
    /// Input cost in USD
    pub input_cost_usd: f64,
    
    /// Output cost in USD
    pub output_cost_usd: f64,
    
    /// Total cost in USD
    pub total_cost_usd: f64,
    
    /// Batch discount (if applicable)
    pub batch_discount_usd: Option<f64>,
    
    /// Cache savings (if applicable)
    pub cache_savings_usd: Option<f64>,
    
    /// Final cost after discounts
    pub final_cost_usd: f64,
    
    /// Cost breakdown
    pub breakdown: CostBreakdown,
}

/// Detailed cost breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    /// Cost per input token
    pub cost_per_input_token_usd: f64,
    
    /// Cost per output token
    pub cost_per_output_token_usd: f64,
    
    /// Effective cost per token (total / total tokens)
    pub effective_cost_per_token_usd: f64,
    
    /// Cost comparison to other models
    pub cost_vs_alternatives: HashMap<String, f64>,
}

impl ModelListParams {
    /// Create new model list parameters
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the before_id for pagination
    pub fn before_id(mut self, before_id: impl Into<String>) -> Self {
        self.before_id = Some(before_id.into());
        self
    }
    
    /// Set the after_id for pagination
    pub fn after_id(mut self, after_id: impl Into<String>) -> Self {
        self.after_id = Some(after_id.into());
        self
    }
    
    /// Set the limit for pagination
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit.min(1000).max(1));
        self
    }
}

impl ModelRequirements {
    /// Create new model requirements
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set maximum input cost per token
    pub fn max_input_cost_per_token(mut self, cost: f64) -> Self {
        self.max_input_cost_per_token = Some(cost);
        self
    }
    
    /// Set maximum output cost per token
    pub fn max_output_cost_per_token(mut self, cost: f64) -> Self {
        self.max_output_cost_per_token = Some(cost);
        self
    }
    
    /// Set minimum context length requirement
    pub fn min_context_length(mut self, length: u64) -> Self {
        self.min_context_length = Some(length);
        self
    }
    
    /// Add required capability
    pub fn require_capability(mut self, capability: ModelCapability) -> Self {
        self.required_capabilities.push(capability);
        self
    }
    
    /// Set required capabilities
    pub fn capabilities(mut self, capabilities: Vec<ModelCapability>) -> Self {
        self.required_capabilities = capabilities;
        self
    }
    
    /// Set preferred model family
    pub fn preferred_family(mut self, family: impl Into<String>) -> Self {
        self.preferred_family = Some(family.into());
        self
    }
    
    /// Require vision support
    pub fn require_vision(mut self) -> Self {
        self.requires_vision = Some(true);
        self
    }
    
    /// Require tool use support
    pub fn require_tools(mut self) -> Self {
        self.requires_tools = Some(true);
        self
    }
    
    /// Set minimum quality score
    pub fn min_quality_score(mut self, score: u8) -> Self {
        self.min_quality_score = Some(score.min(10));
        self
    }
    
    /// Set minimum speed score
    pub fn min_speed_score(mut self, score: u8) -> Self {
        self.min_speed_score = Some(score.min(10));
        self
    }
}

impl ModelObject {
    /// Check if this is a latest/alias model
    pub fn is_alias(&self) -> bool {
        self.id.contains("latest") || self.id.ends_with("-0")
    }
    
    /// Get the model family (e.g., "claude-3-5" from "claude-3-5-sonnet-latest")
    pub fn family(&self) -> String {
        let parts: Vec<&str> = self.id.split('-').collect();
        if parts.len() >= 3 {
            format!("{}-{}", parts[0], parts[1])
        } else {
            parts[0].to_string()
        }
    }
    
    /// Check if model belongs to a specific family
    pub fn is_family(&self, family: &str) -> bool {
        self.id.starts_with(family)
    }
    
    /// Get model size/tier (sonnet, haiku, opus)
    pub fn model_size(&self) -> Option<String> {
        if self.id.contains("opus") {
            Some("opus".to_string())
        } else if self.id.contains("sonnet") {
            Some("sonnet".to_string())
        } else if self.id.contains("haiku") {
            Some("haiku".to_string())
        } else {
            None
        }
    }
}

impl ModelComparison {
    /// Get the best model for a specific criterion
    pub fn best_for_speed(&self) -> Option<&ModelObject> {
        self.performance
            .iter()
            .max_by_key(|p| p.speed_score)
            .and_then(|p| self.models.iter().find(|m| m.id == p.model_id))
    }
    
    /// Get the best model for quality
    pub fn best_for_quality(&self) -> Option<&ModelObject> {
        self.performance
            .iter()
            .max_by_key(|p| p.quality_score)
            .and_then(|p| self.models.iter().find(|m| m.id == p.model_id))
    }
    
    /// Get the most cost-effective model
    pub fn most_cost_effective(&self) -> Option<&ModelObject> {
        self.performance
            .iter()
            .max_by_key(|p| p.cost_efficiency_score)
            .and_then(|p| self.models.iter().find(|m| m.id == p.model_id))
    }
}

impl CostEstimation {
    /// Calculate cost per 1000 tokens
    pub fn cost_per_1k_tokens(&self) -> f64 {
        let total_tokens = self.input_tokens + self.output_tokens;
        if total_tokens > 0 {
            (self.final_cost_usd * 1000.0) / total_tokens as f64
        } else {
            0.0
        }
    }
    
    /// Get savings percentage from discounts
    pub fn savings_percentage(&self) -> f64 {
        let original_cost = self.input_cost_usd + self.output_cost_usd;
        if original_cost > 0.0 {
            ((original_cost - self.final_cost_usd) / original_cost) * 100.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_list_params_builder() {
        let params = ModelListParams::new()
            .limit(50)
            .after_id("model_123");
            
        assert_eq!(params.limit, Some(50));
        assert_eq!(params.after_id, Some("model_123".to_string()));
        assert_eq!(params.before_id, None);
    }
    
    #[test]
    fn test_model_requirements_builder() {
        let requirements = ModelRequirements::new()
            .max_input_cost_per_token(0.01)
            .min_context_length(100000)
            .require_vision()
            .require_capability(ModelCapability::ToolUse);
            
        assert_eq!(requirements.max_input_cost_per_token, Some(0.01));
        assert_eq!(requirements.min_context_length, Some(100000));
        assert_eq!(requirements.requires_vision, Some(true));
        assert!(requirements.required_capabilities.contains(&ModelCapability::ToolUse));
    }
    
    #[test]
    fn test_model_object_methods() {
        let model = ModelObject {
            id: "claude-3-5-sonnet-latest".to_string(),
            display_name: "Claude 3.5 Sonnet".to_string(),
            created_at: Utc::now(),
            object_type: "model".to_string(),
        };
        
        assert!(model.is_alias());
        assert_eq!(model.family(), "claude-3");
        assert!(model.is_family("claude-3-5"));
        assert_eq!(model.model_size(), Some("sonnet".to_string()));
    }
    
    #[test]
    fn test_cost_estimation_calculations() {
        let estimation = CostEstimation {
            model_id: "test-model".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            input_cost_usd: 0.01,
            output_cost_usd: 0.03,
            total_cost_usd: 0.04,
            batch_discount_usd: Some(0.005),
            cache_savings_usd: None,
            final_cost_usd: 0.035,
            breakdown: CostBreakdown {
                cost_per_input_token_usd: 0.00001,
                cost_per_output_token_usd: 0.00006,
                effective_cost_per_token_usd: 0.000023,
                cost_vs_alternatives: HashMap::new(),
            },
        };
        
        assert!((estimation.cost_per_1k_tokens() - 0.02333).abs() < 0.001);
        assert!((estimation.savings_percentage() - 12.5).abs() < 0.1);
    }
    
    #[test]
    fn test_limit_validation() {
        let params = ModelListParams::new().limit(2000);
        assert_eq!(params.limit, Some(1000)); // Should be clamped to max
        
        let params = ModelListParams::new().limit(0);
        assert_eq!(params.limit, Some(1)); // Should be clamped to min
    }
    
    #[test]
    fn test_model_capability_serialization() {
        let capability = ModelCapability::Vision;
        let serialized = serde_json::to_string(&capability).unwrap();
        assert_eq!(serialized, "\"vision\"");
        
        let deserialized: ModelCapability = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, ModelCapability::Vision);
    }
} 