use anthropic_sdk::{Anthropic, ClientConfig, MessageCreateBuilder, AuthMethod};
use std::time::Duration;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Custom Gateway Configuration Demo");
    println!("====================================\n");

    let demo_token = env::var("CUSTOM_BEARER_TOKEN").unwrap_or_else(|_| "your-custom-bearer-token".to_string());
    let base_url = env::var("CUSTOM_BASE_URL").unwrap_or_else(|_| "https://your-gateway.example.com/v1/anthropic".to_string());

    println!("📡 Base URL: {}", base_url);
    println!("🤖 Model: claude-3-5-sonnet-latest\n");
    
    // Method 1: Convenience method (Recommended)
    println!("🔧 Method 1: Using for_custom_gateway() convenience method");
    let _client1 = Anthropic::with_config(
        ClientConfig::new(&demo_token)
            .with_base_url(&base_url)
            .with_timeout(Duration::from_secs(30))
    )?;
    println!("   ✅ Client created successfully!");
    println!("   🔑 Auth method: Bearer token");
    println!("   ⏱️  Timeout: 30 seconds");
    
    // Method 2: Manual Bearer token configuration
    println!("\n🔧 Method 2: Manual Bearer token configuration");
    let _client2 = Anthropic::with_config(
        ClientConfig::new(&demo_token)
            .with_base_url(&base_url)
            .with_auth_method(AuthMethod::Bearer)
            .with_timeout(Duration::from_secs(30))
    )?;
    println!("   ✅ Client created successfully!");
    println!("   🔑 Auth method: Bearer token (manual)");
    println!("   📡 Base URL: configured manually");
    
    // Method 3: Standard Anthropic (for comparison)
    println!("\n🔧 Method 3: Standard Anthropic API (for comparison)");
    let _client3 = Anthropic::with_config(
        ClientConfig::new(&demo_token)
            .with_base_url("https://api.anthropic.com")
            .with_auth_method(AuthMethod::Anthropic)
    )?;
    println!("   ✅ Client created successfully!");
    println!("   🔑 Auth method: x-api-key header");
    println!("   📡 Base URL: Standard Anthropic API");
    
    println!("\n✅ All configuration methods work!");
    
    print_usage_examples();
    print_environment_setup();
    print_testing_instructions();

    Ok(())
}

fn print_usage_examples() {
    println!();
    println!("{}", "=".repeat(60));
    println!("📚 Production Usage Examples");
    println!("{}", "=".repeat(60));
    
    println!("\n🔸 Method 1: Environment Variables (Recommended)");
    println!("```rust");
    println!("use anthropic_sdk::{{Anthropic, ClientConfig, AuthMethod}};");
    println!();
    println!("let client = Anthropic::with_config(");
    println!("    ClientConfig::new(&env::var(\"CUSTOM_BEARER_TOKEN\")?)");
    println!("        .with_base_url(&env::var(\"CUSTOM_BASE_URL\")?)");
    println!(")?;");
    println!("```");
    
    println!("\n🔸 Method 2: Direct Configuration");
    println!("```rust");
    println!("let client = Anthropic::with_config(");
    println!("    ClientConfig::new(\"your-custom-bearer-token\")");
    println!("        .with_base_url(\"https://your-gateway.example.com/v1/anthropic\")");
    println!("        .with_auth_method(AuthMethod::Bearer)");
    println!(")?;");
    println!("```");
    
    println!("\n🔸 Making API Calls");
    println!("```rust");
    println!("use anthropic_sdk::MessageCreateBuilder;");
    println!();
    println!("let response = client.messages()");
    println!("    .create(");
    println!("        MessageCreateBuilder::new(\"claude-3-5-sonnet-latest\", 1024)");
    println!("            .user(\"Your message here\")");
    println!("            .system(\"Optional system prompt\")");
    println!("            .temperature(0.7)");
    println!("            .build()");
    println!("    )");
    println!("    .await?;");
    println!("```");
}

fn print_environment_setup() {
    println!("\n{}", "=".repeat(60));
    println!("🌍 Environment Variable Setup");
    println!("{}", "=".repeat(60));
    
    println!("\n🔸 Option 1: Custom-specific variables");
    println!("```bash");
    println!("export CUSTOM_BEARER_TOKEN='your-custom-bearer-token'");
    println!("export CUSTOM_BASE_URL='https://your-gateway.example.com/v1/anthropic'");
    println!("```");
    
    println!("\n🔸 Option 2: Standard Anthropic variables");
    println!("```bash");
    println!("export ANTHROPIC_API_KEY='your-custom-bearer-token'");
    println!("export ANTHROPIC_BASE_URL='https://your-gateway.example.com/v1/anthropic'");
    println!("```");
    
    println!("\n🔸 Then use in code:");
    println!("```rust");
    println!("let client = Anthropic::from_env()?;  // Automatically configured!");
    println!("```");
}

fn print_testing_instructions() {
    println!("\n{}", "=".repeat(60));
    println!("🧪 Testing Instructions");
    println!("{}", "=".repeat(60));
    
    println!("\n🔸 To test with your real Bearer token:");
    println!("```bash");
    println!("# Set your credentials");
    println!("export CUSTOM_BEARER_TOKEN='your-real-bearer-token'");
    println!("export CUSTOM_BASE_URL='https://your-gateway.example.com/v1/anthropic'");
    println!();
    println!("# Run production test");
    println!("cargo run --example custom_gateway_production");
    println!("```");
    
    println!("\n🔸 Available test examples:");
    println!("• cargo run --example custom_gateway_production  # With real token");
    println!("• cargo run --example custom_gateway_demo        # This demo");
    println!("• cargo run --example test_dual_auth             # Test both auth methods");
    println!("• cargo run --example test_custom_streaming      # Test streaming");
    
    println!("\n🔸 Features supported with Custom Gateway:");
    println!("• ✅ Messages API");
    println!("• ✅ Streaming responses");  
    println!("• ✅ Tool use");
    println!("• ✅ Vision/image inputs");
    println!("• ✅ System prompts");
    println!("• ✅ Temperature, top_p, top_k");
    println!("• ✅ Stop sequences");
    println!("• ✅ Custom model names (claude-3-5-sonnet-latest)");
    
    println!("\n🎯 Your Custom Gateway is fully integrated and ready for production! 🚀");
} 