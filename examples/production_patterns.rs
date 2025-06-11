use anthropic_sdk::{
    Anthropic, MessageCreateBuilder,
    Tool, ToolRegistry, ToolExecutor, ToolFunction,
    File, FileConstraints, to_file,
    TokenCounter, RetryPolicy, RetryCondition, RetryExecutor, RetryResult,
    types::{ToolChoice, MessageContent, ContentBlockParam, Message},
    AnthropicError,
};
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use std::sync::Arc;
use async_trait::async_trait;
use tracing::{info, warn, error, debug};

/// Production-ready document analysis service
pub struct DocumentAnalysisService {
    anthropic_client: Anthropic,
    tool_executor: ToolExecutor,
    token_counter: Arc<TokenCounter>,
    retry_executor: Arc<RetryExecutor>,
}

/// Configuration for the document analysis service
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub max_file_size: usize,
    pub allowed_file_types: Vec<String>,
    pub max_retries: u32,
    pub timeout: Duration,
    pub enable_detailed_logging: bool,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB
            allowed_file_types: vec![
                "text/plain".to_string(),
                "text/csv".to_string(),
                "application/json".to_string(),
                "image/png".to_string(),
                "image/jpeg".to_string(),
                "application/pdf".to_string(),
            ],
            max_retries: 3,
            timeout: Duration::from_secs(30),
            enable_detailed_logging: true,
        }
    }
}

/// Data extraction tool for analyzing various file formats
struct DataExtractionTool;

#[async_trait]
impl ToolFunction for DataExtractionTool {
    async fn call(&self, parameters: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let file_type = parameters.get("file_type")
            .and_then(|v| v.as_str())
            .ok_or("Missing file_type parameter")?;
        
        let content = parameters.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        debug!("Extracting data from {} file", file_type);
        
        // Simulate processing time based on file type
        let processing_time = match file_type {
            "image/png" | "image/jpeg" => Duration::from_millis(500),
            "application/pdf" => Duration::from_millis(1000),
            _ => Duration::from_millis(100),
        };
        
        tokio::time::sleep(processing_time).await;
        
        // Simulate data extraction based on file type
        let extracted_data = match file_type {
            "text/csv" => {
                json!({
                    "type": "structured_data",
                    "rows_detected": content.lines().count(),
                    "columns_detected": content.lines().next()
                        .map(|line| line.split(',').count())
                        .unwrap_or(0),
                    "summary": "CSV file with tabular data detected"
                })
            },
            "application/json" => {
                json!({
                    "type": "json_data",
                    "structure": "object",
                    "fields_detected": serde_json::from_str::<Value>(content)
                        .map(|v| v.as_object().map(|o| o.len()).unwrap_or(0))
                        .unwrap_or(0),
                    "summary": "JSON file with structured data"
                })
            },
            "image/png" | "image/jpeg" => {
                json!({
                    "type": "image_analysis",
                    "format": file_type,
                    "estimated_elements": 5,
                    "summary": "Image file suitable for visual analysis"
                })
            },
            "application/pdf" => {
                json!({
                    "type": "document_analysis",
                    "estimated_pages": 1,
                    "text_detected": true,
                    "summary": "PDF document with extractable content"
                })
            },
            _ => {
                json!({
                    "type": "text_analysis",
                    "word_count": content.split_whitespace().count(),
                    "character_count": content.len(),
                    "summary": "Text file with readable content"
                })
            }
        };
        
        Ok(extracted_data)
    }
}

/// Sentiment analysis tool
struct SentimentAnalysisTool;

