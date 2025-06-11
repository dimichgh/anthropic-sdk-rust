#!/bin/bash

# Test Script: Using anthropic-sdk-rust from Git Repository
# This demonstrates how users can test your SDK directly from GitHub

set -e

echo "🦀 Testing anthropic-sdk-rust from Git Repository"
echo "================================================"

# Create a temporary test project
TEST_DIR="test-anthropic-sdk-git"
rm -rf "$TEST_DIR" 2>/dev/null || true
mkdir "$TEST_DIR"
cd "$TEST_DIR"

echo ""
echo "📦 Creating test project..."

# Create Cargo.toml with git dependency
cat > Cargo.toml << 'EOF'
[package]
name = "test-anthropic-sdk-git"
version = "0.1.0"
edition = "2021"

[dependencies]
# Using SDK directly from git repository
anthropic-sdk-rust = { git = "https://github.com/dimichgh/anthropic-sdk-rust", branch = "main" }
tokio = { version = "1.0", features = ["rt-multi-thread", "macros"] }
EOF

# Create src directory
mkdir src

# Create test main.rs
cat > src/main.rs << 'EOF'
use anthropic_sdk::{Anthropic, MessageCreateBuilder, Result};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🎉 Testing Anthropic Rust SDK from Git!");
    println!("=====================================");
    
    // Test basic client creation
    println!("\n📝 Testing client creation...");
    let client = match Anthropic::new("demo-api-key") {
        Ok(client) => {
            println!("✅ Client created successfully!");
            client
        }
        Err(e) => {
            println!("❌ Failed to create client: {}", e);
            return Err(e);
        }
    };
    
    // Test configuration access
    println!("\n⚙️  Testing client configuration...");
    let config = client.config();
    println!("✅ Base URL: {}", config.base_url);
    println!("✅ Timeout: {:?}", config.timeout);
    println!("✅ Max retries: {}", config.max_retries);
    
    // Test message builder (no actual API call)
    println!("\n🔨 Testing message builder...");
    let params = MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
        .user("Hello from git repository test!")
        .system("You are a helpful assistant.")
        .temperature(0.7)
        .build();
    
    println!("✅ Message parameters created:");
    println!("   Model: {}", params.model);
    println!("   Max tokens: {}", params.max_tokens);
    println!("   Messages: {} message(s)", params.messages.len());
    println!("   Temperature: {:?}", params.temperature);
    
    // Test API resource access
    println!("\n🌐 Testing API resources...");
    let _messages = client.messages();
    println!("✅ Messages API accessible");
    
    let _files = client.files();
    println!("✅ Files API accessible");
    
    let _batches = client.batches();
    println!("✅ Batches API accessible");
    
    let _models = client.models();
    println!("✅ Models API accessible");
    
    println!("\n🎊 All tests passed! SDK is working from Git repository!");
    println!("\n📋 Next steps:");
    println!("   1. Set ANTHROPIC_API_KEY environment variable");
    println!("   2. Replace 'demo-api-key' with Anthropic::from_env()?");
    println!("   3. Make actual API calls");
    
    Ok(())
}
EOF

echo "✅ Test project created!"

echo ""
echo "🔍 Project structure:"
echo "├── Cargo.toml (with git dependency)"
echo "└── src/"
echo "    └── main.rs (test code)"

echo ""
echo "📋 Cargo.toml contents:"
cat Cargo.toml

echo ""
echo "🔨 Building project (this will download from git)..."
if cargo build; then
    echo ""
    echo "✅ Build successful!"
    
    echo ""
    echo "🚀 Running test..."
    cargo run
    
    echo ""
    echo "🎉 SUCCESS! Your SDK works perfectly from Git repository!"
    echo ""
    echo "📝 Users can now add this to their Cargo.toml:"
    echo "   anthropic-sdk-rust = { git = \"https://github.com/dimichgh/anthropic-sdk-rust\" }"
    
else
    echo ""
    echo "❌ Build failed. Check the error messages above."
    exit 1
fi

# Cleanup
cd ..
rm -rf "$TEST_DIR"

echo ""
echo "�� Cleanup complete!" 