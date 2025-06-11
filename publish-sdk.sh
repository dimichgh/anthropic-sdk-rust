#!/bin/bash

# Anthropic Rust SDK Publishing Script for Custom Infrastructure
# This script demonstrates the complete workflow of publishing the SDK
# to your custom registry and building its documentation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REGISTRY_NAME="anthropic"
REGISTRY_URL="https://localhost"

echo -e "${BLUE}🦀 Anthropic Rust SDK Publishing Workflow${NC}"
echo "=========================================="
echo ""

# Check if infrastructure is running
check_infrastructure() {
    echo -e "${BLUE}🔍 Checking infrastructure status...${NC}"
    
    if ! docker-compose -f "$SCRIPT_DIR/infrastructure/docker-compose.yml" ps | grep -q "Up"; then
        echo -e "${RED}❌ Infrastructure not running${NC}"
        echo -e "${YELLOW}💡 Start infrastructure first:${NC}"
        echo -e "   cd infrastructure && ./manage.sh"
        echo -e "   Select: 1. 🏗️ Deploy Infrastructure"
        exit 1
    fi
    
    # Test registry connectivity
    if ! curl -k -s "$REGISTRY_URL/health" >/dev/null; then
        echo -e "${RED}❌ Registry not accessible${NC}"
        echo -e "${YELLOW}💡 Check registry status:${NC}"
        echo -e "   cd infrastructure && ./manage.sh"
        echo -e "   Select: 6. 📊 Service Status"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Infrastructure is running${NC}"
    echo ""
}

# Setup Cargo configuration
setup_cargo_config() {
    echo -e "${BLUE}⚙️  Setting up Cargo configuration...${NC}"
    
    # Create cargo config directory
    mkdir -p ~/.cargo
    
    # Check if registry already configured
    if grep -q "anthropic" ~/.cargo/config.toml 2>/dev/null; then
        echo -e "${YELLOW}⚠️  Anthropic registry already configured${NC}"
    else
        # Add registry configuration
        cat >> ~/.cargo/config.toml << EOF

# Anthropic Private Registry Configuration
[registries]
$REGISTRY_NAME = { index = "$REGISTRY_URL/api/v1/crates/git-index" }

# Optional: Set as default registry
# [registry]
# default = "$REGISTRY_NAME"
EOF
        echo -e "${GREEN}✅ Cargo configuration updated${NC}"
    fi
    
    echo ""
}

# Generate and setup API token
setup_api_token() {
    echo -e "${BLUE}🔑 Setting up API token...${NC}"
    
    # Generate a token (in real scenario, this would come from the registry UI)
    local token="anthropic-rust-$(openssl rand -hex 16)"
    
    echo -e "${GREEN}✅ Generated API token: ${token}${NC}"
    echo -e "${YELLOW}💡 In production, get this token from:${NC}"
    echo -e "   cd infrastructure && ./manage.sh"
    echo -e "   Select: 8. 📦 Manage Registry → 2. 🔑 Generate API Token"
    echo ""
    
    # Login to registry
    echo -e "${BLUE}🔐 Logging into registry...${NC}"
    echo -e "${YELLOW}Note: Using demo token for this example${NC}"
    
    # For demo purposes, we'll skip the actual login
    echo -e "${GREEN}✅ Ready to publish (demo mode)${NC}"
    echo ""
}

# Prepare SDK for publishing
prepare_sdk() {
    echo -e "${BLUE}📦 Preparing SDK for publishing...${NC}"
    
    # Check if we're in the SDK directory
    if [ ! -f "$SCRIPT_DIR/Cargo.toml" ]; then
        echo -e "${RED}❌ Not in SDK directory${NC}"
        exit 1
    fi
    
    # Run tests
    echo -e "${YELLOW}🧪 Running tests...${NC}"
    cargo test --all-features
    echo -e "${GREEN}✅ All tests passed${NC}"
    
    # Build documentation
    echo -e "${YELLOW}📚 Building documentation...${NC}"
    cargo doc --no-deps --all-features
    echo -e "${GREEN}✅ Documentation built${NC}"
    
    # Format and lint
    echo -e "${YELLOW}🎨 Formatting and linting...${NC}"
    cargo fmt --check
    cargo clippy --all-features -- -D warnings
    echo -e "${GREEN}✅ Code quality checks passed${NC}"
    
    echo ""
}

