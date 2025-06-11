//! Tool use types for function calling with Claude.
//!
//! This module provides comprehensive support for tool use (function calling),
//! allowing Claude to interact with external functions and APIs.
//!
//! # Examples
//!
//! ```rust
//! use anthropic_sdk::types::tools::{Tool, ToolChoice, ToolUse, ToolResult};
//! use serde_json::json;
//!
//! // Define a tool
//! let weather_tool = Tool::new("get_weather", "Get the current weather in a given location")
//!     .parameter("location", "string", "The city and state, e.g. San Francisco, CA")
//!     .required("location")
//!     .build();
//!
//! // Tool use request from Claude
//! let tool_use = ToolUse {
//!     id: "toolu_123".to_string(),
//!     name: "get_weather".to_string(),
//!     input: json!({"location": "San Francisco, CA"}),
//! };
//!
//! // Tool result after execution
//! let tool_result = ToolResult::success(
//!     tool_use.id.clone(),
//!     "The weather in San Francisco is 72°F and sunny"
//! );
//! ```

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A tool definition for function calling.
///
/// Tools allow Claude to call external functions with structured inputs.
/// Each tool has a name, description, and JSON schema for input validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tool {
    /// The name of the tool. Must be unique within the request.
    pub name: String,
    
    /// A detailed description of what the tool does.
    pub description: String,
    
    /// JSON schema definition for the tool's input parameters.
    pub input_schema: ToolInputSchema,
}

/// JSON schema for tool input parameters.
///
/// Defines the structure, types, and constraints for tool parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolInputSchema {
    /// The schema type (always "object" for tool inputs).
    #[serde(rename = "type")]
    pub schema_type: String,
    
    /// Property definitions for the input parameters.
    pub properties: Map<String, Value>,
    
    /// List of required parameter names.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required: Vec<String>,
    
    /// Additional schema properties.
    #[serde(flatten)]
    pub additional: Map<String, Value>,
}

/// Tool choice strategy for controlling which tools Claude can use.
///
/// This determines how Claude selects and uses available tools.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolChoice {
    /// Let Claude automatically decide whether and which tools to use.
    #[serde(rename = "auto")]
    Auto,
    
    /// Claude must use one of the available tools.
    #[serde(rename = "any")]
    Any,
    
    /// Force Claude to use a specific tool.
    #[serde(rename = "tool")]
    Tool {
        /// The name of the tool that must be used.
        name: String,
    },
}

/// A tool use request from Claude.
///
/// When Claude decides to use a tool, it returns this structure with
/// the tool name and input parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolUse {
    /// Unique identifier for this tool use request.
    pub id: String,
    
    /// The name of the tool to call.
    pub name: String,
    
    /// Input parameters for the tool call.
    pub input: Value,
}

/// Result of a tool execution.
///
/// After executing a tool, return the result using this structure
/// so Claude can incorporate it into its response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolResult {
    /// The ID of the tool use request this result corresponds to.
    pub tool_use_id: String,
    
    /// The result content from the tool execution.
    pub content: ToolResultContent,
    
    /// Whether the tool execution was successful.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Content of a tool result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    /// Simple text result.
    Text(String),
    
    /// Structured JSON result.
    Json(Value),
    
    /// Multiple content blocks (text, images, etc.).
    Blocks(Vec<ToolResultBlock>),
}

/// A content block in a tool result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolResultBlock {
    /// Text content block.
    #[serde(rename = "text")]
    Text {
        /// The text content.
        text: String,
    },
    
    /// Image content block.
    #[serde(rename = "image")]
    Image {
        /// Image source information.
        source: ImageSource,
    },
}

/// Image source for tool results.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ImageSource {
    /// Base64-encoded image data.
    #[serde(rename = "base64")]
    Base64 {
        /// MIME type of the image.
        media_type: String,
        /// Base64-encoded image data.
        data: String,
    },
}

/// Builder for creating tool definitions.
///
/// Provides a fluent API for constructing tools with parameters and validation.
#[derive(Debug, Clone)]
pub struct ToolBuilder {
    name: String,
    description: String,
    properties: Map<String, Value>,
    required: Vec<String>,
    additional: Map<String, Value>,
}

