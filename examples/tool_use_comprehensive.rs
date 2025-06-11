//! Comprehensive tool use example demonstrating the Anthropic Rust SDK tool capabilities.
//!
//! This example shows how to:
//! - Define and register multiple tools
//! - Handle tool execution with retry logic
//! - Use parallel tool execution
//! - Handle tool validation and errors
//! - Create reusable tool functions

use anthropic_sdk::{
    Anthropic, Tool, ToolRegistry, ToolExecutor, ToolFunction, ToolChoice,
    ToolResult, ToolExecutionConfig, tool_function,
    Result as AnthropicResult,
};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;

/// Example weather tool implementation
struct WeatherTool;

#[async_trait]
impl ToolFunction for WeatherTool {
    async fn execute(&self, input: Value) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let location = input["location"].as_str().unwrap_or("Unknown Location");
        let unit = input["unit"].as_str().unwrap_or("fahrenheit");
        
        // Simulate API call delay
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let temp = match unit {
            "celsius" => "22°C",
            "fahrenheit" => "72°F",
            _ => "72°F",
        };
        
        let weather_data = json!({
            "location": location,
            "temperature": temp,
            "condition": "Sunny",
            "humidity": "45%",
            "wind": "5 mph"
        });
        
        Ok(ToolResult::success_json("weather_result", weather_data))
    }
    
    fn validate_input(&self, input: &Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(location) = input.get("location") {
            if location.as_str().map_or(true, |s| s.trim().is_empty()) {
                return Err("Location cannot be empty".into());
            }
        }
        Ok(())
    }
    
    fn timeout_seconds(&self) -> u64 {
        10 // Custom timeout for weather API
    }
}

/// Example calculator tool using the simple function wrapper
async fn calculate_tool(input: Value) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
    let a = input["a"].as_f64().ok_or("Missing parameter 'a'")?;
    let b = input["b"].as_f64().ok_or("Missing parameter 'b'")?;
    let operation = input["operation"].as_str().ok_or("Missing parameter 'operation'")?;
    
    let result = match operation {
        "add" => a + b,
        "subtract" => a - b,
        "multiply" => a * b,
        "divide" => {
            if b == 0.0 {
                return Ok(ToolResult::error("calc_error", "Division by zero"));
            }
            a / b
        }
        _ => return Ok(ToolResult::error("calc_error", "Unknown operation")),
    };
    
    Ok(ToolResult::success("calc_result", format!("{} {} {} = {}", a, operation, b, result)))
}

/// Example time tool using the macro
fn create_time_tool() -> Box<dyn ToolFunction> {
    Box::new(tool_function!(|_input: Value| async move {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let datetime = chrono::DateTime::from_timestamp(now as i64, 0)
            .unwrap_or_default();
        
        Ok(ToolResult::success("time_result", format!("Current time: {}", datetime.format("%Y-%m-%d %H:%M:%S UTC"))))
    }))
}

