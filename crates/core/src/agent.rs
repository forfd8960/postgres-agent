//! Agent core implementation.

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use postgres_agent_llm::client::LlmClient;
pub use postgres_agent_llm::error::LlmError;
pub use postgres_agent_tools::registry::ToolRegistry;
pub use postgres_agent_tools::{ToolContext, ToolError};

use crate::context::{AgentContext, Message};
use crate::decision::{AgentDecision, ToolCall, ToolResult};
use crate::error::AgentError;

/// Configuration for agent behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    /// Maximum number of reasoning iterations.
    #[serde(default = "default_max_iterations")]
    pub max_iterations: u32,
    /// Whether to require confirmation for mutations.
    #[serde(default)]
    pub require_confirmation: bool,
    /// Safety level for operations.
    #[serde(default)]
    pub safety_level: SafetyLevel,
    /// Timeout for each iteration in seconds.
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    /// Whether to enable verbose reasoning output.
    #[serde(default)]
    pub verbose_reasoning: bool,
}

fn default_max_iterations() -> u32 {
    10
}

fn default_timeout() -> u64 {
    30
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            require_confirmation: true,
            safety_level: SafetyLevel::Balanced,
            timeout_seconds: 30,
            verbose_reasoning: false,
        }
    }
}

/// Builder for AgentConfig.
#[derive(Debug, Clone, Default)]
pub struct AgentConfigBuilder {
    config: AgentConfig,
}

impl AgentConfigBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: AgentConfig::default(),
        }
    }

    /// Set maximum iterations.
    #[must_use]
    pub fn max_iterations(mut self, iterations: u32) -> Self {
        self.config.max_iterations = iterations;
        self
    }

    /// Set whether to require confirmation.
    #[must_use]
    pub fn require_confirmation(mut self, required: bool) -> Self {
        self.config.require_confirmation = required;
        self
    }

    /// Set safety level.
    #[must_use]
    pub fn safety_level(mut self, level: SafetyLevel) -> Self {
        self.config.safety_level = level;
        self
    }

    /// Set timeout in seconds.
    #[must_use]
    pub fn timeout_seconds(mut self, seconds: u64) -> Self {
        self.config.timeout_seconds = seconds;
        self
    }

    /// Enable verbose reasoning.
    #[must_use]
    pub fn verbose_reasoning(mut self, verbose: bool) -> Self {
        self.config.verbose_reasoning = verbose;
        self
    }

    /// Build the config.
    #[must_use]
    pub fn build(self) -> AgentConfig {
        self.config
    }
}

/// Safety levels controlling agent behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SafetyLevel {
    /// Maximum safety - read-only, no modifications.
    ReadOnly,
    /// Balanced safety - confirmations for DML/DDL.
    Balanced,
    /// Permissive - faster execution with minimal checks.
    Permissive,
}

impl Default for SafetyLevel {
    fn default() -> Self {
        SafetyLevel::Balanced
    }
}

/// State of the agent during execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentState {
    /// Agent is idle, waiting for input.
    Idle,
    /// Agent is thinking/reasoning.
    Thinking,
    /// Agent is waiting for user confirmation.
    AwaitingConfirmation,
    /// Agent is executing a tool.
    ExecutingTool,
    /// Agent has completed with a final answer.
    Completed,
    /// Agent encountered an error.
    Error(String),
}

impl Default for AgentState {
    fn default() -> Self {
        AgentState::Idle
    }
}

/// Result of running the agent.
#[derive(Debug, Clone)]
pub struct AgentResponse {
    /// The final answer to the user.
    pub answer: String,
    /// SQL that was executed (if any).
    pub executed_sql: Option<String>,
    /// Number of iterations taken.
    pub iterations: u32,
    /// Whether the query was successful.
    pub success: bool,
    /// Any error message.
    pub error: Option<String>,
    /// Final agent state.
    pub state: AgentState,
}

impl AgentResponse {
    /// Create a successful response.
    #[must_use]
    pub fn success(answer: String, iterations: u32) -> Self {
        Self {
            answer,
            executed_sql: None,
            iterations,
            success: true,
            error: None,
            state: AgentState::Completed,
        }
    }

    /// Create an error response.
    #[must_use]
    pub fn error(message: String, iterations: u32) -> Self {
        let error_msg = message.clone();
        Self {
            answer: String::new(),
            executed_sql: None,
            iterations,
            success: false,
            error: Some(message),
            state: AgentState::Error(error_msg),
        }
    }

    /// Create a response with executed SQL.
    #[must_use]
    pub fn with_sql(answer: String, sql: String, iterations: u32) -> Self {
        Self {
            answer,
            executed_sql: Some(sql),
            iterations,
            success: true,
            error: None,
            state: AgentState::Completed,
        }
    }
}

/// Statistics about agent execution.
#[derive(Debug, Default)]
pub struct AgentStats {
    /// Total iterations.
    pub iterations: u32,
    /// Total tool calls.
    pub tool_calls: u32,
    /// Total reasoning tokens.
    pub reasoning_tokens: u32,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
}