impl ToolBuilder {
    /// Create a new tool builder.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            properties: Map::new(),
            required: Vec::new(),
            additional: Map::new(),
        }
    }
    
    /// Add a parameter to the tool.
    ///
    /// # Arguments
    /// * `name` - Parameter name
    /// * `param_type` - Parameter type (e.g., "string", "number", "boolean")
    /// * `description` - Parameter description
    pub fn parameter(mut self, name: impl Into<String>, param_type: impl Into<String>, description: impl Into<String>) -> Self {
        let param_name = name.into();
        let param_schema = serde_json::json!({
            "type": param_type.into(),
            "description": description.into()
        });
        self.properties.insert(param_name, param_schema);
        self
    }
    
    /// Add an enum parameter with specific allowed values.
    pub fn enum_parameter(mut self, name: impl Into<String>, description: impl Into<String>, values: Vec<String>) -> Self {
        let param_name = name.into();
        let param_schema = serde_json::json!({
            "type": "string",
            "description": description.into(),
            "enum": values
        });
        self.properties.insert(param_name, param_schema);
        self
    }
    
    /// Add an array parameter.
    pub fn array_parameter(mut self, name: impl Into<String>, description: impl Into<String>, item_type: impl Into<String>) -> Self {
        let param_name = name.into();
        let param_schema = serde_json::json!({
            "type": "array",
            "description": description.into(),
            "items": {
                "type": item_type.into()
            }
        });
        self.properties.insert(param_name, param_schema);
        self
    }
    
    /// Add an object parameter with nested properties.
    pub fn object_parameter(mut self, name: impl Into<String>, description: impl Into<String>, properties: Map<String, Value>) -> Self {
        let param_name = name.into();
        let param_schema = serde_json::json!({
            "type": "object",
            "description": description.into(),
            "properties": properties
        });
        self.properties.insert(param_name, param_schema);
        self
    }
    
    /// Mark a parameter as required.
    pub fn required(mut self, name: impl Into<String>) -> Self {
        let param_name = name.into();
        if !self.required.contains(&param_name) {
            self.required.push(param_name);
        }
        self
    }
    
    /// Add additional schema properties.
    pub fn additional_property(mut self, key: impl Into<String>, value: Value) -> Self {
        self.additional.insert(key.into(), value);
        self
    }
    
    /// Build the tool definition.
    pub fn build(self) -> Tool {
        Tool {
            name: self.name,
            description: self.description,
            input_schema: ToolInputSchema {
                schema_type: "object".to_string(),
                properties: self.properties,
                required: self.required,
                additional: self.additional,
            },
        }
    }
}

impl Tool {
    /// Create a new tool builder.
    pub fn builder() -> ToolBuilder {
        ToolBuilder {
            name: String::new(),
            description: String::new(),
            properties: Map::new(),
            required: Vec::new(),
            additional: Map::new(),
        }
    }
    
    /// Create a tool builder with name and description.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> ToolBuilder {
        ToolBuilder::new(name, description)
    }
    
    /// Validate if the given input matches this tool's schema.
    pub fn validate_input(&self, input: &Value) -> Result<(), ToolValidationError> {
        // Basic validation - check required fields
        if let Value::Object(input_obj) = input {
            for required_field in &self.input_schema.required {
                if !input_obj.contains_key(required_field) {
                    return Err(ToolValidationError::MissingRequiredField {
                        field: required_field.clone(),
                        tool: self.name.clone(),
                    });
                }
            }
            
            // Check field types
            for (field_name, field_value) in input_obj {
                if let Some(property_schema) = self.input_schema.properties.get(field_name) {
                    self.validate_field_type(field_name, field_value, property_schema)?;
                }
            }
            
            Ok(())
        } else {
            Err(ToolValidationError::InvalidInputType {
                expected: "object".to_string(),
                actual: input.to_string(),
                tool: self.name.clone(),
            })
        }
    }
    
    fn validate_field_type(&self, field_name: &str, value: &Value, schema: &Value) -> Result<(), ToolValidationError> {
        if let Some(expected_type) = schema.get("type").and_then(|t| t.as_str()) {
            let actual_type = match value {
                Value::Null => "null",
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                Value::String(_) => "string",
                Value::Array(_) => "array",
                Value::Object(_) => "object",
            };
            
            if expected_type != actual_type {
                return Err(ToolValidationError::InvalidFieldType {
                    field: field_name.to_string(),
                    expected: expected_type.to_string(),
                    actual: actual_type.to_string(),
                    tool: self.name.clone(),
                });
            }
        }
        
        Ok(())
    }
}

