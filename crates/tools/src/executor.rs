//! Tool executor.
//!
//! This module provides the [`ToolExecutor`] for executing tools
//! with support for both sequential and parallel execution.

use tokio::time::Instant;
use tracing::{debug, trace};

use crate::trait_def::{ToolCall, ToolResult, Tool};
use crate::{ToolContext, ToolError, ToolRegistry};

/// Tool executor with parallel execution support.
#[derive(Debug)]
pub struct ToolExecutor {
    /// Tool registry for looking up tools.
    registry: ToolRegistry,
}

impl ToolExecutor {
    /// Create a new tool executor.
    #[must_use]
    pub fn new(registry: ToolRegistry) -> Self {
        Self { registry }
    }

    /// Execute a single tool call.
    ///
    /// # Errors
    /// Returns `ToolError::NotFound` if the tool doesn't exist.
    /// Returns `ToolError::Timeout` if execution times out.
    pub async fn execute(
        &self,
        name: &str,
        args: &serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<serde_json::Value, ToolError> {
        trace!("Executing tool: {}", name);

        let tool = self.registry.get(name).ok_or_else(|| ToolError::NotFound {
            tool_name: name.to_string(),
        })?;

        let start = Instant::now();
        let result = tool.execute(args, ctx).await;
        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(value) => {
                debug!("Tool {} executed successfully in {}ms", name, duration_ms);
                Ok(value)
            }
            Err(e) => {
                debug!("Tool {} failed in {}ms: {}", name, duration_ms, e);
                Err(e)
            }
        }
    }

    /// Execute a tool call with automatic result wrapping.
    ///
    /// Wraps the result in a [`ToolResult`] with timing and success status.
    pub async fn execute_with_result(
        &self,
        call: &ToolCall,
        ctx: &ToolContext,
    ) -> ToolResult {
        let start = Instant::now();
        let result = self.execute(&call.name, &call.arguments, ctx).await;
        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(value) => ToolResult::success(
                call.call_id.clone(),
                call.name.clone(),
                value,
                duration_ms,
            ),
            Err(e) => ToolResult::failure(
                call.call_id.clone(),
                call.name.clone(),
                e.to_string(),
                duration_ms,
            ),
        }
    }

    /// Execute multiple tool calls in parallel.
    ///
    /// Uses Tokio's `join_all` to execute all tool calls concurrently.
    /// Returns results in the same order as the input calls.
    pub async fn execute_parallel(
        &self,
        calls: &[ToolCall],
        ctx: &ToolContext,
    ) -> Vec<ToolResult> {
        if calls.is_empty() {
            return Vec::new();
        }

        let futures: Vec<_> = calls
            .iter()
            .map(|call| self.execute_with_result(call, ctx))
            .collect();

        tracing::debug!("Executing {} tools in parallel", calls.len());
        futures::future::join_all(futures).await
    }

    /// Execute a batch of tool calls sequentially.
    ///
    /// Processes tool calls one at a time, stopping on error if `stop_on_error` is true.
    ///
    /// # Arguments
    /// * `calls` - List of tool calls to execute
    /// * `ctx` - Execution context
    /// * `stop_on_error` - Whether to stop execution on first error
    ///
    /// Returns results in the same order as the input calls.
    pub async fn execute_batch(
        &self,
        calls: &[ToolCall],
        ctx: &ToolContext,
        stop_on_error: bool,
    ) -> Vec<ToolResult> {
        if calls.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::with_capacity(calls.len());

        for call in calls {
            let result = self.execute_with_result(call, ctx).await;
            results.push(result.clone());

            if stop_on_error && !result.success {
                tracing::debug!("Stopping batch execution on error");
                break;
            }
        }

        results
    }

    /// Get a reference to the registry.
    #[must_use]
    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }
}
