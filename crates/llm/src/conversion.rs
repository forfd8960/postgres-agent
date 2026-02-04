//! Message conversion between internal and OpenAI formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::error::LlmError;
use crate::prompt::{PromptMessage, PromptToolCall, PromptToolCallFunction};

/// OpenAI chat message format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum OpenAiMessage {
    /// System message.
    #[serde(rename = "system")]
    System {
        /// Content.
        content: String,
    },
    /// User message.
    #[serde(rename = "user")]
    User {
        /// Content.
        content: String,
    },
    /// Assistant message.
    #[serde(rename = "assistant")]
    Assistant {
        /// Content.
        content: Option<String>,
        /// Tool calls.
        #[serde(default)]
        tool_calls: Vec<OpenAiToolCall>,
    },
    /// Tool result.
    #[serde(rename = "tool")]
    Tool {
        /// Tool call ID.
        tool_call_id: String,
        /// Content.
        content: String,
    },
}

/// OpenAI tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAiToolCall {
    /// Call ID.
    pub id: String,
    /// Tool type.
    pub r#type: String,
    /// Function call.
    pub function: OpenAiFunctionCall,
}

/// OpenAI function call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAiFunctionCall {
    /// Function name.
    pub name: String,
    /// Arguments as JSON string.
    pub arguments: String,
}

/// OpenAI chat completion request.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAiChatRequest {
    /// Model identifier.
    pub model: String,
    /// Messages.
    pub messages: Vec<OpenAiMessage>,
    /// Temperature.
    pub temperature: Option<f32>,
    /// Maximum tokens.
    pub max_tokens: Option<u32>,
    /// Tool definitions.
    #[serde(default)]
    pub tools: Vec<OpenAiToolDefinition>,
    /// Response format.
    #[serde(default)]
    pub response_format: Value,
}

/// OpenAI tool definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAiToolDefinition {
    /// Tool type.
    pub r#type: String,
    /// Function specification.
    pub function: OpenAiFunctionSpec,
}

/// OpenAI function specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAiFunctionSpec {
    /// Function name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Parameters schema.
    pub parameters: Value,
}

/// OpenAI chat completion response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAiChatResponse {
    /// ID.
    pub id: String,
    /// Object type.
    pub object: String,
    /// Created timestamp.
    pub created: u64,
    /// Model used.
    pub model: String,
    /// Choices.
    pub choices: Vec<OpenAiChoice>,
    /// Usage statistics.
    pub usage: Option<OpenAiUsage>,
}

/// Choice in the response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAiChoice {
    /// Index.
    pub index: u32,
    /// Message.
    pub message: OpenAiMessage,
    /// Finish reason.
    pub finish_reason: Option<String>,
}

/// Token usage.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAiUsage {
    /// Prompt tokens.
    pub prompt_tokens: u32,
    /// Completion tokens.
    pub completion_tokens: u32,
    /// Total tokens.
    pub total_tokens: u32,
}

/// Convert internal prompt messages to OpenAI format.
#[must_use]
pub fn to_openai_messages(messages: &[PromptMessage]) -> Vec<OpenAiMessage> {
    messages
        .iter()
        .map(|m| match m {
            PromptMessage::System { content } => OpenAiMessage::System {
                content: content.clone(),
            },
            PromptMessage::User { content } => OpenAiMessage::User {
                content: content.clone(),
            },
            PromptMessage::Assistant { content, tool_calls } => OpenAiMessage::Assistant {
                content: Some(content.clone()),
                tool_calls: tool_calls
                    .iter()
                    .map(|tc| OpenAiToolCall {
                        id: tc.id.clone(),
                        r#type: tc.r#type.clone(),
                        function: OpenAiFunctionCall {
                            name: tc.function.name.clone(),
                            arguments: tc.function.arguments.clone(),
                        },
                    })
                    .collect(),
            },
            PromptMessage::Tool {
                tool_call_id,
                name: _,
                content,
            } => OpenAiMessage::Tool {
                tool_call_id: tool_call_id.clone(),
                content: content.clone(),
            },
        })
        .collect()
}