#[async_trait]
impl ToolFunction for SentimentAnalysisTool {
    async fn call(&self, parameters: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let text = parameters.get("text")
            .and_then(|v| v.as_str())
            .ok_or("Missing text parameter")?;
        
        debug!("Analyzing sentiment for {} characters", text.len());
        
        // Simulate analysis time
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Simple sentiment analysis simulation
        let positive_words = ["good", "great", "excellent", "amazing", "wonderful", "fantastic"];
        let negative_words = ["bad", "terrible", "awful", "horrible", "poor", "disappointing"];
        
        let text_lower = text.to_lowercase();
        let positive_count = positive_words.iter()
            .map(|word| text_lower.matches(word).count())
            .sum::<usize>();
        let negative_count = negative_words.iter()
            .map(|word| text_lower.matches(word).count())
            .sum::<usize>();
        
        let sentiment = if positive_count > negative_count {
            "positive"
        } else if negative_count > positive_count {
            "negative"
        } else {
            "neutral"
        };
        
        let confidence = ((positive_count + negative_count) as f64 / text.split_whitespace().count() as f64 * 100.0)
            .min(95.0)
            .max(10.0);
        
        Ok(json!({
            "sentiment": sentiment,
            "confidence": confidence,
            "positive_indicators": positive_count,
            "negative_indicators": negative_count,
            "analysis_summary": format!("Text sentiment: {} ({}% confidence)", sentiment, confidence as u32)
        }))
    }
}

/// Production-ready text analysis tool
struct TextAnalysisTool;

#[async_trait]
impl ToolFunction for TextAnalysisTool {
    async fn call(&self, parameters: Value) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let text = parameters.get("text")
            .and_then(|v| v.as_str())
            .ok_or("Missing text parameter")?;
        
        // Simulate analysis processing
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let word_count = text.split_whitespace().count();
        let char_count = text.len();
        let reading_time = (word_count as f64 / 200.0).ceil() as u32;
        
        Ok(json!({
            "word_count": word_count,
            "character_count": char_count,
            "estimated_reading_time_minutes": reading_time,
            "summary": format!("Analysis: {} words, {} chars, ~{}min read", word_count, char_count, reading_time)
        }))
    }
}

impl DocumentAnalysisService {
    /// Create a new document analysis service with production configuration
    pub async fn new(config: ServiceConfig) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing Document Analysis Service with config: {:?}", config);
        
        // Initialize Anthropic client
        let anthropic_client = Anthropic::from_env()?;
        
        // Setup token counter for cost tracking
        let token_counter = Arc::new(TokenCounter::new());
        
        // Configure retry policy for production resilience
        let retry_policy = RetryPolicy::exponential()
            .max_retries(config.max_retries)
            .initial_delay(Duration::from_millis(500))
            .max_delay(Duration::from_secs(30))
            .multiplier(2.0)
            .jitter(true)
            .max_elapsed_time(Some(config.timeout))
            .retry_conditions(vec![
                RetryCondition::RateLimit,
                RetryCondition::ServerError,
                RetryCondition::Timeout,
                RetryCondition::ConnectionError,
            ]);
        
        let retry_executor = Arc::new(RetryExecutor::new(retry_policy));
        
        // Setup tool registry with production tools
        let mut registry = ToolRegistry::new();
        
        // Register data extraction tool
        let data_tool = Tool::new("extract_data", "Extract structured data from various file formats")
            .parameter("file_type", "string", "MIME type of the file to analyze")
            .parameter("content", "string", "File content to analyze (optional)")
            .required("file_type")
            .build();
        registry.register_tool(data_tool, Box::new(DataExtractionTool))?;
        
        // Register sentiment analysis tool
        let sentiment_tool = Tool::new("analyze_sentiment", "Analyze sentiment of text content")
            .parameter("text", "string", "Text content to analyze for sentiment")
            .required("text")
            .build();
        registry.register_tool(sentiment_tool, Box::new(SentimentAnalysisTool))?;
        
        let tool_executor = ToolExecutor::new(registry);
        
        info!("Service initialized with {} tools", tool_executor.registry().list_tools().len());
        
