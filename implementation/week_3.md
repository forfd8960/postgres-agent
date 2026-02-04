# Week 3 Implementation Summary: LLM Integration

## Document Information

| Attribute | Value |
|-----------|-------|
| Version | 1.0.0 |
| Status | Completed |
| Date | 2026-02-04 |
| Week | 3 |

---

## Overview

Week 3 focused on implementing the LLM integration layer, including the OpenAI provider, message conversion between internal and OpenAI formats, tool call parsing, and comprehensive system prompts for the PostgreSQL Agent.

---

## Deliverables

| Task | Status | Description |
|------|--------|-------------|
| 3.1 | Completed | Define LlmClient trait |
| 3.2 | Completed | Implement OpenAiProvider with async-openai stubs |
| 3.3 | Completed | Implement message conversion (to/from OpenAI format) |
| 3.4 | Completed | Implement tool call parsing |
| 3.5 | Completed | Create system prompts |

---

## Files Created

### `crates/llm/src/prompt.rs`

**New Types:**
- `SystemPrompt` - System prompt with base, tool instructions, safety instructions, and format instructions
- `PromptRole` - Enum for System, User, Assistant, Tool roles
- `PromptMessage` - Enum variants for different message types with tool call support
- `PromptToolCall` - Tool call structure with ID, type, and function
- `PromptToolCallFunction` - Function call with name and arguments
- `PromptBuilder` - Builder pattern for constructing prompt messages
- `ConversationHistory` - Conversation history with pruning based on message count and tokens

**New Methods:**
- `SystemPrompt::standard()`, `full()`, `base_only()`, `with_schema()`
- `PromptBuilder::new()`, `with_system_prompt()`, `system()`, `user()`, `assistant()`, `tool_result()`, `build()`
- `ConversationHistory::new()`, `with_max_messages()`, `with_max_tokens()`, `add()`, `extend()`, `messages()`, `recent()`, `clear()`, `token_estimate()`

### `crates/llm/src/conversion.rs`

**New Types:**
- `OpenAiMessage` - OpenAI chat message format (System, User, Assistant, Tool)
- `OpenAiToolCall` - OpenAI tool call structure
- `OpenAiFunctionCall` - OpenAI function call with name and arguments
- `OpenAiChatRequest` - Request format for OpenAI chat completions
- `OpenAiToolDefinition` - Tool definition for OpenAI function calling
- `OpenAiChatResponse` - Response format from OpenAI
- `OpenAiChoice` - Choice within response
- `OpenAiUsage` - Token usage statistics

**New Functions:**
- `to_openai_messages()` - Convert internal messages to OpenAI format
- `from_openai_response()` - Convert OpenAI response to internal decision format
- `create_tool_definitions()` - Create OpenAI tool definitions for database operations
- `parse_tool_calls()` - Parse tool calls from OpenAI response

### `crates/llm/src/prompts/` (Directory)

**Prompt Files:**
- `base.txt` - Base system prompt for PostgreSQL assistant
- `tools.txt` - Available tools documentation
- `safety.txt` - Safety guidelines and restrictions
- `format.txt` - Response format instructions

---

## Files Modified

### `crates/llm/src/lib.rs`

**Added Modules:**
- `conversion` - Message conversion between internal and OpenAI formats
- `prompt` - System prompts and prompt building

**Added Re-exports:**
- `to_openai_messages`, `from_openai_response`
- `PromptBuilder`, `PromptMessage`, `PromptRole`, `SystemPrompt`, `ConversationHistory`

### `crates/llm/src/openai.rs`

**Enhanced OpenAiProvider:**
- Added `system_prompt` field
- Added `history` field for conversation tracking
- Added `use_api` flag for API enablement
- Implemented `with_prompt()` constructor
- Implemented `set_use_api()` method
- Implemented `build_request()` for constructing OpenAI requests
- Implemented `call_api()` stub for actual API calls
- Enhanced `complete()`, `generate_decision()`, `generate_structured()` methods
- Added `convert_context_to_messages()` helper function

---

## Architecture

### Message Flow

```
User Query
    ↓
PostgresAgent::run()
    ↓
LlmClient::generate_decision()
    ↓
OpenAiProvider::generate_decision()
    ↓
convert_context_to_messages() [internal → OpenAI format]
    ↓
OpenAI API (or stub)
    ↓
from_openai_response() [OpenAI → internal format]
    ↓
AgentDecision (Reasoning/ToolCall/FinalAnswer)
```

### Tool Call Flow

```
LLM decides to call tool
    ↓
AgentDecision::ToolCall
    ↓
ToolExecutor::execute()
    ↓
Tool result added to context
    ↓
Next LLM iteration
```

---

## System Prompt Structure

```
## Base Instructions
You are an intelligent PostgreSQL database assistant...

## Available Tools
### execute_query
Execute a SQL SELECT query...

### get_schema
Get the database schema...

### list_tables
List all tables...

### describe_table
Describe a specific table...

### explain_query
Get the query execution plan...

## Safety Guidelines
- Read-Only by Default
- Query Validation
- PII Protection
- Rate Limiting
- Error Handling

## Response Format
- Reasoning (optional)
- Final Answer
- Tool Call Format
```

---

## OpenAI Integration

### Request Format
```json
{
  "model": "gpt-4o",
  "messages": [...],
  "temperature": 0.0,
  "max_tokens": 4096,
  "tools": [...],
  "response_format": { "type": "json_object" }
}
```

### Tool Definitions
Five database tools defined for OpenAI function calling:
1. `execute_query` - Execute SELECT queries
2. `get_schema` - Get database schema
3. `list_tables` - List all tables
4. `describe_table` - Describe table structure
5. `explain_query` - Get query execution plan

---

## Tests Added

### Prompt Tests
- `test_prompt_builder()` - Builder pattern construction
- `test_system_prompt_full()` - Full prompt generation
- `test_conversation_history()` - History pruning

### Conversion Tests
- `test_to_openai_messages()` - Message conversion
- `test_from_openai_response()` - Response parsing
- `test_create_tool_definitions()` - Tool definitions

### Provider Tests
- `test_openai_provider_new()` - Provider creation
- `test_openai_provider_with_prompt()` - Custom prompt
- `test_convert_context_to_messages()` - Context conversion
- `test_stub_complete()` - Stub response generation

---

## Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| async-openai | 0.32.4 | OpenAI API client (available for integration) |
| serde | 1.0 | Serialization/deserialization |
| serde_json | 1.0 | JSON handling |
| url | 2 | URL parsing for API endpoints |

---

## Next Steps (Week 4)

Week 4 will focus on Database Integration:
- Implement DbConnection with sqlx
- Implement QueryExecutor with SELECT support
- Implement DatabaseSchema extraction
- Implement tool execution layer

---

## Notes

- The OpenAI provider works in stub mode by default
- Set `OPENAI_API_KEY` environment variable and call `set_use_api(true)` for real API calls
- All messages are serialized to JSON for OpenAI API compatibility
- Tool calling uses OpenAI's function calling feature for structured tool invocation
- Conversation history is automatically pruned to stay within token limits