#[tokio::main]
async fn main() -> AnthropicResult<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    println!("🔧 Anthropic Rust SDK - Comprehensive Tool Use Example");
    println!("=======================================================\n");
    
    // 1. Create tool registry and register tools
    println!("1. Setting up tool registry...");
    let mut registry = ToolRegistry::new();
    
    // Weather tool with validation
    let weather_tool_def = Tool::new("get_weather", "Get current weather information for a location")
        .parameter("location", "string", "The city and state, e.g. 'San Francisco, CA'")
        .parameter("unit", "string", "Temperature unit ('celsius' or 'fahrenheit')")
        .required("location")
        .build();
    
    registry.register("get_weather", weather_tool_def, Box::new(WeatherTool))?;
    
    // Calculator tool using simple function
    let calc_tool_def = Tool::new("calculate", "Perform mathematical calculations")
        .parameter("a", "number", "First number")
        .parameter("b", "number", "Second number")
        .parameter("operation", "string", "Operation: add, subtract, multiply, divide")
        .required("a")
        .required("b")
        .required("operation")
        .build();
    
    registry.register("calculate", calc_tool_def, Box::new(anthropic_sdk::tools::SimpleTool::new(
        |input| Box::pin(calculate_tool(input))
    )))?;
    
    // Time tool using macro
    let time_tool_def = Tool::new("get_time", "Get the current time")
        .build();
    
    registry.register("get_time", time_tool_def, create_time_tool())?;
    
    println!("   ✅ Registered {} tools", registry.len());
    
    // 2. Create executor with advanced configuration
    println!("\n2. Configuring tool executor...");
    let config = ToolExecutionConfig {
        max_retries: 3,
        retry_delay: Duration::from_millis(100),
        exponential_backoff: true,
        max_retry_delay: Duration::from_secs(5),
        parallel_execution: true,
        max_concurrent_tools: 4,
    };
    
    let executor = ToolExecutor::with_config(Arc::new(registry), config);
    println!("   ✅ Executor configured with retry logic and parallel execution");
    
    // 3. Execute individual tools
    println!("\n3. Testing individual tool execution...");
    
    // Weather tool
    let weather_use = anthropic_sdk::ToolUse {
        id: "weather_001".to_string(),
        name: "get_weather".to_string(),
        input: json!({
            "location": "San Francisco, CA",
            "unit": "celsius"
        }),
    };
    
    match executor.execute_with_retry(&weather_use).await {
        Ok(result) => {
            println!("   🌤️  Weather result: {:?}", result.content);
        }
        Err(e) => {
            println!("   ❌ Weather tool failed: {}", e);
        }
    }
    
    // Calculator tool
    let calc_use = anthropic_sdk::ToolUse {
        id: "calc_001".to_string(),
        name: "calculate".to_string(),
        input: json!({
            "a": 15,
            "b": 7,
            "operation": "multiply"
        }),
    };
    
    match executor.execute_with_retry(&calc_use).await {
        Ok(result) => {
            println!("   🧮 Calculator result: {:?}", result.content);
        }
        Err(e) => {
            println!("   ❌ Calculator tool failed: {}", e);
        }
    }
    
    // 4. Test parallel execution
    println!("\n4. Testing parallel tool execution...");
    let tool_uses = vec![
        anthropic_sdk::ToolUse {
            id: "parallel_1".to_string(),
            name: "get_time".to_string(),
            input: json!({}),
        },
        anthropic_sdk::ToolUse {
            id: "parallel_2".to_string(),
            name: "calculate".to_string(),
            input: json!({"a": 10, "b": 5, "operation": "add"}),
        },
        anthropic_sdk::ToolUse {
            id: "parallel_3".to_string(),
            name: "get_weather".to_string(),
            input: json!({"location": "New York, NY", "unit": "fahrenheit"}),
        },
    ];
    
    let start_time = std::time::Instant::now();
    let results = executor.execute_multiple(&tool_uses).await;
    let duration = start_time.elapsed();
    
    println!("   ⚡ Executed {} tools in parallel in {:?}", results.len(), duration);
    
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(tool_result) => {
                println!("     {}. ✅ {}: {:?}", i + 1, tool_result.tool_use_id, tool_result.content);
            }
            Err(e) => {
                println!("     {}. ❌ Error: {}", i + 1, e);
            }
        }
    }
    
    // 5. Test error handling and validation
    println!("\n5. Testing error handling...");
    
    // Invalid input - missing required field
    let invalid_use = anthropic_sdk::ToolUse {
        id: "invalid_001".to_string(),
        name: "get_weather".to_string(),
        input: json!({"unit": "celsius"}), // Missing required "location"
    };
    
    match executor.execute_with_retry(&invalid_use).await {
        Ok(result) => {
            if result.is_error == Some(true) {
                println!("   ✅ Validation error caught: {:?}", result.content);
            }
        }
        Err(e) => {
            println!("   ✅ Execution error caught: {}", e);
        }
    }
    
    // Invalid tool name
    let unknown_use = anthropic_sdk::ToolUse {
        id: "unknown_001".to_string(),
        name: "unknown_tool".to_string(),
        input: json!({}),
    };
    
    match executor.execute_with_retry(&unknown_use).await {
        Err(e) => {
            println!("   ✅ Unknown tool error caught: {}", e);
        }
        Ok(_) => {
            println!("   ❓ Unexpected success for unknown tool");
        }
    }
    
    // 6. Show tool information
    println!("\n6. Tool registry information:");
    println!("   📊 Total tools: {}", executor.registry().len());
    println!("   🏷️  Tool names: {:?}", executor.registry().tool_names());
    
    let definitions = executor.registry().get_tool_definitions();
    for tool in &definitions {
        println!("   🔧 {}: {}", tool.name, tool.description);
        println!("      Parameters: {} (Required: {:?})", 
                 tool.input_schema.properties.len(), 
                 tool.input_schema.required);
    }
    
    println!("\n✨ Tool use demonstration complete!");
    println!("   • Tool registration and validation ✅");
    println!("   • Individual tool execution ✅");
    println!("   • Parallel tool execution ✅");
    println!("   • Error handling and recovery ✅");
    println!("   • Production-ready configuration ✅");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_weather_tool() {
        let tool = WeatherTool;
        let input = json!({"location": "Test City", "unit": "celsius"});
        
        let result = tool.execute(input).await.unwrap();
        assert_eq!(result.tool_use_id, "weather_result");
        assert!(result.is_error.is_none());
    }
    
    #[tokio::test]
    async fn test_calculator_tool() {
        let input = json!({"a": 10, "b": 5, "operation": "add"});
        let result = calculate_tool(input).await.unwrap();
        
        if let anthropic_sdk::ToolResultContent::Text(content) = result.content {
            assert!(content.contains("15"));
        } else {
            panic!("Expected text content");
        }
    }
} 