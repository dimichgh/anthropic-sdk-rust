use std::time::{Duration, SystemTime};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use crate::types::Usage;

/// Enhanced token counting and cost estimation utilities
pub struct TokenCounter {
    /// Accumulated usage statistics
    usage_stats: Arc<Mutex<UsageStats>>,
    /// Model pricing information
    pricing: ModelPricing,
}

/// Accumulated usage statistics across all requests
#[derive(Debug, Clone)]
pub struct UsageStats {
    /// Total input tokens across all requests
    pub total_input_tokens: u32,
    /// Total output tokens across all requests
    pub total_output_tokens: u32,
    /// Total cache read tokens
    pub total_cache_read_tokens: u32,
    /// Total cache write tokens  
    pub total_cache_write_tokens: u32,
    /// Number of requests made
    pub request_count: u32,
    /// Total cost in USD
    pub total_cost_usd: f64,
    /// Usage by model
    pub model_usage: HashMap<String, ModelUsage>,
    /// Session start time
    pub session_start: SystemTime,
    /// Last request time
    pub last_request: Option<SystemTime>,
}

/// Usage statistics for a specific model
#[derive(Debug, Clone, Default)]
pub struct ModelUsage {
    /// Input tokens for this model
    pub input_tokens: u32,
    /// Output tokens for this model
    pub output_tokens: u32,
    /// Cache read tokens for this model
    pub cache_read_tokens: u32,
    /// Cache write tokens for this model
    pub cache_write_tokens: u32,
    /// Number of requests for this model
    pub request_count: u32,
    /// Total cost for this model
    pub cost_usd: f64,
}

/// Real-time usage tracking for a single request
#[derive(Debug, Clone)]
pub struct RequestUsage {
    /// Input tokens for this request
    pub input_tokens: u32,
    /// Output tokens accumulated so far
    pub output_tokens: u32,
    /// Cache read tokens for this request
    pub cache_read_tokens: u32,
    /// Cache write tokens for this request
    pub cache_write_tokens: u32,
    /// Model used for this request
    pub model: String,
    /// Request start time
    pub start_time: SystemTime,
    /// Request completion time
    pub end_time: Option<SystemTime>,
    /// Cost for this request
    pub cost_usd: f64,
}

/// Model pricing information
#[derive(Debug, Clone)]
pub struct ModelPricing {
    /// Pricing per model
    pricing_table: HashMap<String, ModelPrice>,
}

/// Pricing for a specific model
#[derive(Debug, Clone)]
pub struct ModelPrice {
    /// Cost per 1M input tokens in USD
    pub input_cost_per_million: f64,
    /// Cost per 1M output tokens in USD
    pub output_cost_per_million: f64,
    /// Cost per 1M cache read tokens in USD (if applicable)
    pub cache_read_cost_per_million: Option<f64>,
    /// Cost per 1M cache write tokens in USD (if applicable)
    pub cache_write_cost_per_million: Option<f64>,
}

/// Cost breakdown for detailed analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    /// Input token cost
    pub input_cost: f64,
    /// Output token cost
    pub output_cost: f64,
    /// Cache read cost
    pub cache_read_cost: f64,
    /// Cache write cost
    pub cache_write_cost: f64,
    /// Total cost
    pub total_cost: f64,
    /// Cost per token
    pub cost_per_token: f64,
    /// Model used
    pub model: String,
}

/// Usage summary for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    /// Total tokens (input + output)
    pub total_tokens: u32,
    /// Input tokens
    pub input_tokens: u32,
    /// Output tokens
    pub output_tokens: u32,
    /// Cache tokens
    pub cache_tokens: u32,
    /// Total cost
    pub total_cost_usd: f64,
    /// Average cost per token
    pub avg_cost_per_token: f64,
    /// Session duration
    pub session_duration: Duration,
    /// Requests per minute
    pub requests_per_minute: f64,
    /// Tokens per minute
    pub tokens_per_minute: f64,
    /// Cost per request
    pub avg_cost_per_request: f64,
}

impl TokenCounter {
    /// Create a new token counter with default pricing
    pub fn new() -> Self {
        Self {
            usage_stats: Arc::new(Mutex::new(UsageStats::new())),
            pricing: ModelPricing::default(),
        }
    }

    /// Create a token counter with custom pricing
    pub fn with_pricing(pricing: ModelPricing) -> Self {
        Self {
            usage_stats: Arc::new(Mutex::new(UsageStats::new())),
            pricing,
        }
    }

    /// Record usage from a completed request
    pub fn record_usage(&self, model: &str, usage: &Usage) -> CostBreakdown {
        let cost_breakdown = self.calculate_cost(model, usage);
        
        let mut stats = self.usage_stats.lock().unwrap();
        stats.add_usage(model, usage, cost_breakdown.total_cost);
        
        cost_breakdown
    }

    /// Start tracking a new request
    pub fn start_request(&self, model: &str) -> RequestUsage {
        RequestUsage {
            model: model.to_string(),
            start_time: SystemTime::now(),
            ..Default::default()
        }
    }

