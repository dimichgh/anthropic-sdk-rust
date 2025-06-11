# Phase 5: Beta Features Implementation Plan

## Objective
Implement beta and utility features that enhance the SDK's capabilities, focusing on batch processing, file management, model information, and pagination utilities. These features provide additional functionality beyond the core messaging API.

## Priority: Medium (Useful but not essential for basic functionality)

## Components to Implement

### 1. Message Batches API (Beta Feature) - **HIGHEST PRIORITY**
**Batch Processing**:
- `MessageBatch` - Batch request container
- `BatchRequest` - Individual request in batch
- `BatchResponse` - Batch processing results
- `BatchStatus` - Processing status tracking
- Batch creation, monitoring, and result retrieval
- Error handling for batch failures

**Key Features**:
- Efficient bulk message processing
- Cost optimization for large workloads
- Async batch status monitoring
- Result aggregation and error collection

### 2. Files API (Beta Feature)
**File Management**:
- `FileObject` - File metadata and content
- `FileUpload` - File upload operations
- `FileList` - File listing and pagination
- File upload, retrieval, and deletion
- Content type validation and processing

**Integration**:
- Seamless integration with existing File system
- Enhanced file metadata management
- File lifecycle management
- Storage quota and limits handling

### 3. Models API (Informational)
**Model Information**:
- `Model` - Model capabilities and metadata
- `ModelList` - Available models listing
- Model capability queries
- Pricing and rate limit information
- Feature support detection

**Capabilities**:
- Dynamic model discovery
- Feature compatibility checking
- Cost estimation improvements
- Model recommendation system

### 4. Auto-pagination Utilities
**Pagination Support**:
- `PageIterator<T>` - Generic pagination iterator
- `PagedResponse<T>` - Paginated response wrapper
- `PaginationParams` - Pagination configuration
- Automatic page fetching and iteration
- Memory-efficient streaming pagination

**Features**:
- Async iteration over paginated results
- Configurable page sizes and limits
- Error handling and retry for pagination
- Type-safe pagination for all list endpoints

## Implementation Strategy

### Phase 5.1: Message Batches API (Day 1-2)
**Batch Types** (`src/types/batches.rs`):
- Define all batch-related types
- Request/response structures
- Status tracking and error handling
- Batch configuration options

**Batches Resource** (`src/resources/batches.rs`):
- Batch creation and submission
- Status monitoring and polling
- Result retrieval and processing
- Batch cancellation support

### Phase 5.2: Files API Enhancement (Day 2-3)
**File API Types** (`src/types/files_api.rs`):
- Extended file metadata types
- Upload progress tracking
- File listing and filtering
- Storage management types

**Files Resource** (`src/resources/files.rs`):
- File upload with progress tracking
- File listing with pagination
- File metadata retrieval
- File deletion and cleanup

### Phase 5.3: Models API (Day 3)
**Model Types** (`src/types/models.rs`):
- Model metadata and capabilities
- Pricing and rate limit information
- Feature support matrices
- Model comparison utilities

**Models Resource** (`src/resources/models.rs`):
- Model listing and discovery
- Capability queries
- Pricing information retrieval
- Model recommendation logic

### Phase 5.4: Auto-pagination (Day 4)
**Pagination Framework** (`src/pagination/mod.rs`):
- Generic pagination traits
- Iterator implementations
- Page fetching logic
- Error handling and retry

**Integration** (Various resources):
- Add pagination to all list endpoints
- Implement async iteration
- Memory optimization
- Performance benchmarks

## Key Technical Decisions

### Batch Processing Architecture
**Rust Approach**: Async batch monitoring with status polling
**Benefits**: Non-blocking batch processing, efficient resource usage

### File API Strategy
**Method**: Extend existing file system with API integration
**Benefits**: Unified file handling, consistent API patterns

### Pagination Design
**Strategy**: Generic trait-based pagination with async iterators
**Benefits**: Type safety, memory efficiency, reusable across endpoints