/// The core agent that implements the ReAct reasoning loop.
#[derive(Debug)]
pub struct PostgresAgent<Client: LlmClient> {
    /// LLM client for generating decisions.
    llm_client: Box<Client>,
    /// Context manager for conversation state.
    pub context: AgentContext,
    /// Tool registry for executing tools.
    tools: ToolRegistry,
    /// Agent configuration.
    pub config: AgentConfig,
    /// Current execution state.
    state: AgentState,
    /// Execution statistics.
    stats: AgentStats,
    /// Tool execution context.
    tool_context: ToolContext,
}

impl<Client: LlmClient> PostgresAgent<Client> {
    /// Create a new agent with default configuration.
    #[must_use]
    pub fn new(llm_client: Box<Client>) -> Self {
        Self {
            llm_client,
            context: AgentContext::new(),
            tools: ToolRegistry::default(),
            config: AgentConfig::default(),
            state: AgentState::Idle,
            stats: AgentStats::default(),
            tool_context: ToolContext::default(),
        }
    }

    /// Create a new agent with custom configuration.
    #[must_use]
    pub fn with_config(llm_client: Box<Client>, config: AgentConfig) -> Self {
        Self {
            llm_client,
            context: AgentContext::new(),
            tools: ToolRegistry::default(),
            config,
            state: AgentState::Idle,
            stats: AgentStats::default(),
            tool_context: ToolContext::default(),
        }
    }

    /// Create a new agent with tools registry.
    #[must_use]
    pub fn with_tools(llm_client: Box<Client>, tools: ToolRegistry) -> Self {
        Self {
            llm_client,
            context: AgentContext::new(),
            tools,
            config: AgentConfig::default(),
            state: AgentState::Idle,
            stats: AgentStats::default(),
            tool_context: ToolContext::default(),
        }
    }

    /// Get the current agent state.
    #[must_use]
    pub fn state(&self) -> &AgentState {
        &self.state
    }

    /// Get execution statistics.
    #[must_use]
    pub fn stats(&self) -> &AgentStats {
        &self.stats
    }

    /// Get reference to the tool registry.
    #[must_use]
    pub fn tools(&self) -> &ToolRegistry {
        &self.tools
    }

    /// Get mutable reference to the tool registry.
    pub fn tools_mut(&mut self) -> &mut ToolRegistry {
        &mut self.tools
    }

    /// Set the tool context for tool executions.
    pub fn set_tool_context(&mut self, context: ToolContext) {
        self.tool_context = context;
    }

    /// Run the agent on a user query.
    ///
    /// # Errors
    ///
    /// Returns an error if the LLM call fails or tool execution fails.
    pub async fn run(&mut self, query: &str) -> Result<AgentResponse, AgentError> {
        self.state = AgentState::Thinking;
        self.stats = AgentStats::default();

        // Add user message to context
        self.context.add_user_message(query);

        // ReAct loop
        let result = self.react_loop(query).await;

        // Set final state
        self.state = match &result {
            Ok(_) => AgentState::Completed,
            Err(e) => AgentState::Error(e.to_string()),
        };

        result
    }

    /// Run a single reasoning iteration.
    async fn react_loop(&mut self, _initial_query: &str) -> Result<AgentResponse, AgentError> {
        let mut iterations = 0u32;
        let mut final_answer = String::new();
        let mut executed_sql = None;

        while iterations < self.config.max_iterations {
            iterations += 1;
            self.stats.iterations += 1;
            self.state = AgentState::Thinking;

            // Serialize context to JSON for LLM
            let context_json = serde_json::to_value(&self.context)
                .map_err(|e| AgentError::SerializationError {
                    message: e.to_string(),
                })?;

            // Get LLM decision
            let decision_value = self
                .llm_client
                .generate_decision(&context_json)
                .await
                .map_err(|e| AgentError::LlmError {
                    message: e.to_string(),
                })?;

            // Parse decision
            let decision = parse_decision(&decision_value)
                .map_err(|e| AgentError::InvalidToolCall { details: e })?;

            // Process decision
            match decision {
                AgentDecision::Reasoning { thought } => {
                    // Add reasoning as assistant message
                    self.context.add_assistant_message(&thought);

                    if self.config.verbose_reasoning {
                        tracing::info!("Thought: {}", thought);
                    }
                }

                AgentDecision::ToolCall(call) => {
                    self.state = AgentState::ExecutingTool;

                    // Execute tool
                    let tool_result = self.execute_tool(&call).await?;

                    // Add tool result to context
                    self.context.add_tool_message(&tool_result.result.to_string(), &call.name);

                    if let Some(sql) = extract_sql(&tool_result.result) {
                        executed_sql = Some(sql);
                    }

                    self.stats.tool_calls += 1;
                }

                AgentDecision::FinalAnswer(answer) => {
                    final_answer = answer.clone();
                    self.context.add_assistant_message(&answer);
                    break;
                }
            }
        }

        if final_answer.is_empty() {
            return Err(AgentError::MaxIterationsExceeded {
                iterations: self.config.max_iterations,
            });
        }

        Ok(AgentResponse {
            answer: final_answer,
            executed_sql,
            iterations,
            success: true,
            error: None,
            state: AgentState::Completed,
        })
    }

