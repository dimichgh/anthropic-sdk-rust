use crate::client::Anthropic;
use crate::types::{
    AnthropicError, Result, 
    ModelObject, ModelListParams, ModelList, ModelCapabilities, ModelCapability,
    ModelPricing, PricingTier, ModelComparison, ModelPerformance, ComparisonSummary,
    ModelRequirements, ModelUsageRecommendations, CostEstimation, CostBreakdown,
};
use std::collections::HashMap;
use chrono::Utc;

/// Resource for managing models
pub struct ModelsResource<'a> {
    client: &'a Anthropic,
}

impl<'a> ModelsResource<'a> {
    pub(crate) fn new(client: &'a Anthropic) -> Self {
        Self { client }
    }

    /// List all available models with pagination support
    /// 
    /// # Arguments
    /// 
    /// * `params` - Optional pagination parameters
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use anthropic_sdk::{Anthropic, ModelListParams};
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Anthropic::from_env()?;
    /// 
    /// // List all models
    /// let models = client.models().list(None).await?;
    /// println!("Found {} models", models.data.len());
    /// 
    /// // List with pagination
    /// let params = ModelListParams::new().limit(10);
    /// let models = client.models().list(Some(params)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self, params: Option<ModelListParams>) -> Result<ModelList> {
        let mut query_params = Vec::new();
        
        if let Some(params) = params {
            if let Some(before_id) = params.before_id {
                query_params.push(("before_id", before_id));
            }
            if let Some(after_id) = params.after_id {
                query_params.push(("after_id", after_id));
            }
            if let Some(limit) = params.limit {
                query_params.push(("limit", limit.to_string()));
            }
        }
        
        let url = format!("{}/v1/models", self.client.config().base_url);
        let response = self.client.http_client()
            .get(&url)
            .query(&query_params)
            .send()
            .await?;
            
