//! Streaming response types for real-time message generation.
//!
//! This module provides types for handling Server-Sent Events (SSE) from the Anthropic API
//! when streaming is enabled. It includes all event types and delta structures needed to
//! process incremental responses from Claude.

use serde::{Deserialize, Serialize};
use crate::types::{Message, ContentBlock, StopReason, ServerToolUsage};

/// Main stream event type that encompasses all possible streaming events.
///
/// This is the primary type you'll work with when processing streaming responses.
/// Each event represents a different stage of the message generation process.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum MessageStreamEvent {
    /// Initial event when a message starts being generated.
    ///
    /// Contains the initial message structure with metadata but no content yet.
    #[serde(rename = "message_start")]
    MessageStart {
        /// The initial message structure
        message: Message,
    },

    /// Event when the message metadata is updated during generation.
    ///
    /// Contains updates to stop reason, stop sequence, and usage information.
    #[serde(rename = "message_delta")]
    MessageDelta {
        /// The delta containing updated fields
        delta: MessageDelta,
        /// Current usage statistics
        usage: MessageDeltaUsage,
    },

    /// Final event when message generation is complete.
    #[serde(rename = "message_stop")]
    MessageStop,

    /// Event when a new content block starts being generated.
    ///
    /// This occurs when Claude begins generating a new piece of content
    /// (text, tool use, etc.).
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        /// The content block being started
        content_block: ContentBlock,
        /// Index of this content block in the message
        index: usize,
    },

    /// Event containing incremental updates to a content block.
    ///
    /// This is where you'll receive the actual text being generated,
    /// tool input being parsed, etc.
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        /// The incremental update
        delta: ContentBlockDelta,
        /// Index of the content block being updated
        index: usize,
    },

    /// Event when a content block finishes generation.
    #[serde(rename = "content_block_stop")]
    ContentBlockStop {
        /// Index of the content block that finished
        index: usize,
    },
}

/// Delta updates for message-level information during streaming.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageDelta {
    /// Updated stop reason, if the message has stopped
    pub stop_reason: Option<StopReason>,
    /// The stop sequence that triggered stopping, if any
    pub stop_sequence: Option<String>,
}

/// Usage statistics for streaming responses.
///
/// These are cumulative totals that get updated throughout the stream.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageDeltaUsage {
    /// Cumulative output tokens generated so far
    pub output_tokens: u32,
    /// Cumulative input tokens used (may be null during streaming)
    pub input_tokens: Option<u32>,
    /// Cumulative cache creation tokens (may be null)
    pub cache_creation_input_tokens: Option<u32>,
    /// Cumulative cache read tokens (may be null)
    pub cache_read_input_tokens: Option<u32>,
    /// Server tool usage statistics (may be null)
    pub server_tool_use: Option<ServerToolUsage>,
}



/// Delta updates for content blocks during streaming.
///
/// This enum contains all possible types of incremental updates
/// that can occur within a content block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ContentBlockDelta {
    /// Incremental text content being generated.
    #[serde(rename = "text_delta")]
    TextDelta {
        /// The new text to append
        text: String,
    },

    /// Incremental JSON input for tool use blocks.
    ///
    /// This provides partial JSON as it's being parsed,
    /// useful for streaming tool input generation.
    #[serde(rename = "input_json_delta")]
    InputJsonDelta {
        /// Partial JSON string
        partial_json: String,
    },

    /// Citations being added to text blocks.
    #[serde(rename = "citations_delta")]
    CitationsDelta {
        /// The citation being added
        citation: TextCitation,
    },

    /// Incremental thinking content (extended reasoning).
    #[serde(rename = "thinking_delta")]
    ThinkingDelta {
        /// The thinking text being generated
        thinking: String,
    },

    /// Signature updates for thinking blocks.
    #[serde(rename = "signature_delta")]
    SignatureDelta {
        /// The signature string
        signature: String,
    },
}