    /// Calculate cost for a usage
    pub fn calculate_cost(&self, model: &str, usage: &Usage) -> CostBreakdown {
        let price = self.pricing.get_price(model);
        
        let input_cost = (usage.input_tokens as f64 / 1_000_000.0) * price.input_cost_per_million;
        let output_cost = (usage.output_tokens as f64 / 1_000_000.0) * price.output_cost_per_million;
        
        let cache_read_tokens = usage.cache_read_input_tokens.unwrap_or(0);
        let cache_write_tokens = usage.cache_creation_input_tokens.unwrap_or(0);
        
        let cache_read_cost = price.cache_read_cost_per_million
            .map(|rate| (cache_read_tokens as f64 / 1_000_000.0) * rate)
            .unwrap_or(0.0);
            
        let cache_write_cost = price.cache_write_cost_per_million
            .map(|rate| (cache_write_tokens as f64 / 1_000_000.0) * rate)
            .unwrap_or(0.0);
        
        let total_cost = input_cost + output_cost + cache_read_cost + cache_write_cost;
        let total_tokens = usage.input_tokens + usage.output_tokens + cache_read_tokens + cache_write_tokens;
        let cost_per_token = if total_tokens > 0 {
            total_cost / total_tokens as f64
        } else {
            0.0
        };

        CostBreakdown {
            input_cost,
            output_cost,
            cache_read_cost,
            cache_write_cost,
            total_cost,
            cost_per_token,
            model: model.to_string(),
        }
    }

    /// Get current usage statistics
    pub fn get_stats(&self) -> UsageStats {
        self.usage_stats.lock().unwrap().clone()
    }

    /// Get usage summary
    pub fn get_summary(&self) -> UsageSummary {
        let stats = self.usage_stats.lock().unwrap();
        stats.to_summary()
    }

    /// Reset usage statistics
    pub fn reset(&self) {
        let mut stats = self.usage_stats.lock().unwrap();
        *stats = UsageStats::new();
    }

    /// Estimate cost for a request before sending
    pub fn estimate_cost(&self, model: &str, estimated_input_tokens: u32, estimated_output_tokens: u32) -> f64 {
        let usage = Usage {
            input_tokens: estimated_input_tokens,
            output_tokens: estimated_output_tokens,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
            server_tool_use: None,
            service_tier: None,
        };
        
        self.calculate_cost(model, &usage).total_cost
    }
}

impl UsageStats {
    fn new() -> Self {
        Self {
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_cache_read_tokens: 0,
            total_cache_write_tokens: 0,
            request_count: 0,
            total_cost_usd: 0.0,
            model_usage: HashMap::new(),
            session_start: SystemTime::now(),
            last_request: None,
        }
    }

    fn add_usage(&mut self, model: &str, usage: &Usage, cost: f64) {
        self.total_input_tokens += usage.input_tokens;
        self.total_output_tokens += usage.output_tokens;
        self.total_cache_read_tokens += usage.cache_read_input_tokens.unwrap_or(0);
        self.total_cache_write_tokens += usage.cache_creation_input_tokens.unwrap_or(0);
        self.total_cost_usd += cost;
        self.request_count += 1;
        self.last_request = Some(SystemTime::now());

        // Update model-specific usage
        let model_usage = self.model_usage.entry(model.to_string()).or_default();
        model_usage.input_tokens += usage.input_tokens;
        model_usage.output_tokens += usage.output_tokens;
        model_usage.cache_read_tokens += usage.cache_read_input_tokens.unwrap_or(0);
        model_usage.cache_write_tokens += usage.cache_creation_input_tokens.unwrap_or(0);
        model_usage.cost_usd += cost;
        model_usage.request_count += 1;
    }

    fn to_summary(&self) -> UsageSummary {
        let total_tokens = self.total_input_tokens + self.total_output_tokens;
        let cache_tokens = self.total_cache_read_tokens + self.total_cache_write_tokens;
        
        let session_duration = self.session_start.elapsed().unwrap_or(Duration::ZERO);
        let session_minutes = session_duration.as_secs_f64() / 60.0;
        
        let requests_per_minute = if session_minutes > 0.0 {
            self.request_count as f64 / session_minutes
        } else {
            0.0
        };
        
        let tokens_per_minute = if session_minutes > 0.0 {
            total_tokens as f64 / session_minutes
        } else {
            0.0
        };
        
        let avg_cost_per_token = if total_tokens > 0 {
            self.total_cost_usd / total_tokens as f64
        } else {
            0.0
        };
        
        let avg_cost_per_request = if self.request_count > 0 {
            self.total_cost_usd / self.request_count as f64
        } else {
            0.0
        };

        UsageSummary {
            total_tokens,
            input_tokens: self.total_input_tokens,
            output_tokens: self.total_output_tokens,
            cache_tokens,
            total_cost_usd: self.total_cost_usd,
            avg_cost_per_token,
            session_duration,
            requests_per_minute,
            tokens_per_minute,
            avg_cost_per_request,
        }
    }
}