    /// Execute a tool call.
    async fn execute_tool(&mut self, call: &ToolCall) -> Result<ToolResult, AgentError> {
        let start = std::time::Instant::now();

        let result = self
            .tools
            .execute(&call.name, &call.arguments, &self.tool_context)
            .await
            .map_err(|e| AgentError::ToolExecutionFailed {
                tool_name: call.name.clone(),
                reason: e.to_string(),
            })?;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(ToolResult {
            call_id: call.call_id.clone(),
            tool: call.name.clone(),
            result,
            success: true,
            error: None,
            duration_ms,
        })
    }

    /// Reset the agent to initial state.
    pub fn reset(&mut self) {
        self.context.clear();
        self.state = AgentState::Idle;
        self.stats = AgentStats::default();
    }

    /// Set the database schema in context.
    pub fn set_schema(&mut self, schema: String) {
        self.context.set_database_schema(schema);
    }

    /// Get provider info from the LLM client.
    #[must_use]
    pub fn provider_info(&self) -> String {
        format!("{:?}", self.llm_client.provider_info())
    }
}

/// Parse a decision from JSON value.
fn parse_decision(value: &Value) -> Result<AgentDecision, String> {
    let decision_type = value
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'type' field")?;

    match decision_type {
        "reasoning" => {
            let thought = value
                .get("thought")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'thought' field")?
                .to_string();
            Ok(AgentDecision::Reasoning { thought })
        }
        "tool_call" => {
            let name = value
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'name' field")?
                .to_string();
            let arguments = value
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| Value::Object(serde_json::Map::new()));
            let call_id = value
                .get("call_id")
                .and_then(|v| v.as_str())
                .unwrap_or("default")
                .to_string();
            Ok(AgentDecision::ToolCall(ToolCall {
                name,
                arguments,
                call_id,
            }))
        }
        "final_answer" => {
            let answer = value
                .get("answer")
                .and_then(|v| v.as_str())
                .ok_or("Missing 'answer' field")?
                .to_string();
            Ok(AgentDecision::FinalAnswer(answer))
        }
        _ => Err(format!("Unknown decision type: {}", decision_type)),
    }
}

/// Extract SQL from a tool result if present.
fn extract_sql(result: &serde_json::Value) -> Option<String> {
    result
        .as_object()
        .and_then(|obj| obj.get("sql"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decision::AgentDecision;
    use postgres_agent_llm::provider::ProviderInfo;

    // Mock LLM client for testing
    #[derive(Debug, Default)]
    struct MockLlmClient;

    #[async_trait::async_trait]
    impl LlmClient for MockLlmClient {
        async fn complete(&self, _prompt: &str) -> Result<String, LlmError> {
            Ok("Mock completion".to_string())
        }

        async fn generate_decision(&self, _context_json: &Value) -> Result<Value, LlmError> {
            Ok(serde_json::json!({
                "type": "final_answer",
                "answer": "Mock response"
            }))
        }

        async fn generate_structured<T: serde::de::DeserializeOwned + std::fmt::Debug>(
            &self,
            _prompt: &str,
            _schema: &T,
        ) -> Result<T, LlmError> {
            unimplemented!()
        }

        fn provider_info(&self) -> ProviderInfo {
            ProviderInfo {
                provider: "Mock".to_string(),
                model: "mock".to_string(),
            }
        }
    }

    #[tokio::test]
    async fn test_agent_run() {
        let client = Box::new(MockLlmClient);
        let mut agent = PostgresAgent::new(client);

        let result = agent.run("Test query").await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.success);
        assert_eq!(response.answer, "Mock response");
    }

    #[test]
    fn test_config_builder() {
        let config = AgentConfigBuilder::new()
            .max_iterations(20)
            .safety_level(SafetyLevel::ReadOnly)
            .require_confirmation(false)
            .build();

        assert_eq!(config.max_iterations, 20);
        assert_eq!(config.safety_level, SafetyLevel::ReadOnly);
        assert!(!config.require_confirmation);
    }

    #[test]
    fn test_agent_response_error() {
        let response = AgentResponse::error("test error".to_string(), 1);
        assert!(!response.success);
        assert_eq!(response.error, Some("test error".to_string()));
    }

    #[test]
    fn test_parse_decision() {
        let json = serde_json::json!({
            "type": "final_answer",
            "answer": "Test answer"
        });
        let decision = parse_decision(&json);
        assert!(decision.is_ok());
        if let Ok(AgentDecision::FinalAnswer(ans)) = decision {
            assert_eq!(ans, "Test answer");
        }
    }
}