        if response.status().is_success() {
            let model_list: ModelList = response.json().await?;
            Ok(model_list)
        } else {
            let status = response.status().as_u16();
            let error_text = response.text().await?;
            Err(AnthropicError::from_status(status, error_text))
        }
    }

    /// Get a specific model by ID or alias
    /// 
    /// # Arguments
    /// 
    /// * `model_id` - Model identifier or alias (e.g., "claude-3-5-sonnet-latest")
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use anthropic_sdk::Anthropic;
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Anthropic::from_env()?;
    /// 
    /// // Get specific model
    /// let model = client.models().get("claude-3-5-sonnet-latest").await?;
    /// println!("Model: {} ({})", model.display_name, model.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, model_id: &str) -> Result<ModelObject> {
        let url = format!("{}/v1/models/{}", self.client.config().base_url, model_id);
        let response = self.client.http_client()
            .get(&url)
            .send()
            .await?;
            
        if response.status().is_success() {
            let model: ModelObject = response.json().await?;
            Ok(model)
        } else {
            let status = response.status().as_u16();
            let error_text = response.text().await?;
            Err(AnthropicError::from_status(status, error_text))
        }
    }

    /// List models by family (e.g., "claude-3", "claude-3-5")
    /// 
    /// # Arguments
    /// 
    /// * `family` - Model family to filter by
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use anthropic_sdk::Anthropic;
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Anthropic::from_env()?;
    /// 
    /// let claude35_models = client.models().list_by_family("claude-3-5").await?;
    /// println!("Found {} Claude 3.5 models", claude35_models.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_by_family(&self, family: &str) -> Result<Vec<ModelObject>> {
        let all_models = self.list(None).await?;
        let filtered_models = all_models.data
            .into_iter()
            .filter(|model| model.is_family(family))
            .collect();
        
        Ok(filtered_models)
    }

    /// Get model capabilities and limitations
    /// 
    /// Note: This method provides enhanced capabilities based on known model information.
    /// The actual API may not return all these details.
    /// 
    /// # Arguments
    /// 
    /// * `model_id` - Model identifier
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use anthropic_sdk::Anthropic;
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Anthropic::from_env()?;
    /// 
    /// let capabilities = client.models().get_capabilities("claude-3-5-sonnet-latest").await?;
    /// println!("Max context: {} tokens", capabilities.max_context_length);
    /// println!("Supports vision: {}", capabilities.supports_vision);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_capabilities(&self, model_id: &str) -> Result<ModelCapabilities> {
        // Get the base model info
        let model = self.get(model_id).await?;
        
        // Enhanced capabilities based on known model information
        let capabilities = self.derive_capabilities(&model);
        Ok(capabilities)
    }

    /// Get current pricing information for a model
    /// 
    /// Note: This method provides estimated pricing based on known information.
    /// Actual pricing may vary and should be verified with official documentation.
    /// 
    /// # Arguments
    /// 
    /// * `model_id` - Model identifier
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use anthropic_sdk::Anthropic;
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Anthropic::from_env()?;
    /// 
    /// let pricing = client.models().get_pricing("claude-3-5-sonnet-latest").await?;
    /// println!("Input: ${:.3}/1M tokens", pricing.input_price_per_million);
    /// println!("Output: ${:.3}/1M tokens", pricing.output_price_per_million);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_pricing(&self, model_id: &str) -> Result<ModelPricing> {
        let model = self.get(model_id).await?;
        let pricing = self.derive_pricing(&model);
        Ok(pricing)
    }

    /// Find the best model based on requirements
    /// 
    /// # Arguments
    /// 
    /// * `requirements` - Model requirements and preferences
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use anthropic_sdk::{Anthropic, ModelRequirements, ModelCapability};
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Anthropic::from_env()?;
    /// 
    /// let requirements = ModelRequirements::new()
    ///     .max_input_cost_per_token(0.01)
    ///     .min_context_length(100000)
    ///     .require_capability(ModelCapability::Vision);
    /// 
    /// let best_model = client.models().find_best_model(&requirements).await?;
    /// println!("Best model: {}", best_model.display_name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_best_model(&self, requirements: &ModelRequirements) -> Result<ModelObject> {
        let all_models = self.list(None).await?;
        let mut scored_models = Vec::new();
        
        for model in all_models.data {
            let capabilities = self.derive_capabilities(&model);
            let pricing = self.derive_pricing(&model);
            let performance = self.derive_performance(&model);
            
            if let Some(score) = self.score_model(&model, &capabilities, &pricing, &performance, requirements) {
                scored_models.push((score, model));
            }
        }
        
        scored_models.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        
        scored_models.into_iter()
            .next()
            .map(|(_, model)| model)
            .ok_or_else(|| AnthropicError::Other("No models match the requirements".to_string()))
    }

    /// Compare multiple models side by side
    /// 
    /// # Arguments
    /// 
    /// * `model_ids` - List of model identifiers to compare
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use anthropic_sdk::Anthropic;
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Anthropic::from_env()?;
    /// 
    /// let comparison = client.models().compare_models(&[
    ///     "claude-3-5-sonnet-latest",
    ///     "claude-3-5-haiku-latest"
    /// ]).await?;
    /// 
    /// println!("Fastest: {}", comparison.summary.fastest_model);
    /// println!("Best quality: {}", comparison.summary.highest_quality_model);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn compare_models(&self, model_ids: &[&str]) -> Result<ModelComparison> {
        let mut models = Vec::new();
        let mut capabilities = Vec::new();
        let mut pricing = Vec::new();
        let mut performance = Vec::new();
        
        for model_id in model_ids {
            let model = self.get(model_id).await?;
            let caps = self.derive_capabilities(&model);
            let price = self.derive_pricing(&model);
            let perf = self.derive_performance(&model);
            
            models.push(model);
            capabilities.push(caps);
            pricing.push(price);
            performance.push(perf);
        }
        
        let summary = self.create_comparison_summary(&models, &performance);
        
        Ok(ModelComparison {
            models,
            capabilities,
            pricing,
            performance,
            summary,
        })
    }

    /// Estimate cost for specific usage patterns
    /// 
    /// # Arguments
    /// 
    /// * `model_id` - Model identifier
    /// * `input_tokens` - Expected input tokens
    /// * `output_tokens` - Expected output tokens
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use anthropic_sdk::Anthropic;
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Anthropic::from_env()?;
    /// 
    /// let cost = client.models().estimate_cost("claude-3-5-sonnet-latest", 1000, 500).await?;
    /// println!("Estimated cost: ${:.4}", cost.final_cost_usd);
    /// println!("Cost per 1K tokens: ${:.4}", cost.cost_per_1k_tokens());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn estimate_cost(&self, model_id: &str, input_tokens: u64, output_tokens: u64) -> Result<CostEstimation> {
        let pricing = self.get_pricing(model_id).await?;
        
        let input_cost_usd = (input_tokens as f64 / 1_000_000.0) * pricing.input_price_per_million;
        let output_cost_usd = (output_tokens as f64 / 1_000_000.0) * pricing.output_price_per_million;
        let total_cost_usd = input_cost_usd + output_cost_usd;
        
        // Apply potential batch discount
        let batch_discount_usd = if input_tokens + output_tokens > 100_000 {
            Some(total_cost_usd * 0.1) // 10% discount for large batches
        } else {
            None
        };
        
        let final_cost_usd = total_cost_usd - batch_discount_usd.unwrap_or(0.0);
        
        let breakdown = CostBreakdown {
            cost_per_input_token_usd: pricing.input_price_per_million / 1_000_000.0,
            cost_per_output_token_usd: pricing.output_price_per_million / 1_000_000.0,
            effective_cost_per_token_usd: final_cost_usd / (input_tokens + output_tokens) as f64,
            cost_vs_alternatives: HashMap::new(), // Could be populated with comparisons
        };
        
        Ok(CostEstimation {
            model_id: model_id.to_string(),
            input_tokens,
            output_tokens,
            input_cost_usd,
            output_cost_usd,
            total_cost_usd,
            batch_discount_usd,
            cache_savings_usd: None,
            final_cost_usd,
            breakdown,
        })
    }

    /// Get usage recommendations for specific use cases
    /// 
    /// # Arguments
    /// 
    /// * `use_case` - Use case category (e.g., "code-generation", "creative-writing")
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use anthropic_sdk::Anthropic;
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Anthropic::from_env()?;
    /// 
    /// let recommendations = client.models().get_recommendations("code-generation").await?;
    /// for rec in &recommendations.recommended_models {
    ///     println!("Recommended: {} - {}", rec.model_id, rec.reason);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_recommendations(&self, use_case: &str) -> Result<ModelUsageRecommendations> {
        let all_models = self.list(None).await?;
        let recommendations = self.create_use_case_recommendations(use_case, &all_models.data);
        Ok(recommendations)
    }

    // Helper methods for deriving model information
    
    fn derive_capabilities(&self, model: &ModelObject) -> ModelCapabilities {
        let model_id = &model.id;
        
        // Derive capabilities based on known model information
        let (max_context_length, max_output_tokens, supports_vision, supports_tools) = 
            if model_id.contains("claude-4") {
                (200_000, 8_192, true, true)
            } else if model_id.contains("claude-3-5") {
                (200_000, 8_192, model_id.contains("sonnet"), true)
            } else if model_id.contains("claude-3") && model_id.contains("opus") {
                (200_000, 4_096, true, true)
            } else if model_id.contains("claude-3") && model_id.contains("sonnet") {
                (200_000, 4_096, true, true)
            } else if model_id.contains("claude-3") && model_id.contains("haiku") {
                (200_000, 4_096, true, true)
            } else {
                (100_000, 4_096, false, false)
            };
        
        let capabilities = if supports_vision && supports_tools {
            vec![
                ModelCapability::TextGeneration,
                ModelCapability::Vision,
                ModelCapability::ToolUse,
                ModelCapability::CodeGeneration,
                ModelCapability::Mathematical,
                ModelCapability::Analysis,
                ModelCapability::Creative,
                ModelCapability::Summarization,
                ModelCapability::Translation,
                ModelCapability::LongContext,
            ]
        } else if supports_tools {
            vec![
                ModelCapability::TextGeneration,
                ModelCapability::ToolUse,
                ModelCapability::CodeGeneration,
                ModelCapability::Mathematical,
                ModelCapability::Analysis,
                ModelCapability::Creative,
                ModelCapability::Summarization,
                ModelCapability::Translation,
            ]
        } else {
            vec![
                ModelCapability::TextGeneration,
                ModelCapability::Creative,
                ModelCapability::Summarization,
                ModelCapability::Translation,
            ]
        };
        
        ModelCapabilities {
            max_context_length,
            max_output_tokens,
            capabilities,
            family: model.family(),
            generation: if model_id.contains("claude-4") {
                "4".to_string()
            } else if model_id.contains("claude-3-7") {
                "3.7".to_string()
            } else if model_id.contains("claude-3-5") {
                "3.5".to_string()
            } else if model_id.contains("claude-3") {
                "3".to_string()
            } else {
                "unknown".to_string()
            },
            supports_vision,
            supports_tools,
            supports_system_messages: true,
            supports_streaming: true,
            supported_languages: vec![
                "en".to_string(), "es".to_string(), "fr".to_string(), "de".to_string(),
                "it".to_string(), "pt".to_string(), "ru".to_string(), "ja".to_string(),
                "ko".to_string(), "zh".to_string(), "ar".to_string(), "hi".to_string(),
            ],
            training_cutoff: Some(model.created_at),
        }
    }
    
    fn derive_pricing(&self, model: &ModelObject) -> ModelPricing {
        let model_id = &model.id;
        
        // Estimated pricing based on known information (as of 2024)
        let (input_price, output_price, tier) = if model_id.contains("claude-4") {
            if model_id.contains("opus") {
                (15.0, 75.0, PricingTier::Premium)
            } else {
                (3.0, 15.0, PricingTier::Standard)
            }
        } else if model_id.contains("claude-3-7") {
            (3.0, 15.0, PricingTier::Standard)
        } else if model_id.contains("claude-3-5") {
            if model_id.contains("sonnet") {
                (3.0, 15.0, PricingTier::Standard)
            } else if model_id.contains("haiku") {
                (0.25, 1.25, PricingTier::Fast)
            } else {
                (3.0, 15.0, PricingTier::Standard)
            }
        } else if model_id.contains("claude-3") {
            if model_id.contains("opus") {
                (15.0, 75.0, PricingTier::Premium)
            } else if model_id.contains("sonnet") {
                (3.0, 15.0, PricingTier::Standard)
            } else if model_id.contains("haiku") {
                (0.25, 1.25, PricingTier::Fast)
            } else {
                (3.0, 15.0, PricingTier::Standard)
            }
        } else {
            (3.0, 15.0, PricingTier::Standard)
        };
        
        ModelPricing {
            model_id: model_id.clone(),
            input_price_per_million: input_price,
            output_price_per_million: output_price,
            batch_input_price_per_million: Some(input_price * 0.5), // 50% discount for batch
            batch_output_price_per_million: Some(output_price * 0.5),
            cache_write_price_per_million: Some(input_price * 1.25), // 25% premium for cache write
            cache_read_price_per_million: Some(input_price * 0.1), // 90% discount for cache read
            tier,
            currency: "USD".to_string(),
            updated_at: Utc::now(),
        }
    }
    
    fn derive_performance(&self, model: &ModelObject) -> ModelPerformance {
        let model_id = &model.id;
        
        // Performance characteristics based on known information
        let (speed_score, quality_score, cost_efficiency_score) = 
            if model_id.contains("claude-4") {
                if model_id.contains("opus") {
                    (6, 10, 5) // Slower but highest quality
                } else {
                    (8, 9, 8) // Fast and high quality
                }
            } else if model_id.contains("claude-3-7") {
                (8, 9, 8)
            } else if model_id.contains("claude-3-5") {
                if model_id.contains("sonnet") {
                    (8, 9, 8)
                } else if model_id.contains("haiku") {
                    (10, 7, 10) // Fastest and most cost-effective
                } else {
                    (8, 9, 8)
                }
            } else if model_id.contains("claude-3") {
                if model_id.contains("opus") {
                    (5, 10, 4)
                } else if model_id.contains("sonnet") {
                    (7, 8, 7)
                } else if model_id.contains("haiku") {
                    (10, 6, 9)
                } else {
                    (7, 8, 7)
                }
            } else {
                (7, 8, 7)
            };
        
        ModelPerformance {
            model_id: model_id.clone(),
            speed_score,
            quality_score,
            avg_response_time_ms: Some(match speed_score {
                10 => 500,
                9 => 750,
                8 => 1000,
                7 => 1500,
                6 => 2000,
                _ => 3000,
            }),
            tokens_per_second: Some(match speed_score {
                10 => 100.0,
                9 => 80.0,
                8 => 60.0,
                7 => 40.0,
                6 => 25.0,
                _ => 15.0,
            }),
            cost_efficiency_score,
        }
    }
    
    fn score_model(
        &self,
        model: &ModelObject,
        capabilities: &ModelCapabilities,
        pricing: &ModelPricing,
        performance: &ModelPerformance,
        requirements: &ModelRequirements,
    ) -> Option<f64> {
        let mut score = 0.0;
        let mut penalty = 0.0;
        
        // Check hard requirements
        if let Some(max_input_cost) = requirements.max_input_cost_per_token {
            let input_cost_per_token = pricing.input_price_per_million / 1_000_000.0;
            if input_cost_per_token > max_input_cost {
                return None; // Eliminate this model
            }
            score += (max_input_cost - input_cost_per_token) * 1000.0; // Reward lower cost
        }
        
        if let Some(max_output_cost) = requirements.max_output_cost_per_token {
            let output_cost_per_token = pricing.output_price_per_million / 1_000_000.0;
            if output_cost_per_token > max_output_cost {
                return None;
            }
            score += (max_output_cost - output_cost_per_token) * 1000.0;
        }
        
        if let Some(min_context) = requirements.min_context_length {
            if capabilities.max_context_length < min_context {
                return None;
            }
            score += (capabilities.max_context_length - min_context) as f64 / 1000.0;
        }
        
        // Check capability requirements
        for required_cap in &requirements.required_capabilities {
            if !capabilities.capabilities.contains(required_cap) {
                return None;
            }
            score += 10.0; // Reward having required capabilities
        }
        
        if let Some(requires_vision) = requirements.requires_vision {
            if requires_vision && !capabilities.supports_vision {
                return None;
            }
            if requires_vision && capabilities.supports_vision {
                score += 20.0;
            }
        }
        
        if let Some(requires_tools) = requirements.requires_tools {
            if requires_tools && !capabilities.supports_tools {
                return None;
            }
            if requires_tools && capabilities.supports_tools {
                score += 20.0;
            }
        }
        
        // Soft preferences
        if let Some(family) = &requirements.preferred_family {
            if model.is_family(family) {
                score += 15.0;
            }
        }
        
        if let Some(min_speed) = requirements.min_speed_score {
            if performance.speed_score < min_speed {
                penalty += 10.0;
            } else {
                score += (performance.speed_score - min_speed) as f64;
            }
        }
        
        if let Some(min_quality) = requirements.min_quality_score {
            if performance.quality_score < min_quality {
                penalty += 10.0;
            } else {
                score += (performance.quality_score - min_quality) as f64;
            }
        }
        
        // Add base performance scores
        score += performance.speed_score as f64;
        score += performance.quality_score as f64;
        score += performance.cost_efficiency_score as f64;
        
        Some(score - penalty)
    }
    
    fn create_comparison_summary(&self, _models: &[ModelObject], performance: &[ModelPerformance]) -> ComparisonSummary {
        let fastest_model = performance
            .iter()
            .max_by_key(|p| p.speed_score)
            .map(|p| p.model_id.clone())
            .unwrap_or_default();
            
        let highest_quality_model = performance
            .iter()
            .max_by_key(|p| p.quality_score)
            .map(|p| p.model_id.clone())
            .unwrap_or_default();
            
        let most_cost_effective_model = performance
            .iter()
            .max_by_key(|p| p.cost_efficiency_score)
            .map(|p| p.model_id.clone())
            .unwrap_or_default();
            
        // Simple heuristic for best overall: highest combined score
        let best_overall_model = performance
            .iter()
            .max_by_key(|p| p.speed_score + p.quality_score + p.cost_efficiency_score)
            .map(|p| p.model_id.clone())
            .unwrap_or_default();
        
        let key_differences = vec![
            "Performance varies significantly across models".to_string(),
            "Cost-effectiveness inversely correlated with quality".to_string(),
            "Vision support available in selected models".to_string(),
        ];
        
        let mut use_case_recommendations = HashMap::new();
        use_case_recommendations.insert("speed".to_string(), fastest_model.clone());
        use_case_recommendations.insert("quality".to_string(), highest_quality_model.clone());
        use_case_recommendations.insert("cost".to_string(), most_cost_effective_model.clone());
        use_case_recommendations.insert("balanced".to_string(), best_overall_model.clone());
        
        ComparisonSummary {
            fastest_model,
            highest_quality_model,
            most_cost_effective_model,
            best_overall_model,
            key_differences,
            use_case_recommendations,
        }
    }
    
    fn create_use_case_recommendations(&self, use_case: &str, _models: &[ModelObject]) -> ModelUsageRecommendations {
        use crate::types::{
            ModelRecommendation, RecommendedParameters, PerformanceExpectations,
            CostRange, QualityLevel,
        };
        
        // Create recommendations based on use case
        let (recommended_models, guidelines, recommended_params, expected_perf) = match use_case {
            "code-generation" => {
                let models = vec![
                    ModelRecommendation {
                        model_id: "claude-3-5-sonnet-latest".to_string(),
                        reason: "Excellent code understanding and generation capabilities".to_string(),
                        confidence_score: 9,
                        cost_range: CostRange {
                            min_cost_usd: 0.003,
                            max_cost_usd: 0.015,
                            typical_cost_usd: 0.008,
                        },
                        strengths: vec![
                            "Strong programming language support".to_string(),
                            "Good debugging assistance".to_string(),
                            "Comprehensive code explanations".to_string(),
                        ],
                        limitations: vec![
                            "May generate verbose explanations".to_string(),
                        ],
                    },
                    ModelRecommendation {
                        model_id: "claude-3-5-haiku-latest".to_string(),
                        reason: "Fast and cost-effective for simple code tasks".to_string(),
                        confidence_score: 7,
                        cost_range: CostRange {
                            min_cost_usd: 0.0003,
                            max_cost_usd: 0.0015,
                            typical_cost_usd: 0.0008,
                        },
                        strengths: vec![
                            "Very fast response times".to_string(),
                            "Cost-effective for bulk operations".to_string(),
                        ],
                        limitations: vec![
                            "Less sophisticated for complex problems".to_string(),
                        ],
                    },
                ];
                
                let guidelines = vec![
                    "Provide clear specifications and examples".to_string(),
                    "Request code comments for maintainability".to_string(),
                    "Ask for error handling and edge cases".to_string(),
                ];
                
                let params = RecommendedParameters {
                    temperature_range: (0.0, 0.3),
                    max_tokens_range: (1024, 4096),
                    top_p_range: Some((0.1, 0.5)),
                    use_streaming: Some(true),
                    system_message_patterns: vec![
                        "You are an expert programmer. Provide clean, well-documented code.".to_string(),
                    ],
                };
                
                let perf = PerformanceExpectations {
                    response_time_range_ms: (1000, 5000),
                    cost_range: CostRange {
                        min_cost_usd: 0.0003,
                        max_cost_usd: 0.015,
                        typical_cost_usd: 0.008,
                    },
                    quality_level: QualityLevel::Excellent,
                    success_rate_percentage: 90.0,
                };
                
                (models, guidelines, params, perf)
            },
            
            "creative-writing" => {
                let models = vec![
                    ModelRecommendation {
                        model_id: "claude-3-opus-latest".to_string(),
                        reason: "Highest quality creative output with nuanced understanding".to_string(),
                        confidence_score: 10,
                        cost_range: CostRange {
                            min_cost_usd: 0.015,
                            max_cost_usd: 0.075,
                            typical_cost_usd: 0.035,
                        },
                        strengths: vec![
                            "Exceptional creativity and originality".to_string(),
                            "Strong narrative structure".to_string(),
                            "Rich character development".to_string(),
                        ],
                        limitations: vec![
                            "Higher cost per token".to_string(),
                            "Slower response times".to_string(),
                        ],
                    },
                ];
                
                let guidelines = vec![
                    "Use higher temperature for more creativity".to_string(),
                    "Provide detailed prompts for better context".to_string(),
                    "Consider iterative refinement".to_string(),
                ];
                
                let params = RecommendedParameters {
                    temperature_range: (0.7, 1.0),
                    max_tokens_range: (2048, 8192),
                    top_p_range: Some((0.8, 0.95)),
                    use_streaming: Some(true),
                    system_message_patterns: vec![
                        "You are a creative writer with expertise in storytelling and narrative structure.".to_string(),
                    ],
                };
                
                let perf = PerformanceExpectations {
                    response_time_range_ms: (2000, 8000),
                    cost_range: CostRange {
                        min_cost_usd: 0.015,
                        max_cost_usd: 0.075,
                        typical_cost_usd: 0.035,
                    },
                    quality_level: QualityLevel::Excellent,
                    success_rate_percentage: 95.0,
                };
                
                (models, guidelines, params, perf)
            },
            
            _ => {
                // Default recommendations
                let models = vec![
                    ModelRecommendation {
                        model_id: "claude-3-5-sonnet-latest".to_string(),
                        reason: "Well-balanced model suitable for most tasks".to_string(),
                        confidence_score: 8,
                        cost_range: CostRange {
                            min_cost_usd: 0.003,
                            max_cost_usd: 0.015,
                            typical_cost_usd: 0.008,
                        },
                        strengths: vec![
                            "Good balance of speed, quality, and cost".to_string(),
                            "Wide range of capabilities".to_string(),
                        ],
                        limitations: vec![
                            "May not be optimal for specialized tasks".to_string(),
                        ],
                    },
                ];
                
                let guidelines = vec![
                    "Start with moderate temperature settings".to_string(),
                    "Adjust parameters based on specific needs".to_string(),
                ];
                
                let params = RecommendedParameters {
                    temperature_range: (0.3, 0.7),
                    max_tokens_range: (1024, 4096),
                    top_p_range: Some((0.5, 0.9)),
                    use_streaming: Some(false),
                    system_message_patterns: vec![
                        "You are a helpful AI assistant.".to_string(),
                    ],
                };
                
                let perf = PerformanceExpectations {
                    response_time_range_ms: (1000, 4000),
                    cost_range: CostRange {
                        min_cost_usd: 0.003,
                        max_cost_usd: 0.015,
                        typical_cost_usd: 0.008,
                    },
                    quality_level: QualityLevel::Good,
                    success_rate_percentage: 85.0,
                };
                
                (models, guidelines, params, perf)
            }
        };
        
        let pitfalls = vec![
            "Using inappropriate temperature settings".to_string(),
            "Not providing sufficient context".to_string(),
            "Ignoring token limits and costs".to_string(),
        ];
        
        ModelUsageRecommendations {
            use_case: use_case.to_string(),
            recommended_models,
            guidelines,
            recommended_parameters: recommended_params,
            pitfalls,
            expected_performance: expected_perf,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_derive_capabilities() {
        let client = Anthropic::new("test_key".to_string()).unwrap();
        let models_resource = ModelsResource::new(&client);
        
        let model = ModelObject {
            id: "claude-3-5-sonnet-latest".to_string(),
            display_name: "Claude 3.5 Sonnet".to_string(),
            created_at: Utc::now(),
            object_type: "model".to_string(),
        };
        
        let capabilities = models_resource.derive_capabilities(&model);
        
        assert_eq!(capabilities.max_context_length, 200_000);
        assert!(capabilities.supports_vision);
        assert!(capabilities.supports_tools);
        assert!(capabilities.capabilities.contains(&ModelCapability::Vision));
        assert!(capabilities.capabilities.contains(&ModelCapability::ToolUse));
    }
    
    #[test]
    fn test_derive_pricing() {
        let client = Anthropic::new("test_key".to_string()).unwrap();
        let models_resource = ModelsResource::new(&client);
        
        let model = ModelObject {
            id: "claude-3-5-haiku-latest".to_string(),
            display_name: "Claude 3.5 Haiku".to_string(),
            created_at: Utc::now(),
            object_type: "model".to_string(),
        };
        
        let pricing = models_resource.derive_pricing(&model);
        
        assert_eq!(pricing.input_price_per_million, 0.25);
        assert_eq!(pricing.output_price_per_million, 1.25);
        assert_eq!(pricing.tier, PricingTier::Fast);
    }
    
    #[test]
    fn test_derive_performance() {
        let client = Anthropic::new("test_key".to_string()).unwrap();
        let models_resource = ModelsResource::new(&client);
        
        let model = ModelObject {
            id: "claude-3-5-haiku-latest".to_string(),
            display_name: "Claude 3.5 Haiku".to_string(),
            created_at: Utc::now(),
            object_type: "model".to_string(),
        };
        
        let performance = models_resource.derive_performance(&model);
        
        assert_eq!(performance.speed_score, 10); // Haiku should be fastest
        assert_eq!(performance.cost_efficiency_score, 10); // And most cost-effective
        assert!(performance.tokens_per_second.unwrap() > 50.0);
    }
    
    #[test]
    fn test_score_model_with_requirements() {
        let client = Anthropic::new("test_key".to_string()).unwrap();
        let models_resource = ModelsResource::new(&client);
        
        let model = ModelObject {
            id: "claude-3-5-sonnet-latest".to_string(),
            display_name: "Claude 3.5 Sonnet".to_string(),
            created_at: Utc::now(),
            object_type: "model".to_string(),
        };
        
        let capabilities = models_resource.derive_capabilities(&model);
        let pricing = models_resource.derive_pricing(&model);
        let performance = models_resource.derive_performance(&model);
        
        let requirements = ModelRequirements::new()
            .require_vision()
            .min_context_length(50000);
        
        let score = models_resource.score_model(&model, &capabilities, &pricing, &performance, &requirements);
        
        assert!(score.is_some());
        assert!(score.unwrap() > 0.0);
    }
    
    #[test]
    fn test_score_model_elimination() {
        let client = Anthropic::new("test_key".to_string()).unwrap();
        let models_resource = ModelsResource::new(&client);
        
        let model = ModelObject {
            id: "claude-3-haiku-20240307".to_string(),
            display_name: "Claude 3 Haiku".to_string(),
            created_at: Utc::now(),
            object_type: "model".to_string(),
        };
        
        let capabilities = models_resource.derive_capabilities(&model);
        let pricing = models_resource.derive_pricing(&model);
        let performance = models_resource.derive_performance(&model);
        
        // Requirements that eliminate this model
        let requirements = ModelRequirements::new()
            .max_input_cost_per_token(0.0000001); // Extremely low cost that will eliminate any model
        
        let score = models_resource.score_model(&model, &capabilities, &pricing, &performance, &requirements);
        
        assert!(score.is_none()); // Should be eliminated
    }
} 