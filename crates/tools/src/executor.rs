//! Tool executor.

use crate::{ToolContext, ToolError, ToolRegistry};

/// Tool executor.
#[derive(Debug)]
pub struct ToolExecutor {
    /// Tool registry.
    registry: ToolRegistry,
}

impl ToolExecutor {
    /// Create a new tool executor.
    #[must_use]
    pub fn new(registry: ToolRegistry) -> Self {
        Self { registry }
    }

    /// Execute a tool call.
    #[allow(dead_code)]
    pub async fn execute(
        &self,
        name: &str,
        args: &serde_json::Value,
        timeout: Option<std::time::Duration>,
    ) -> Result<serde_json::Value, ToolError> {
        let ctx = ToolContext { timeout };
        self.registry.execute(name, args, &ctx).await
    }
}