/// Citation information for text blocks.
///
/// Citations provide source attribution for generated text,
/// with different types depending on the source document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum TextCitation {
    /// Citation pointing to character locations in plain text.
    #[serde(rename = "char_location")]
    CharLocation {
        /// The text being cited
        cited_text: String,
        /// Index of the source document
        document_index: usize,
        /// Title of the source document (may be null)
        document_title: Option<String>,
        /// Starting character index
        start_char_index: usize,
        /// Ending character index
        end_char_index: usize,
    },

    /// Citation pointing to page locations in PDFs.
    #[serde(rename = "page_location")]
    PageLocation {
        /// The text being cited
        cited_text: String,
        /// Index of the source document
        document_index: usize,
        /// Title of the source document (may be null)
        document_title: Option<String>,
        /// Starting page number
        start_page_number: usize,
        /// Ending page number
        end_page_number: usize,
    },

    /// Citation pointing to content block locations.
    #[serde(rename = "content_block_location")]
    ContentBlockLocation {
        /// The text being cited
        cited_text: String,
        /// Index of the source document
        document_index: usize,
        /// Title of the source document (may be null)
        document_title: Option<String>,
        /// Starting content block index
        start_block_index: usize,
        /// Ending content block index
        end_block_index: usize,
    },

    /// Citation pointing to web search results.
    #[serde(rename = "web_search_result_location")]
    WebSearchResultLocation {
        /// The text being cited
        cited_text: String,
        /// Encrypted index for the search result
        encrypted_index: String,
        /// Title of the web page (may be null)
        title: Option<String>,
        /// URL of the web page
        url: String,
    },
}

/// Type aliases for clarity and compatibility with the main API types.
pub type MessageStartEvent = MessageStreamEvent;
pub type MessageDeltaEvent = MessageStreamEvent;
pub type MessageStopEvent = MessageStreamEvent;
pub type ContentBlockStartEvent = MessageStreamEvent;
pub type ContentBlockDeltaEvent = MessageStreamEvent;
pub type ContentBlockStopEvent = MessageStreamEvent;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use crate::types::Usage;

    #[test]
    fn test_message_start_event_serialization() {
        let event = MessageStreamEvent::MessageStart {
            message: Message {
                id: "msg_123".to_string(),
                type_: "message".to_string(),
                role: crate::types::Role::Assistant,
                content: vec![],
                model: "claude-3-5-sonnet-latest".to_string(),
                stop_reason: None,
                stop_sequence: None,
                usage: Usage {
                    input_tokens: 10,
                    output_tokens: 0,
                    cache_creation_input_tokens: None,
                    cache_read_input_tokens: None,
                    server_tool_use: None,
                    service_tier: None,
                },
                request_id: None,
            },
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: MessageStreamEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, parsed);
    }

    #[test]
    fn test_content_block_delta_serialization() {
        let event = MessageStreamEvent::ContentBlockDelta {
            delta: ContentBlockDelta::TextDelta {
                text: "Hello".to_string(),
            },
            index: 0,
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: MessageStreamEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, parsed);
    }

    #[test]
    fn test_message_delta_event_serialization() {
        let event = MessageStreamEvent::MessageDelta {
            delta: MessageDelta {
                stop_reason: Some(StopReason::EndTurn),
                stop_sequence: None,
            },
            usage: MessageDeltaUsage {
                output_tokens: 25,
                input_tokens: Some(10),
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
                server_tool_use: None,
            },
        };

        let json = serde_json::to_string(&event).unwrap();
        let parsed: MessageStreamEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, parsed);
    }

    #[test]
    fn test_citation_serialization() {
        let citation = TextCitation::CharLocation {
            cited_text: "Example text".to_string(),
            document_index: 0,
            document_title: Some("Document Title".to_string()),
            start_char_index: 10,
            end_char_index: 23,
        };

        let json = serde_json::to_string(&citation).unwrap();
        let parsed: TextCitation = serde_json::from_str(&json).unwrap();
        assert_eq!(citation, parsed);
    }

    #[test]
    fn test_all_delta_types() {
        let deltas = vec![
            ContentBlockDelta::TextDelta {
                text: "Hello world".to_string(),
            },
            ContentBlockDelta::InputJsonDelta {
                partial_json: r#"{"key": "val"#.to_string(),
            },
            ContentBlockDelta::ThinkingDelta {
                thinking: "Let me think...".to_string(),
            },
            ContentBlockDelta::SignatureDelta {
                signature: "signature_123".to_string(),
            },
        ];

        for delta in deltas {
            let json = serde_json::to_string(&delta).unwrap();
            let parsed: ContentBlockDelta = serde_json::from_str(&json).unwrap();
            assert_eq!(delta, parsed);
        }
    }
} 