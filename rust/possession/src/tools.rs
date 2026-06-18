use std::collections::HashMap;
use std::sync::Arc;

use foundation::{Result, ToolDefinition, ToolCall};

const DEFAULT_MAX_TOOL_ROUNDS: usize = 5;
const CODING_MAX_TOOL_ROUNDS: usize = 20;

pub fn parse_soul_tools(tools_str: &str) -> Vec<String> {
    if tools_str.trim().is_empty() {
        return Vec::new();
    }
    // Try JSON format first (legacy)
    if let Ok(tools) = serde_json::from_str::<Vec<ToolDefinition>>(tools_str) {
        return tools.into_iter().map(|t| t.function.name).collect();
    }
    // Comma-separated names: "Read, Bash, Glob, Grep, Write, WebFetch"
    tools_str
        .split(',')
        .map(|s| resolve_tool_name(s.trim()))
        .filter(|s| !s.is_empty())
        .collect()
}

/// Map soul tool short names to registry handler names
pub fn resolve_tool_name(soul_name: &str) -> String {
    match soul_name {
        "Read" => "read_file".to_string(),
        "Write" => "write_file".to_string(),
        "Edit" => "edit_file".to_string(),
        "Bash" => "bash_command".to_string(),
        "Glob" => "glob_search".to_string(),
        "Grep" => "grep_search".to_string(),
        "ClaudeCode" | "claude_code" => "claude_code".to_string(),
        "WebSearch" | "WebFetch" | "Search" => "web_search".to_string(),
        other => other.to_string(),
    }
}

pub fn max_tool_rounds_for_tools(tool_names: &[String]) -> usize {
    let has_coding_tools = tool_names.iter().any(|n| {
        matches!(
            n.as_str(),
            "read_file" | "write_file" | "edit_file" | "bash_command" | "glob_search" | "grep_search" | "claude_code"
        )
    });
    if has_coding_tools {
        CODING_MAX_TOOL_ROUNDS
    } else {
        DEFAULT_MAX_TOOL_ROUNDS
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
    DEFAULT_MAX_TOOL_ROUNDS
}