        Ok(Self {
            anthropic_client,
            tool_executor,
            token_counter,
            retry_executor,
        })
    }
    
    /// Process a document with comprehensive error handling and monitoring
    pub async fn process_document(
        &self,
        file_data: &[u8],
        filename: &str,
        mime_type: &str,
    ) -> Result<DocumentAnalysisResult, DocumentAnalysisError> {
        let start_time = Instant::now();
        let request_id = uuid::Uuid::new_v4().to_string();
        
        info!("Processing document: {} ({})", filename, mime_type);
        debug!("Request ID: {}", request_id);
        
        // Step 1: Validate file
        let file = self.validate_and_create_file(file_data, filename, mime_type)?;
        info!("File validation successful: {} bytes", file.size());
        
        // Step 2: Extract data using tools with retry logic
        let extraction_result = self.extract_data_with_retry(&file).await?;
        
        // Step 3: Analyze with Claude if needed
        let claude_analysis = self.analyze_with_claude(&file, &extraction_result).await?;
        
        // Step 4: Calculate costs and metrics
        let processing_time = start_time.elapsed();
        let cost_breakdown = self.calculate_processing_cost(&claude_analysis);
        
        let result = DocumentAnalysisResult {
            request_id,
            filename: filename.to_string(),
            file_size: file.size(),
            mime_type: mime_type.to_string(),
            extraction_data: extraction_result,
            claude_analysis,
            processing_time,
            cost_breakdown,
        };
        
        info!("Document processing completed in {:?}", processing_time);
        Ok(result)
    }
    
    /// Validate file and create File object with constraints
    fn validate_and_create_file(
        &self,
        data: &[u8],
        filename: &str,
        mime_type: &str,
    ) -> Result<File, DocumentAnalysisError> {
        // Create file constraints for validation
        let constraints = FileConstraints::new()
            .max_size(10 * 1024 * 1024) // 10MB
            .allowed_types(vec![
                "text/plain".to_string(),
                "text/csv".to_string(),
                "application/json".to_string(),
                "image/png".to_string(),
                "image/jpeg".to_string(),
            ])
            .require_hash(true);
        
        // Create file
        let file = File::from_bytes(data.to_vec(), filename, mime_type)
            .map_err(DocumentAnalysisError::FileCreation)?;
        
        // Validate against constraints
        constraints.validate(&file)
            .map_err(DocumentAnalysisError::FileValidation)?;
        
        Ok(file)
    }
    
    /// Extract data using tools with retry logic
    async fn extract_data_with_retry(&self, file: &File) -> Result<Value, DocumentAnalysisError> {
        let extraction_request = json!({
            "id": format!("extract_{}", uuid::Uuid::new_v4()),
            "name": "extract_data",
            "input": {
                "file_type": file.mime_type(),
                "content": String::from_utf8_lossy(&file.to_bytes().unwrap_or_default())
            }
        });
        
        let retry_executor = Arc::clone(&self.retry_executor);
        let tool_executor = &self.tool_executor;
        
        let result = retry_executor.execute(|| {
            let request = extraction_request.clone();
            async move {
                tool_executor.execute_tool(&request).await
                    .map_err(|e| AnthropicError::Other(e.to_string()))
            }
        }).await;
        
        match result {
            RetryResult::Success(data) => Ok(data),
            RetryResult::Failed(error) => Err(DocumentAnalysisError::ToolExecution(error)),
        }
    }
    
    /// Analyze document with Claude using the comprehensive message system
    async fn analyze_with_claude(&self, file: &File, extraction_data: &Value) -> Result<Message, DocumentAnalysisError> {
        let start_time = Instant::now();
        
        // Build comprehensive message with file and extracted data
        let content = MessageContent::Blocks(vec![
            ContentBlockParam::text("Please analyze this document comprehensively:"),
            ContentBlockParam::from_file(file.clone())
                .map_err(DocumentAnalysisError::FileProcessing)?,
            ContentBlockParam::text(&format!(
                "Extracted data: {}",
                serde_json::to_string_pretty(extraction_data).unwrap_or_default()
            )),
            ContentBlockParam::text("Provide insights about the content, structure, and key findings."),
        ]);
        
        // Setup tools for Claude to use
        let tools = self.tool_executor.registry().get_all_tools();
        
        let retry_executor = Arc::clone(&self.retry_executor);
        let client = &self.anthropic_client;
        
        let result = retry_executor.execute(|| {
            let content = content.clone();
            let tools = tools.clone();
            async move {
                client.messages()
                    .create_with_builder("claude-3-5-sonnet-latest", 2048)
                    .message(anthropic_sdk::types::Role::User, content)
                    .tools(tools)
                    .tool_choice(ToolChoice::Auto)
                    .temperature(0.3)
                    .send()
                    .await
            }
        }).await;
        
        let message = match result {
            RetryResult::Success(msg) => msg,
            RetryResult::Failed(error) => return Err(DocumentAnalysisError::ClaudeAnalysis(error)),
        };
        
        let analysis_time = start_time.elapsed();
        info!("Claude analysis completed in {:?}", analysis_time);
        
        Ok(message)
    }
    
    /// Calculate processing costs
    fn calculate_processing_cost(&self, message: &Message) -> Option<anthropic_sdk::CostBreakdown> {
        if let Some(usage) = &message.usage {
            Some(self.token_counter.record_usage("claude-3-5-sonnet-latest", usage))
        } else {
            None
        }
    }
    
    /// Get service metrics and statistics
    pub fn get_metrics(&self) -> ServiceMetrics {
        let usage_summary = self.token_counter.get_summary();
        let retry_policy = self.retry_executor.get_policy();
        
        ServiceMetrics {
            total_requests: usage_summary.request_count,
            total_cost: usage_summary.total_cost_usd,
            average_cost_per_request: usage_summary.avg_cost_per_request,
            session_duration: usage_summary.session_duration,
            retry_policy_max_retries: retry_policy.max_retries,
            tools_available: self.tool_executor.registry().list_tools().len() as u32,
        }
    }
}