## Dependencies to Add
```toml
# Pagination and async utilities
futures-util = "0.3"     # Stream utilities
pin-project = "1.0"      # Pin projection for custom streams

# Enhanced HTTP handling
reqwest = { version = "0.11", features = ["stream", "multipart"] }

# Batch processing utilities
dashmap = "5.0"          # Concurrent hash maps for batch tracking
```

## Success Criteria
- [ ] Message batches API fully functional with status monitoring
- [ ] Files API integrates seamlessly with existing file system
- [ ] Models API provides comprehensive model information
- [ ] Auto-pagination works across all list endpoints
- [ ] Performance benchmarks show efficiency gains
- [ ] Complete test coverage (>95%)
- [ ] TypeScript SDK feature parity for beta features

## API Usage Examples

### Message Batches
```rust
use anthropic_sdk::{Anthropic, MessageBatch, BatchRequest};

let client = Anthropic::from_env()?;

// Create batch requests
let requests = vec![
    BatchRequest::new("req1", "claude-3-5-sonnet-latest", 1024)
        .user("Translate to French: Hello")
        .build(),
    BatchRequest::new("req2", "claude-3-5-sonnet-latest", 1024)
        .user("Translate to Spanish: Hello")
        .build(),
];

// Submit batch
let batch = client.batches()
    .create(requests)
    .await?;

// Monitor progress
while !batch.is_complete() {
    tokio::time::sleep(Duration::from_secs(5)).await;
    batch.refresh().await?;
    println!("Progress: {}/{}", batch.completed_count(), batch.total_count());
}

// Retrieve results
let results = batch.get_results().await?;
for result in results {
    println!("Request {}: {:?}", result.request_id, result.response);
}
```

### Files API
```rust
use anthropic_sdk::{Anthropic, FileUpload};

let client = Anthropic::from_env()?;

// Upload file
let upload = FileUpload::from_path("document.pdf")
    .with_purpose("batch_input")
    .with_progress_callback(|progress| {
        println!("Upload progress: {}%", progress.percentage());
    });

let file_object = client.files()
    .upload(upload)
    .await?;

// List files
let files = client.files()
    .list()
    .purpose("batch_input")
    .paginate()
    .collect::<Vec<_>>()
    .await?;

// Download file
let content = client.files()
    .get_content(&file_object.id)
    .await?;
```

### Models API
```rust
use anthropic_sdk::Anthropic;

let client = Anthropic::from_env()?;

// List available models
let models = client.models()
    .list()
    .await?;

for model in models {
    println!("Model: {} - Max tokens: {}", model.id, model.max_tokens);
    println!("Capabilities: {:?}", model.capabilities);
    println!("Pricing: ${:.4}/1K tokens", model.pricing.input_per_1k);
}

// Get specific model info
let sonnet = client.models()
    .get("claude-3-5-sonnet-latest")
    .await?;

println!("Context window: {} tokens", sonnet.context_window);
println!("Supports tools: {}", sonnet.supports_tools);
```

### Auto-pagination
```rust
use anthropic_sdk::Anthropic;
use futures_util::StreamExt;

let client = Anthropic::from_env()?;

// Paginated iteration
let mut file_stream = client.files()
    .list()
    .limit(10)
    .stream();

while let Some(file) = file_stream.next().await {
    let file = file?;
    println!("File: {} ({})", file.name, file.size);
}

// Collect all pages
let all_files: Vec<_> = client.files()
    .list()
    .paginate()
    .collect()
    .await?;

println!("Total files: {}", all_files.len());
```

## Files to Create/Modify
- `src/types/batches.rs` - Batch processing types
- `src/types/files_api.rs` - Extended file API types  
- `src/types/models.rs` - Model information types
- `src/resources/batches.rs` - Batch operations
- `src/resources/files.rs` - Enhanced file operations
- `src/resources/models.rs` - Model information API
- `src/pagination/mod.rs` - Pagination framework
- `src/lib.rs` - Export new modules
- `examples/batches.rs` - Batch processing examples
- `examples/files_api.rs` - Files API examples
- `examples/models.rs` - Models API examples
- `examples/pagination.rs` - Pagination examples

This phase will provide comprehensive beta feature support, enhancing the SDK's utility for production applications with batch processing, enhanced file management, model discovery, and efficient pagination. 