impl ToolChoice {
    /// Create an auto tool choice.
    pub fn auto() -> Self {
        Self::Auto
    }
    
    /// Create an any tool choice.
    pub fn any() -> Self {
        Self::Any
    }
    
    /// Create a specific tool choice.
    pub fn tool(name: impl Into<String>) -> Self {
        Self::Tool { name: name.into() }
    }
}

impl ToolResult {
    /// Create a successful tool result with text content.
    pub fn success(tool_use_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: ToolResultContent::Text(content.into()),
            is_error: None,
        }
    }
    
    /// Create a successful tool result with JSON content.
    pub fn success_json(tool_use_id: impl Into<String>, content: Value) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: ToolResultContent::Json(content),
            is_error: None,
        }
    }
    
    /// Create an error tool result.
    pub fn error(tool_use_id: impl Into<String>, error_message: impl Into<String>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: ToolResultContent::Text(error_message.into()),
            is_error: Some(true),
        }
    }
    
    /// Create a tool result with multiple content blocks.
    pub fn with_blocks(tool_use_id: impl Into<String>, blocks: Vec<ToolResultBlock>) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content: ToolResultContent::Blocks(blocks),
            is_error: None,
        }
    }
}

impl ToolResultBlock {
    /// Create a text content block.
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }
    
    /// Create an image content block from base64 data.
    pub fn image_base64(media_type: impl Into<String>, data: impl Into<String>) -> Self {
        Self::Image {
            source: ImageSource::Base64 {
                media_type: media_type.into(),
                data: data.into(),
            },
        }
    }
}

/// Tool validation errors.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ToolValidationError {
    /// A required field is missing from the input.
    #[error("Missing required field '{field}' for tool '{tool}'")]
    MissingRequiredField { field: String, tool: String },
    
    /// Invalid input type (expected object).
    #[error("Invalid input type for tool '{tool}': expected {expected}, got {actual}")]
    InvalidInputType { expected: String, actual: String, tool: String },
    
    /// Invalid field type.
    #[error("Invalid type for field '{field}' in tool '{tool}': expected {expected}, got {actual}")]
    InvalidFieldType { field: String, expected: String, actual: String, tool: String },
}

/// Server-side tools provided by Anthropic.
///
/// These tools are executed on Anthropic's servers and don't require
/// client-side implementation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerTool {
    /// Web search tool for retrieving current information.
    #[serde(rename = "web_search_20250305")]
    WebSearch {
        /// Optional search parameters.
        #[serde(skip_serializing_if = "Option::is_none")]
        parameters: Option<WebSearchParameters>,
    },
}

/// Parameters for the web search server tool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebSearchParameters {
    /// Maximum number of search results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    max_results: Option<u32>,
    
    /// Search language preference.
    #[serde(skip_serializing_if = "Option::is_none")]
    language: Option<String>,
    
    /// Geographic region for search results.
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<String>,
}

impl ServerTool {
    /// Create a web search tool with default parameters.
    pub fn web_search() -> Self {
        Self::WebSearch { parameters: None }
    }
    
    /// Create a web search tool with custom parameters.
    pub fn web_search_with_params(parameters: WebSearchParameters) -> Self {
        Self::WebSearch { parameters: Some(parameters) }
    }
}

impl WebSearchParameters {
    /// Create web search parameters with maximum results.
    pub fn with_max_results(max_results: u32) -> Self {
        Self {
            max_results: Some(max_results),
            language: None,
            region: None,
        }
    }
    