# Publish SDK to registry
publish_sdk() {
    echo -e "${BLUE}🚀 Publishing anthropic-sdk-rust to private registry...${NC}"
    
    # Show what would be published
    echo -e "${YELLOW}📋 Package contents:${NC}"
    cargo package --list --registry $REGISTRY_NAME
    
    echo ""
    echo -e "${YELLOW}🤔 This would normally run:${NC}"
    echo -e "   cargo publish --registry $REGISTRY_NAME"
    echo ""
    echo -e "${GREEN}✅ anthropic-sdk-rust ready for publishing${NC}"
    echo -e "${YELLOW}💡 To actually publish, ensure your registry is fully configured and run:${NC}"
    echo -e "   cargo login --registry $REGISTRY_NAME <your-api-token>"
    echo -e "   cargo publish --registry $REGISTRY_NAME"
    echo ""
}

# Upload documentation
upload_documentation() {
    echo -e "${BLUE}📚 Uploading documentation to custom docs server...${NC}"
    
    # Copy documentation to the infrastructure docs volume
    local docs_source="$SCRIPT_DIR/target/doc"
    local docs_dest="$SCRIPT_DIR/infrastructure/data/docs/anthropic-sdk-rust"
    
    if [ -d "$docs_source" ]; then
        mkdir -p "$docs_dest"
        cp -r "$docs_source"/* "$docs_dest/"
        
        # Create index page for the SDK
        cat > "$docs_dest/index.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta http-equiv="refresh" content="0; url=anthropic_sdk/index.html">
    <title>Anthropic Rust SDK Documentation</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 800px;
            margin: 50px auto;
            padding: 20px;
            text-align: center;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            justify-content: center;
        }
        .container {
            background: rgba(255, 255, 255, 0.1);
            border-radius: 20px;
            padding: 40px;
            backdrop-filter: blur(10px);
            box-shadow: 0 8px 32px rgba(31, 38, 135, 0.37);
        }
        h1 { font-size: 2.5em; margin-bottom: 20px; }
        .subtitle { font-size: 1.2em; opacity: 0.9; margin-bottom: 30px; }
        .cta-button {
            display: inline-block;
            padding: 15px 30px;
            background: rgba(255, 255, 255, 0.2);
            color: white;
            text-decoration: none;
            border-radius: 50px;
            font-weight: bold;
            transition: all 0.3s ease;
            border: 2px solid rgba(255, 255, 255, 0.3);
        }
        .cta-button:hover {
            background: rgba(255, 255, 255, 0.3);
            transform: translateY(-2px);
            box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🦀 Anthropic Rust SDK</h1>
        <p class="subtitle">
            Comprehensive, type-safe Rust SDK for the Anthropic API
        </p>
        <a href="anthropic_sdk/index.html" class="cta-button">
            📚 View API Documentation
        </a>
        <p style="margin-top: 20px; font-style: italic; opacity: 0.8;">
            Redirecting automatically...
        </p>
    </div>
    
    <script>
        setTimeout(function() {
            window.location.href = 'anthropic_sdk/index.html';
        }, 3000);
    </script>
</body>
</html>
EOF
        
        echo -e "${GREEN}✅ Documentation uploaded${NC}"
        echo -e "${BLUE}📖 Available at: ${REGISTRY_URL}/docs/anthropic-sdk-rust/${NC}"
    else
        echo -e "${RED}❌ Documentation not found${NC}"
        echo -e "${YELLOW}💡 Run: cargo doc --no-deps --all-features${NC}"
    fi
    
    echo ""
}

# Create example project using the published SDK
create_example_project() {
    echo -e "${BLUE}📝 Creating example project using published SDK...${NC}"
    
    local example_dir="$SCRIPT_DIR/example-usage"
    mkdir -p "$example_dir"
    
    # Create Cargo.toml
    cat > "$example_dir/Cargo.toml" << EOF
[package]
name = "anthropic-sdk-rust-example"
version = "0.1.0"
edition = "2021"

[dependencies]
anthropic-sdk-rust = { registry = "$REGISTRY_NAME", version = "0.1.0" }
tokio = { version = "1.0", features = ["rt-multi-thread", "macros"] }
EOF
    
    # Create example main.rs
    cat > "$example_dir/src/main.rs" << 'EOF'
use anthropic_sdk::{Anthropic, MessageCreateBuilder, Result};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🦀 Anthropic Rust SDK Example - Using Private Registry");
    println!("=".repeat(55));
    
    // Create client
    let client = Anthropic::new("demo-api-key")?;
    println!("✅ Client created successfully!");
    
    // Create a message (demo mode - won't actually call API)
    let params = MessageCreateBuilder::new("claude-3-5-sonnet-latest", 1024)
        .user("Hello from the private registry!")
        .system("You are a helpful assistant.")
        .build();
    
    println!("✅ Message parameters created:");
    println!("   Model: {}", params.model);
    println!("   Max tokens: {}", params.max_tokens);
    println!("   Messages: {} message(s)", params.messages.len());
    
    // Test model selection
    let models = client.models();
    println!("✅ Models API accessible");
    
    println!("\n🎉 Successfully using Anthropic Rust SDK from private registry!");
    println!("📚 Documentation: https://localhost/docs/anthropic-sdk-rust/");
    
    Ok(())
}
EOF
    
    mkdir -p "$example_dir/src"
    
    echo -e "${GREEN}✅ Example project created at: $example_dir${NC}"
    echo -e "${YELLOW}💡 To test the example:${NC}"
    echo -e "   cd $example_dir"
    echo -e "   cargo run"
    echo ""
}

# Show usage summary
show_summary() {
    echo -e "${GREEN}🎉 Publishing Workflow Complete!${NC}"
    echo "=================================="
    echo ""
    echo -e "${BLUE}📋 What was accomplished:${NC}"
    echo -e "   ✅ Infrastructure status verified"
    echo -e "   ✅ Cargo configuration updated"
    echo -e "   ✅ API token setup (demo)"
    echo -e "   ✅ anthropic-sdk-rust prepared and tested"
    echo -e "   ✅ Documentation uploaded"
    echo -e "   ✅ Example project created"
    echo ""
    echo -e "${BLUE}🌐 Access Points:${NC}"
    echo -e "   📦 Registry:        $REGISTRY_URL/registry/"
    echo -e "   📚 Documentation:   $REGISTRY_URL/docs/anthropic-sdk-rust/"
    echo -e "   📊 Monitoring:      $REGISTRY_URL/monitoring/"
    echo -e "   🔍 Metrics:         http://localhost:9090/"
    echo ""
    echo -e "${BLUE}📝 Next Steps:${NC}"
    echo -e "   1. Get real API token from registry UI"
    echo -e "   2. Run: cargo login --registry $REGISTRY_NAME <token>"
    echo -e "   3. Run: cargo publish --registry $REGISTRY_NAME"
    echo -e "   4. Test with: cd example-usage && cargo run"
    echo ""
    echo -e "${YELLOW}🛠️  Infrastructure Management:${NC}"
    echo -e "   cd infrastructure && ./manage.sh"
    echo ""
}

# Run tests for the example
test_example() {
    echo -e "${BLUE}🧪 Testing example project...${NC}"
    
    local example_dir="$SCRIPT_DIR/example-usage"
    if [ -d "$example_dir" ]; then
        cd "$example_dir"
        
        echo -e "${YELLOW}📦 Building example...${NC}"
        if cargo build; then
            echo -e "${GREEN}✅ Example builds successfully${NC}"
            echo -e "${YELLOW}💡 Example is ready to run with real API token${NC}"
        else
            echo -e "${RED}❌ Example build failed${NC}"
        fi
        
        cd "$SCRIPT_DIR"
    fi
    
    echo ""
}

# Main execution flow
main() {
    check_infrastructure
    setup_cargo_config
    setup_api_token
    prepare_sdk
    publish_sdk
    upload_documentation
    create_example_project
    test_example
    show_summary
}

# Handle command line arguments
case "${1:-full}" in
    "full")
        main
        ;;
    "check")
        check_infrastructure
        ;;
    "config")
        setup_cargo_config
        setup_api_token
        ;;
    "publish")
        prepare_sdk
        publish_sdk
        ;;
    "docs")
        upload_documentation
        ;;
    "example")
        create_example_project
        test_example
        ;;
    *)
        echo "Usage: $0 {full|check|config|publish|docs|example}"
        echo ""
        echo "Commands:"
        echo "  full     - Run complete publishing workflow (default)"
        echo "  check    - Check infrastructure status"
        echo "  config   - Setup Cargo configuration only"
        echo "  publish  - Prepare and publish SDK"
        echo "  docs     - Upload documentation"
        echo "  example  - Create example project"
        exit 1
        ;;
esac 