/// Convert OpenAI response to internal decision format.
pub fn from_openai_response(response: &OpenAiChatResponse) -> Result<Value, LlmError> {
    if response.choices.is_empty() {
        return Err(LlmError::NoResponse);
    }

    let message = &response.choices[0].message;

    match message {
        OpenAiMessage::Assistant { content, tool_calls } => {
            if !tool_calls.is_empty() {
                // Tool call
                let tool = &tool_calls[0];
                Ok(serde_json::json!({
                    "type": "tool_call",
                    "name": tool.function.name,
                    "arguments": tool.function.arguments,
                    "call_id": tool.id,
                }))
            } else if let Some(text) = content {
                // Final answer or reasoning
                // Try to parse as structured decision first
                if let Ok(decision) = serde_json::from_str::<Value>(text) {
                    if decision.get("type").is_some() {
                        return Ok(decision);
                    }
                }
                // Fall back to final answer
                Ok(serde_json::json!({
                    "type": "final_answer",
                    "answer": text
                }))
            } else {
                Err(LlmError::ApiError {
                    message: "Empty assistant message".to_string(),
                })
            }
        }
        _ => Err(LlmError::ApiError {
            message: "Unexpected message type in response".to_string(),
        }),
    }
}

/// Create tool definitions for OpenAI function calling.
#[must_use]
pub fn create_tool_definitions() -> Vec<OpenAiToolDefinition> {
    vec![
        OpenAiToolDefinition {
            r#type: "function".to_string(),
            function: OpenAiFunctionSpec {
                name: "execute_query".to_string(),
                description: "Execute a SQL SELECT query on the database".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "sql": {
                            "type": "string",
                            "description": "The SQL SELECT query to execute"
                        }
                    },
                    "required": ["sql"]
                }),
            },
        },
        OpenAiToolDefinition {
            r#type: "function".to_string(),
            function: OpenAiFunctionSpec {
                name: "get_schema".to_string(),
                description: "Get the database schema with tables and columns".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "filter": {
                            "type": "string",
                            "description": "Optional table name prefix filter"
                        }
                    }
                }),
            },
        },
        OpenAiToolDefinition {
            r#type: "function".to_string(),
            function: OpenAiFunctionSpec {
                name: "list_tables".to_string(),
                description: "List all tables in the database".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }),
            },
        },
        OpenAiToolDefinition {
            r#type: "function".to_string(),
            function: OpenAiFunctionSpec {
                name: "describe_table".to_string(),
                description: "Describe a specific table's structure".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "table_name": {
                            "type": "string",
                            "description": "The name of the table to describe"
                        }
                    },
                    "required": ["table_name"]
                }),
            },
        },
        OpenAiToolDefinition {
            r#type: "function".to_string(),
            function: OpenAiFunctionSpec {
                name: "explain_query".to_string(),
                description: "Get the query execution plan".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "sql": {
                            "type": "string",
                            "description": "The SQL query to explain"
                        }
                    },
                    "required": ["sql"]
                }),
            },
        },
    ]
}

/// Parse tool calls from OpenAI response.
#[must_use]
pub fn parse_tool_calls(response: &OpenAiChatResponse) -> Vec<PromptToolCall> {
    let mut calls = Vec::new();

    if let Some(choice) = response.choices.first() {
        if let OpenAiMessage::Assistant { tool_calls, .. } = &choice.message {
            for tc in tool_calls {
                calls.push(PromptToolCall {
                    id: tc.id.clone(),
                    r#type: tc.r#type.clone(),
                    function: PromptToolCallFunction {
                        name: tc.function.name.clone(),
                        arguments: tc.function.arguments.clone(),
                    },
                });
            }
        }
    }

    calls
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::{PromptBuilder, SystemPrompt};

    #[test]
    fn test_to_openai_messages() {
        let messages = PromptBuilder::new()
            .system("You are a helpful assistant.")
            .user("Hello, world!")
            .build();

        let openai_messages = to_openai_messages(&messages);
        assert_eq!(openai_messages.len(), 2);

        if let OpenAiMessage::System { content } = &openai_messages[0] {
            assert!(content.contains("helpful"));
        }
    }

    #[test]
    fn test_from_openai_response() {
        let response = OpenAiChatResponse {
            id: "test-123".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "gpt-4o".to_string(),
            choices: vec![OpenAiChoice {
                index: 0,
                message: OpenAiMessage::Assistant {
                    content: Some("Hello!".to_string()),
                    tool_calls: Vec::new(),
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: None,
        };

        let decision = from_openai_response(&response);
        assert!(decision.is_ok());
        let value = decision.unwrap();
        assert_eq!(value["type"], "final_answer");
    }

    #[test]
    fn test_create_tool_definitions() {
        let tools = create_tool_definitions();
        assert_eq!(tools.len(), 5);
        assert!(tools.iter().any(|t| t.function.name == "execute_query"));
    }
}