    /// Set the search language.
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }
    
    /// Set the search region.
    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tool_builder() {
        let tool = Tool::new("get_weather", "Get the current weather")
            .parameter("location", "string", "The location to get weather for")
            .parameter("unit", "string", "Temperature unit")
            .enum_parameter("format", "Response format", vec!["json".to_string(), "text".to_string()])
            .required("location")
            .build();

        assert_eq!(tool.name, "get_weather");
        assert_eq!(tool.description, "Get the current weather");
        assert_eq!(tool.input_schema.required, vec!["location"]);
        assert_eq!(tool.input_schema.properties.len(), 3);
    }

    #[test]
    fn test_tool_validation() {
        let tool = Tool::new("test_tool", "Test tool")
            .parameter("required_field", "string", "Required field")
            .parameter("optional_field", "number", "Optional field")
            .required("required_field")
            .build();

        // Valid input
        let valid_input = json!({
            "required_field": "test",
            "optional_field": 42
        });
        assert!(tool.validate_input(&valid_input).is_ok());

        // Missing required field
        let invalid_input = json!({
            "optional_field": 42
        });
        assert!(tool.validate_input(&invalid_input).is_err());

        // Wrong type
        let wrong_type_input = json!({
            "required_field": 123
        });
        assert!(tool.validate_input(&wrong_type_input).is_err());
    }

    #[test]
    fn test_tool_choice_serialization() {
        let auto_choice = ToolChoice::auto();
        let json = serde_json::to_value(&auto_choice).unwrap();
        assert_eq!(json, json!({"type": "auto"}));

        let tool_choice = ToolChoice::tool("get_weather");
        let json = serde_json::to_value(&tool_choice).unwrap();
        assert_eq!(json, json!({"type": "tool", "name": "get_weather"}));
    }

    #[test]
    fn test_tool_result_creation() {
        let success_result = ToolResult::success("tool_123", "Success message");
        assert_eq!(success_result.tool_use_id, "tool_123");
        assert!(success_result.is_error.is_none());

        let error_result = ToolResult::error("tool_456", "Error message");
        assert_eq!(error_result.tool_use_id, "tool_456");
        assert_eq!(error_result.is_error, Some(true));

        let json_result = ToolResult::success_json("tool_789", json!({"temperature": 72}));
        if let ToolResultContent::Json(value) = json_result.content {
            assert_eq!(value["temperature"], 72);
        } else {
            panic!("Expected JSON content");
        }
    }

    #[test]
    fn test_server_tool_creation() {
        let web_search = ServerTool::web_search();
        assert!(matches!(web_search, ServerTool::WebSearch { parameters: None }));

        let params = WebSearchParameters::with_max_results(10)
            .language("en")
            .region("US");
        let web_search_with_params = ServerTool::web_search_with_params(params);
        
        if let ServerTool::WebSearch { parameters: Some(p) } = web_search_with_params {
            assert_eq!(p.max_results, Some(10));
            assert_eq!(p.language, Some("en".to_string()));
            assert_eq!(p.region, Some("US".to_string()));
        } else {
            panic!("Expected web search with parameters");
        }
    }

    #[test]
    fn test_tool_serialization() {
        let tool = Tool::new("calculate", "Perform mathematical calculations")
            .parameter("expression", "string", "Mathematical expression to evaluate")
            .required("expression")
            .build();

        let json = serde_json::to_string(&tool).unwrap();
        let deserialized: Tool = serde_json::from_str(&json).unwrap();
        assert_eq!(tool, deserialized);
    }

    #[test]
    fn test_tool_use_deserialization() {
        let json = r#"
        {
            "id": "toolu_123456",
            "name": "get_weather",
            "input": {
                "location": "San Francisco, CA",
                "unit": "celsius"
            }
        }"#;

        let tool_use: ToolUse = serde_json::from_str(json).unwrap();
        assert_eq!(tool_use.id, "toolu_123456");
        assert_eq!(tool_use.name, "get_weather");
        assert_eq!(tool_use.input["location"], "San Francisco, CA");
        assert_eq!(tool_use.input["unit"], "celsius");
    }
} 