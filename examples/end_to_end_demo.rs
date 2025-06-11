use anthropic_sdk::{
    Tool, ToolRegistry, ToolExecutor, ToolFunction,
    File, FileConstraints, to_file,
    TokenCounter, RetryPolicy, RetryCondition, RetryExecutor, RetryResult,
    AnthropicError,
    types::{MessageContent, ContentBlockParam},
};
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use std::sync::Arc;
use async_trait::async_trait;

/// Simple text analyzer tool for demonstration
struct SimpleAnalyzerTool;

#[async_trait]
impl ToolFunction for SimpleAnalyzerTool {
    async fn execute(&self, parameters: Value) -> Result<anthropic_sdk::ToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let text = parameters.get("text")
            .and_then(|v| v.as_str())
            .ok_or("Missing text parameter")?;
        
        // Simulate processing
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        let word_count = text.split_whitespace().count();
        let char_count = text.len();
        
        let result = json!({
            "analysis": {
                "word_count": word_count,
                "character_count": char_count,
                "summary": format!("Text contains {} words and {} characters", word_count, char_count)
            }
        });
        
        Ok(anthropic_sdk::ToolResult {
            content: vec![anthropic_sdk::types::ContentBlock::Text {
                text: result.to_string(),
            }],
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 End-to-End Phase 4 Demo");
    println!("===========================\n");
    println!("This demo showcases all Phase 4 advanced features working together:");
    println!("• 🔧 Tool Use (Phase 4.1-4.2)");
    println!("• 📁 File Upload System (Phase 4.3)"); 
    println!("• 📊 Token Counting & 🔄 Retry Logic (Phase 4.4)");
    println!();

    // Phase 4.4: Initialize infrastructure components
    println!("🏗️ Phase 4.4: Initializing Infrastructure");
    println!("==========================================");
    
    let token_counter = TokenCounter::new();
    let retry_policy = RetryPolicy::exponential()
        .max_retries(3)
        .initial_delay(Duration::from_millis(100))
        .max_delay(Duration::from_secs(10))
        .retry_conditions(vec![
            RetryCondition::Timeout,
            RetryCondition::ServerError,
            RetryCondition::RateLimit,
        ]);
    
    let retry_executor = RetryExecutor::new(retry_policy);
    
    println!("✅ Token counter initialized");
    println!("✅ Retry policy configured: {} max retries", retry_executor.get_policy().max_retries);
    
    // Phase 4.1-4.2: Setup tool system
    println!("\n🔧 Phase 4.1-4.2: Setting Up Tool System");
    println!("=========================================");
    
    let mut registry = ToolRegistry::new();
    
    let analyzer_tool = Tool::new("analyze_text", "Analyze text content for basic metrics")
        .parameter("text", "string", "Text content to analyze")
        .required("text")
        .build();
    
    registry.register("analyze_text", analyzer_tool, Box::new(SimpleAnalyzerTool))?;
    let tool_executor = ToolExecutor::new(Arc::new(registry));
    
    println!("✅ Tool registry created");
    println!("✅ Text analyzer tool registered");
    println!("✅ Tool executor initialized");
    
    // Phase 4.3: File processing demonstration
    println!("\n📁 Phase 4.3: File Processing Demonstration");
    println!("===========================================");
    
    let sample_documents = vec![
        ("report.txt", "This is a comprehensive business report analyzing market trends and performance metrics for Q4."),
        ("analysis.csv", "name,score,category\nProduct A,85,Electronics\nProduct B,92,Software\nProduct C,78,Hardware"),
        ("data.json", r#"{"metrics": {"users": 1500, "sessions": 3200, "satisfaction": 4.2}}"#),
    ];
    
    let mut processed_files = Vec::new();
    
    for (filename, content) in sample_documents {
        let start_time = Instant::now();
        
        // Create file with appropriate MIME type
        let mime_type = match filename.split('.').last() {
            Some("txt") => "text/plain",
            Some("csv") => "text/csv", 
            Some("json") => "application/json",
            _ => "text/plain",
        };
        
        let file = File::from_bytes(
            content.as_bytes().to_vec(),
            filename,
            mime_type
        )?;
        
        // Validate file with constraints
        let constraints = FileConstraints::new()
            .max_size(1024 * 1024) // 1MB
            .allowed_types(vec![
                "text/plain".to_string(),
                "text/csv".to_string(),
                "application/json".to_string(),
            ]);
        
        constraints.validate(&file)?;
        
        let processing_time = start_time.elapsed();
        
        println!("📄 Processed: {} ({} bytes, {}) in {:?}", 
            file.name(), file.size(), file.mime_type(), processing_time);
        
        processed_files.push(file);
    }
    
    println!("✅ {} files processed and validated", processed_files.len());
    
    // Phase 4.1-4.2: Tool execution with retry logic
    println!("\n🔧 Phase 4.1-4.2: Tool Execution with Retry Logic");
    println!("=================================================");
    
    for file in &processed_files {
        let file_content = String::from_utf8_lossy(&file.to_bytes()?);
        let analysis_request = vec![json!({
            "name": "analyze_text",
            "input": { "text": file_content }
        })];
        
        // Execute tool with retry logic
        let result = retry_executor.execute(|| {
            let request = analysis_request.clone();
            async move {
                tool_executor.execute_multiple(&request).await
                    .map_err(|e| AnthropicError::Other(e.to_string()))
            }
        }).await;
        
        match result {
            RetryResult::Success(results) => {
                if let Some(first_result) = results.first() {
                    println!("✅ Analysis for {}: Tool executed successfully", file.name());
                    if let Some(content) = first_result.get("success") {
                        if content.as_bool().unwrap_or(false) {
                            println!("   📊 Tool execution completed");
                        }
                    }
                }
            }
            RetryResult::Failed(error) => {
                println!("❌ Analysis for {}: {}", file.name(), error);
            }
        }
    }
    
    // Phase 4.3: File upload integration
    println!("\n📁 Phase 4.3: File Upload Integration");
    println!("=====================================");
    
    // Demonstrate file-to-message integration
    let sample_image_data = vec![0u8; 100]; // Simulated image data
    let image_file = to_file(&sample_image_data, "sample.png", "image/png")?;
    
    let message_content = MessageContent::Blocks(vec![
        ContentBlockParam::text("Please analyze this image:"),
        ContentBlockParam::image_file(image_file.clone()),
        ContentBlockParam::text("What can you tell me about it?"),
    ]);
    
    println!("✅ Created multi-part message with file attachment");
    println!("📄 Image file: {} ({} bytes)", image_file.name(), image_file.size());
    
    if let MessageContent::Blocks(blocks) = &message_content {
        println!("📝 Message has {} content blocks", blocks.len());
    }
    
    // Phase 4.4: Cost estimation and monitoring
    println!("\n📊 Phase 4.4: Cost Estimation & Monitoring");
    println!("===========================================");
    
    // Estimate costs for hypothetical API calls
    let estimated_cost = token_counter.estimate_cost("claude-3-5-sonnet-latest", 500, 200);
    println!("💰 Estimated cost for 500 input + 200 output tokens: ${:.4}", estimated_cost);
    
    // Simulate some usage
    let usage = anthropic_sdk::types::Usage {
        input_tokens: 750,
        output_tokens: 300,
        cache_creation_input_tokens: None,
        cache_read_input_tokens: Some(50),
        server_tool_use: None,
        service_tier: None,
    };
    
    let cost_breakdown = token_counter.record_usage("claude-3-5-sonnet-latest", &usage);
    println!("📈 Recorded usage: {} input, {} output, {} cache read tokens", 
        usage.input_tokens, usage.output_tokens, usage.cache_read_input_tokens.unwrap_or(0));
    println!("💵 Cost breakdown: ${:.4} total", cost_breakdown.total_cost);
    
    // Final metrics and summary
    println!("\n📈 Final Metrics & Summary");
    println!("==========================");
    
    let usage_summary = token_counter.get_summary();
    println!("Session Statistics:");
    println!("  • Total tokens: {}", usage_summary.total_tokens);
    println!("  • Total cost: ${:.4}", usage_summary.total_cost_usd);
    println!("  • Session duration: {:.1} seconds", usage_summary.session_duration.as_secs_f64());
    
    println!("\nInfrastructure Status:");
    println!("  • Files processed: {}", processed_files.len());
    println!("  • Tools available: 1 (text analyzer)");
    println!("  • Retry policy: {} max retries configured", retry_executor.get_policy().max_retries);
    println!("  • File constraints: Size limits and type validation active");
    
    println!("\n✨ End-to-End Demo Complete!");
    println!("🎉 Phase 4 Advanced Features Successfully Demonstrated:");
    println!();
    println!("🔧 **Tool Use System (Phase 4.1-4.2)**");
    println!("   ✅ Tool registry with type-safe registration");
    println!("   ✅ Tool execution with async processing");
    println!("   ✅ Error handling and result processing");
    println!();
    println!("📁 **File Upload System (Phase 4.3)**");
    println!("   ✅ Multi-format file creation (txt, csv, json, png)");
    println!("   ✅ File validation with size and type constraints");
    println!("   ✅ MIME type detection and processing");
    println!("   ✅ Message integration with file attachments");
    println!();
    println!("🏗️ **Enhanced Infrastructure (Phase 4.4)**");
    println!("   ✅ Token counting with real-time cost estimation");
    println!("   ✅ Retry logic with exponential backoff policies");
    println!("   ✅ Usage monitoring and session analytics");
    println!("   ✅ Production-ready error handling");
    println!();
    println!("🚀 **All systems operational and ready for production use!**");
    
    Ok(())
} 