/// Result of document analysis
#[derive(Debug)]
pub struct DocumentAnalysisResult {
    pub request_id: String,
    pub filename: String,
    pub file_size: usize,
    pub mime_type: String,
    pub extraction_data: Value,
    pub claude_analysis: Message,
    pub processing_time: Duration,
    pub cost_breakdown: Option<anthropic_sdk::CostBreakdown>,
}

/// Service metrics for monitoring
#[derive(Debug)]
pub struct ServiceMetrics {
    pub total_requests: u32,
    pub total_cost: f64,
    pub average_cost_per_request: f64,
    pub session_duration: Duration,
    pub retry_policy_max_retries: u32,
    pub tools_available: u32,
}

/// Comprehensive error handling for the service
#[derive(Debug, thiserror::Error)]
pub enum DocumentAnalysisError {
    #[error("File creation failed: {0}")]
    FileCreation(#[from] anthropic_sdk::FileError),
    
    #[error("File validation failed: {0}")]
    FileValidation(anthropic_sdk::FileError),
    
    #[error("File processing failed: {0}")]
    FileProcessing(anthropic_sdk::FileError),
    
    #[error("Tool execution failed: {0}")]
    ToolExecution(AnthropicError),
    
    #[error("Claude analysis failed: {0}")]
    ClaudeAnalysis(AnthropicError),
    
    #[error("Service configuration error: {0}")]
    Configuration(String),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for production logging
    tracing_subscriber::fmt()
        .with_level(true)
        .with_target(true)
        .init();
    
    println!("🏭 Production Patterns Demo");
    println!("===========================\n");
    
    // Initialize service with production configuration
    let config = ServiceConfig {
        max_file_size: 5 * 1024 * 1024, // 5MB for demo
        allowed_file_types: vec![
            "text/plain".to_string(),
            "text/csv".to_string(),
            "application/json".to_string(),
        ],
        max_retries: 3,
        timeout: Duration::from_secs(30),
        enable_detailed_logging: true,
    };
    
    let service = DocumentAnalysisService::new(config).await?;
    info!("Document Analysis Service initialized");
    
    // Demo documents for processing
    let demo_documents = vec![
        ("sample.txt", "text/plain", "This is a sample document for analysis. It contains important information about our product performance and customer feedback. The overall sentiment appears positive with some areas for improvement."),
        ("data.csv", "text/csv", "name,score,feedback\nAlice,85,Great product\nBob,72,Good but could be better\nCarol,91,Excellent experience"),
        ("config.json", "application/json", r#"{"settings": {"theme": "dark", "notifications": true}, "users": 150, "active_sessions": 75}"#),
    ];
    
    println!("📊 Processing Demo Documents");
    println!("============================\n");
    
    let mut results = Vec::new();
    
    for (filename, mime_type, content) in demo_documents {
        println!("Processing: {}", filename);
        
        match service.process_document(content.as_bytes(), filename, mime_type).await {
            Ok(result) => {
                println!("✅ Success: {} processed in {:?}", 
                    result.filename, result.processing_time);
                
                if let Some(cost) = &result.cost_breakdown {
                    println!("   Cost: ${:.4}", cost.total_cost);
                }
                
                println!("   Extraction: {}", 
                    result.extraction_data.get("summary")
                        .and_then(|v| v.as_str())
                        .unwrap_or("No summary available")
                );
                
                results.push(result);
            }
            Err(e) => {
                error!("❌ Failed to process {}: {}", filename, e);
                println!("❌ Error processing {}: {}", filename, e);
            }
        }
        
        println!();
    }
    
    // Service metrics and monitoring
    println!("📈 Service Metrics");
    println!("==================");
    
    let metrics = service.get_metrics();
    println!("Total requests processed: {}", metrics.total_requests);
    println!("Total cost: ${:.4}", metrics.total_cost);
    println!("Average cost per request: ${:.4}", metrics.average_cost_per_request);
    println!("Session duration: {:.1} seconds", metrics.session_duration.as_secs_f64());
    println!("Retry policy max retries: {}", metrics.retry_policy_max_retries);
    println!("Tools available: {}", metrics.tools_available);
    
    // Performance analysis
    println!("\n⚡ Performance Analysis");
    println!("======================");
    
    let total_processing_time: Duration = results.iter()
        .map(|r| r.processing_time)
        .sum();
    let average_processing_time = total_processing_time / results.len() as u32;
    let total_file_size: usize = results.iter().map(|r| r.file_size).sum();
    
    println!("Documents processed: {}", results.len());
    println!("Total processing time: {:?}", total_processing_time);
    println!("Average processing time: {:?}", average_processing_time);
    println!("Total data processed: {} bytes", total_file_size);
    println!("Throughput: {:.2} KB/s", 
        (total_file_size as f64 / 1024.0) / total_processing_time.as_secs_f64());
    
    // Error resilience demonstration
    println!("\n🛡️ Error Resilience Features");
    println!("=============================");
    println!("✅ Comprehensive file validation with size and type constraints");
    println!("✅ Retry logic with exponential backoff for transient failures");
    println!("✅ Tool execution with timeout and error recovery");
    println!("✅ Cost tracking and usage monitoring");
    println!("✅ Structured logging with correlation IDs");
    println!("✅ Type-safe error handling with detailed error types");
    
    println!("\n✨ Production Patterns Demo Complete!");
    println!("🚀 This demonstrates enterprise-grade patterns:");
    println!("   • Comprehensive error handling and recovery");
    println!("   • Production-ready service architecture");
    println!("   • Cost tracking and performance monitoring");
    println!("   • Tool integration with retry logic");
    println!("   • File processing with validation");
    println!("   • Structured logging and observability");
    
    Ok(())
} 