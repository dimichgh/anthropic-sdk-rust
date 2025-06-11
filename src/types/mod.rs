pub mod errors;
pub mod shared;
pub mod messages;
pub mod models;
pub mod models_api;
pub mod streaming;
pub mod tools;
pub mod batches;
pub mod files_api;

// Re-exports for convenience
pub use errors::{AnthropicError, Result};
pub use shared::{RequestId, Usage, ServerToolUsage, HasRequestId};

// Message types
pub use messages::{
    Message, Role, ContentBlock, ImageSource, StopReason,
    MessageCreateParams, MessageParam, MessageContent, ContentBlockParam,
    MessageCreateBuilder,
};

// Model types
pub use models::Model;

// Streaming types
pub use streaming::{
    MessageStreamEvent, MessageDelta, MessageDeltaUsage,
    ContentBlockDelta, TextCitation,
    MessageStartEvent, MessageDeltaEvent, MessageStopEvent,
    ContentBlockStartEvent, ContentBlockDeltaEvent, ContentBlockStopEvent,
};

// Tool types
pub use tools::{
    Tool, ToolBuilder, ToolChoice, ToolUse, ToolResult, ToolResultContent,
    ToolResultBlock, ToolInputSchema, ToolValidationError,
    ServerTool, WebSearchParameters, ImageSource as ToolImageSource,
};

// Batch types
pub use batches::{
    MessageBatch, BatchStatus, BatchRequestCounts, BatchRequest, BatchRequestBuilder,
    BatchResult, BatchResponse, BatchResponseBody, BatchError,
    BatchCreateParams, BatchListParams, BatchList,
};

// Files API types
pub use files_api::{
    FileObject, FilePurpose, FileStatus, FileUploadParams, FileListParams, FileList,
    FileOrder, UploadProgress, StorageInfo, FileDownload,
};

// Models API types
pub use models_api::{
    ModelObject, ModelListParams, ModelList, ModelCapabilities, ModelCapability,
    ModelPricing, PricingTier, ModelComparison, ModelPerformance, ComparisonSummary,
    ModelRequirements, ModelUsageRecommendations, ModelRecommendation,
    RecommendedParameters, PerformanceExpectations, CostRange, QualityLevel,
    CostEstimation, CostBreakdown,
}; 