impl ModelPricing {
    /// Create default pricing with current Anthropic rates
    pub fn default() -> Self {
        let mut pricing_table = HashMap::new();
        
        // Claude 3.5 Sonnet (latest)
        pricing_table.insert("claude-3-5-sonnet-latest".to_string(), ModelPrice {
            input_cost_per_million: 3.00,
            output_cost_per_million: 15.00,
            cache_read_cost_per_million: Some(0.30),
            cache_write_cost_per_million: Some(3.75),
        });
        
        // Claude 3.5 Sonnet (20241022)
        pricing_table.insert("claude-3-5-sonnet-20241022".to_string(), ModelPrice {
            input_cost_per_million: 3.00,
            output_cost_per_million: 15.00,
            cache_read_cost_per_million: Some(0.30),
            cache_write_cost_per_million: Some(3.75),
        });
        
        // Claude 3.5 Haiku (latest)
        pricing_table.insert("claude-3-5-haiku-latest".to_string(), ModelPrice {
            input_cost_per_million: 1.00,
            output_cost_per_million: 5.00,
            cache_read_cost_per_million: Some(0.10),
            cache_write_cost_per_million: Some(1.25),
        });
        
        // Claude 3 Opus
        pricing_table.insert("claude-3-opus-20240229".to_string(), ModelPrice {
            input_cost_per_million: 15.00,
            output_cost_per_million: 75.00,
            cache_read_cost_per_million: Some(1.50),
            cache_write_cost_per_million: Some(18.75),
        });

        Self { pricing_table }
    }

    /// Get pricing for a model
    pub fn get_price(&self, model: &str) -> &ModelPrice {
        self.pricing_table.get(model)
            .unwrap_or_else(|| {
                // Default to Claude 3.5 Sonnet pricing for unknown models
                self.pricing_table.get("claude-3-5-sonnet-latest").unwrap()
            })
    }

    /// Set pricing for a model
    pub fn set_price(&mut self, model: &str, price: ModelPrice) {
        self.pricing_table.insert(model.to_string(), price);
    }
}

impl Default for TokenCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for UsageStats {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RequestUsage {
    fn default() -> Self {
        Self {
            input_tokens: 0,
            output_tokens: 0,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
            model: String::new(),
            start_time: SystemTime::now(),
            end_time: None,
            cost_usd: 0.0,
        }
    }
}

impl std::fmt::Display for UsageSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "Usage Summary:\n\
             Total Tokens: {} (Input: {}, Output: {}, Cache: {})\n\
             Total Cost: ${:.4}\n\
             Avg Cost/Token: ${:.6}\n\
             Avg Cost/Request: ${:.4}\n\
             Session Duration: {:.1}min\n\
             Rate: {:.1} tokens/min, {:.1} requests/min",
            self.total_tokens,
            self.input_tokens,
            self.output_tokens,
            self.cache_tokens,
            self.total_cost_usd,
            self.avg_cost_per_token,
            self.avg_cost_per_request,
            self.session_duration.as_secs_f64() / 60.0,
            self.tokens_per_minute,
            self.requests_per_minute
        )
    }
}

impl std::fmt::Display for CostBreakdown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "Cost Breakdown ({}):\n\
             Input: ${:.4}\n\
             Output: ${:.4}\n\
             Cache Read: ${:.4}\n\
             Cache Write: ${:.4}\n\
             Total: ${:.4} (${:.6}/token)",
            self.model,
            self.input_cost,
            self.output_cost,
            self.cache_read_cost,
            self.cache_write_cost,
            self.total_cost,
            self.cost_per_token
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculation() {
        let counter = TokenCounter::new();
        let usage = Usage {
            input_tokens: 1000,
            output_tokens: 500,
            cache_creation_input_tokens: Some(100),
            cache_read_input_tokens: Some(200),
            server_tool_use: None,
            service_tier: None,
        };
        
        let cost = counter.calculate_cost("claude-3-5-sonnet-latest", &usage);
        
        // Expected: (1000/1M * $3) + (500/1M * $15) + (200/1M * $0.30) + (100/1M * $3.75)
        let expected = 0.003 + 0.0075 + 0.00006 + 0.000375;
        assert!((cost.total_cost - expected).abs() < 0.0001);
    }

    #[test]
    fn test_usage_tracking() {
        let counter = TokenCounter::new();
        let usage = Usage {
            input_tokens: 100,
            output_tokens: 50,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
            server_tool_use: None,
            service_tier: None,
        };
        
        counter.record_usage("claude-3-5-sonnet-latest", &usage);
        let stats = counter.get_stats();
        
        assert_eq!(stats.total_input_tokens, 100);
        assert_eq!(stats.total_output_tokens, 50);
        assert_eq!(stats.request_count, 1);
        assert!(stats.total_cost_usd > 0.0);
    }

    #[test]
    fn test_cost_estimation() {
        let counter = TokenCounter::new();
        let cost = counter.estimate_cost("claude-3-5-sonnet-latest", 1000, 500);
        assert!(cost > 0.0);
        
        // Should be: (1000/1M * $3) + (500/1M * $15) = $0.0105
        assert!((cost - 0.0105).abs() < 0.0001);
    }
}
