# Custom Gateway Integration Tests

This project includes comprehensive integration tests for custom API gateway functionality.

## 🔧 Setup

Create a .env file with your custom gateway credentials:

CUSTOM_BEARER_TOKEN=your-bearer-token-here
CUSTOM_BASE_URL=https://your-custom-gateway.example.com/v1/anthropic

## 🚀 Quick Start

1. Set up: cp .env.example .env
2. Edit .env with your credentials
3. Run: cargo test custom_gateway_integration

## 🔧 Model Configuration

CUSTOM_MODEL_NAME: Your gateway's model identifier
Default: claude-3-5-sonnet-latest
