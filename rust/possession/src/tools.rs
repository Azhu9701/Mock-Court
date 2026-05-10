use std::collections::HashMap;
use std::sync::Arc;

use foundation::{Result, ToolDefinition, ToolCall};

const MAX_TOOL_ROUNDS: usize = 3;

pub fn parse_soul_tools(tools_json: &str) -> Vec<String> {
    if tools_json.trim().is_empty() {
        return Vec::new();
    }
    let parsed: Result<Vec<ToolDefinition>> = serde_json::from_str(tools_json)
        .map_err(|e| foundation::FoundationError::Validation(format!("Invalid tools JSON: {}", e)));
    match parsed {
        Ok(tools) => tools.into_iter().map(|t| t.function.name).collect(),
        Err(_) => Vec::new(),
    }
}

#[async_trait::async_trait]
pub trait ToolHandler: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> serde_json::Value;
    async fn execute(&self, arguments: &str) -> Result<String>;

    fn to_definition(&self) -> ToolDefinition {
        ToolDefinition {
            r#type: "function".to_string(),
            function: foundation::FunctionDef {
                name: self.name().to_string(),
                description: self.description().to_string(),
                parameters: self.parameters_schema(),
                strict: None,
            },
        }
    }
}

#[derive(Clone)]
pub struct ToolRegistry {
    handlers: Arc<HashMap<String, Arc<dyn ToolHandler>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        ToolRegistry {
            handlers: Arc::new(HashMap::new()),
        }
    }

    pub fn register(&mut self, handler: Arc<dyn ToolHandler>) {
        Arc::get_mut(&mut self.handlers)
            .expect("ToolRegistry must be mutated before sharing")
            .insert(handler.name().to_string(), handler);
    }

    pub fn get_definition(&self, name: &str) -> Option<ToolDefinition> {
        self.handlers.get(name).map(|h| h.to_definition())
    }

    pub fn get_all_definitions(&self) -> Vec<ToolDefinition> {
        self.handlers.values().map(|h| h.to_definition()).collect()
    }

    pub fn filter_definitions(&self, names: &[String]) -> Vec<ToolDefinition> {
        names
            .iter()
            .filter_map(|n| self.handlers.get(n).map(|h| h.to_definition()))
            .collect()
    }

    pub async fn execute(&self, call: &ToolCall) -> Result<String> {
        let handler = self.handlers.get(&call.function.name).ok_or_else(|| {
            foundation::FoundationError::Validation(format!(
                "Unknown tool: {}",
                call.function.name
            ))
        })?;
        handler.execute(&call.function.arguments).await
    }

    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }
}

pub fn max_tool_rounds() -> usize {
    MAX_TOOL_ROUNDS
}
