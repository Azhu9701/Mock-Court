use std::path::{Path, PathBuf};
use std::time::Duration;

use async_trait::async_trait;
use possession::tools::ToolHandler;

// ── Shared sandbox infrastructure ───────────────────────────────────────

fn safe_path(working_dir: &Path, requested: &str) -> foundation::Result<PathBuf> {
    if requested.trim().is_empty() {
        return Err(foundation::FoundationError::Validation(
            "Path cannot be empty".into(),
        ));
    }
    let path = if Path::new(requested).is_absolute() {
        PathBuf::from(requested)
    } else {
        working_dir.join(requested)
    };
    // Try to canonicalize; if file doesn't exist yet, canonicalize the parent
    let canonical = if path.exists() {
        path.canonicalize().map_err(|e| {
            foundation::FoundationError::Validation(format!("Path error: {} ({})", requested, e))
        })?
    } else {
        let parent = path.parent().unwrap_or(Path::new("."));
        let canon_parent = parent.canonicalize().map_err(|e| {
            foundation::FoundationError::Validation(format!(
                "Parent directory not found: {} ({})",
                parent.display(),
                e
            ))
        })?;
        canon_parent.join(path.file_name().unwrap_or_default())
    };
    let wd_canonical = working_dir.canonicalize().map_err(|e| {
        foundation::FoundationError::Validation(format!("Working directory error: {}", e))
    })?;
    if !canonical.starts_with(&wd_canonical) {
        return Err(foundation::FoundationError::Validation(format!(
            "Access denied: path '{}' is outside working directory",
            requested
        )));
    }
    Ok(canonical)
}

const MAX_OUTPUT_BYTES: usize = 50 * 1024; // 50KB
const MAX_FILE_READ_BYTES: usize = 1024 * 1024; // 1MB

fn truncate_output(s: &str) -> String {
    if s.len() <= MAX_OUTPUT_BYTES {
        s.to_string()
    } else {
        format!(
            "{}...\n[Output truncated at {} bytes]",
            &s[..MAX_OUTPUT_BYTES],
            MAX_OUTPUT_BYTES
        )
    }
}

// ── ReadFile ────────────────────────────────────────────────────────────

pub struct ReadFileTool {
    working_dir: PathBuf,
}

impl ReadFileTool {
    pub fn new(working_dir: PathBuf) -> Self {
        Self { working_dir }
    }
}

#[async_trait]
impl ToolHandler for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file. Returns content with line numbers. \
         Supports optional offset and limit for large files."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path relative to working directory"
                },
                "offset": {
                    "type": "integer",
                    "description": "Starting line number (0-based, optional)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Max lines to read (default 2000)"
                }
            },
            "required": ["path"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, arguments: &str) -> foundation::Result<String> {
        let args: serde_json::Value = serde_json::from_str(arguments)?;
        let path = safe_path(&self.working_dir, args["path"].as_str().unwrap_or(""))?;

        let metadata = std::fs::metadata(&path)?;
        if metadata.len() > MAX_FILE_READ_BYTES as u64 {
            return Err(foundation::FoundationError::Validation(format!(
                "File too large: {} bytes (max {})",
                metadata.len(),
                MAX_FILE_READ_BYTES
            )));
        }

        let content = std::fs::read_to_string(&path)?;
        let offset = args["offset"].as_u64().unwrap_or(0) as usize;
        let limit = args["limit"].as_u64().unwrap_or(2000) as usize;

        let selected: Vec<String> = content
            .lines()
            .skip(offset)
            .take(limit)
            .enumerate()
            .map(|(i, line)| format!("{}\t{}", offset + i + 1, line))
            .collect();

        Ok(truncate_output(&selected.join("\n")))
    }
}

// ── WriteFile ───────────────────────────────────────────────────────────

pub struct WriteFileTool {
    working_dir: PathBuf,
}

impl WriteFileTool {
    pub fn new(working_dir: PathBuf) -> Self {
        Self { working_dir }
    }
}

#[async_trait]
impl ToolHandler for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Write content to a file. Creates if doesn't exist, overwrites if it does. \
         Creates parent directories as needed."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path relative to working directory"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write"
                }
            },
            "required": ["path", "content"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, arguments: &str) -> foundation::Result<String> {
        let args: serde_json::Value = serde_json::from_str(arguments)?;
        let path = safe_path(&self.working_dir, args["path"].as_str().unwrap_or(""))?;
        let content = args["content"].as_str().unwrap_or("");

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, content)?;
        Ok(format!(
            "File written: {} ({} bytes)",
            path.display(),
            content.len()
        ))
    }
}

// ── EditFile ────────────────────────────────────────────────────────────

pub struct EditFileTool {
    working_dir: PathBuf,
}

impl EditFileTool {
    pub fn new(working_dir: PathBuf) -> Self {
        Self { working_dir }
    }
}

#[async_trait]
impl ToolHandler for EditFileTool {
    fn name(&self) -> &str {
        "edit_file"
    }

    fn description(&self) -> &str {
        "Edit a file by replacing exact text. old_str must match exactly one location."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "File path"
                },
                "old_str": {
                    "type": "string",
                    "description": "Exact text to find (must be unique in file)"
                },
                "new_str": {
                    "type": "string",
                    "description": "Replacement text"
                }
            },
            "required": ["path", "old_str", "new_str"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, arguments: &str) -> foundation::Result<String> {
        let args: serde_json::Value = serde_json::from_str(arguments)?;
        let path = safe_path(&self.working_dir, args["path"].as_str().unwrap_or(""))?;
        let old_str = args["old_str"].as_str().unwrap_or("");
        let new_str = args["new_str"].as_str().unwrap_or("");

        let content = std::fs::read_to_string(&path)?;
        let count = content.matches(old_str).count();
        if count == 0 {
            return Err(foundation::FoundationError::Validation(
                "old_str not found in file".into(),
            ));
        }
        if count > 1 {
            return Err(foundation::FoundationError::Validation(format!(
                "old_str matches {} locations; must be unique. Provide more context.",
                count
            )));
        }
        let new_content = content.replacen(old_str, new_str, 1);
        std::fs::write(&path, &new_content)?;
        Ok(format!("File edited: {}", path.display()))
    }
}

// ── BashCommand ─────────────────────────────────────────────────────────

const BASH_TIMEOUT_SECS: u64 = 30;

const BLOCKED_COMMANDS: &[&str] = &[
    "rm -rf /",
    "rm -rf /*",
    "mkfs",
    "dd if=",
    "> /dev/sd",
    "chmod -R 777 /",
    "chown -R /",
    ":(){ :|:& };:",
    "shutdown",
    "reboot",
    "halt",
    "poweroff",
];

fn is_command_blocked(cmd: &str) -> bool {
    let lower = cmd.trim().to_lowercase();
    if lower.starts_with("sudo ") {
        return true;
    }
    for blocked in BLOCKED_COMMANDS {
        if lower.contains(&blocked.to_lowercase()) {
            return true;
        }
    }
    false
}

pub struct BashCommandTool {
    working_dir: PathBuf,
}

impl BashCommandTool {
    pub fn new(working_dir: PathBuf) -> Self {
        Self { working_dir }
    }
}

#[async_trait]
impl ToolHandler for BashCommandTool {
    fn name(&self) -> &str {
        "bash_command"
    }

    fn description(&self) -> &str {
        "Execute a shell command. Runs in the project directory with a 30-second timeout."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Shell command to execute"
                }
            },
            "required": ["command"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, arguments: &str) -> foundation::Result<String> {
        let args: serde_json::Value = serde_json::from_str(arguments)?;
        let command = args["command"].as_str().unwrap_or("");

        if is_command_blocked(command) {
            return Err(foundation::FoundationError::Validation(format!(
                "Command blocked for safety: {}",
                command
            )));
        }

        let result = tokio::time::timeout(
            Duration::from_secs(BASH_TIMEOUT_SECS),
            tokio::process::Command::new("bash")
                .arg("-c")
                .arg(command)
                .current_dir(&self.working_dir)
                .output(),
        )
        .await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let exit_code = output.status.code().unwrap_or(-1);
                let mut result = String::new();
                if !stdout.is_empty() {
                    result.push_str(&format!("stdout:\n{}\n", truncate_output(&stdout)));
                }
                if !stderr.is_empty() {
                    result.push_str(&format!("stderr:\n{}\n", truncate_output(&stderr)));
                }
                result.push_str(&format!("exit_code: {}", exit_code));
                Ok(result)
            }
            Ok(Err(e)) => Err(foundation::FoundationError::Io(std::io::Error::other(
                format!("Failed to execute command: {}", e),
            ))),
            Err(_) => Err(foundation::FoundationError::Validation(format!(
                "Command timed out after {} seconds",
                BASH_TIMEOUT_SECS
            ))),
        }
    }
}

// ── GlobSearch ──────────────────────────────────────────────────────────

pub struct GlobSearchTool {
    working_dir: PathBuf,
}

impl GlobSearchTool {
    pub fn new(working_dir: PathBuf) -> Self {
        Self { working_dir }
    }
}

#[async_trait]
impl ToolHandler for GlobSearchTool {
    fn name(&self) -> &str {
        "glob_search"
    }

    fn description(&self) -> &str {
        "Find files matching a glob pattern. Examples: '**/*.rs', 'src/**/*.ts', '*.toml'"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Glob pattern to match"
                },
                "path": {
                    "type": "string",
                    "description": "Directory to search in (default: working directory)"
                }
            },
            "required": ["pattern"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, arguments: &str) -> foundation::Result<String> {
        let args: serde_json::Value = serde_json::from_str(arguments)?;
        let pattern = args["pattern"].as_str().unwrap_or("");
        let search_dir = match args["path"].as_str() {
            Some(p) => safe_path(&self.working_dir, p)?,
            None => self.working_dir.clone(),
        };

        let full_pattern = search_dir.join(pattern).to_string_lossy().to_string();
        let glob_paths = glob::glob(&full_pattern).map_err(|e| {
            foundation::FoundationError::Validation(format!("Invalid glob pattern: {}", e))
        })?;

        let mut results: Vec<String> = Vec::new();
        for entry in glob_paths.take(200) {
            if let Ok(path) = entry {
                if path.is_file() {
                    if let Ok(rel) = path.strip_prefix(&self.working_dir) {
                        results.push(rel.display().to_string());
                    } else {
                        results.push(path.display().to_string());
                    }
                }
            }
        }

        if results.is_empty() {
            Ok(format!("No files found matching pattern: {}", pattern))
        } else {
            Ok(format!(
                "Found {} files:\n{}",
                results.len(),
                results.join("\n")
            ))
        }
    }
}

// ── GrepSearch ──────────────────────────────────────────────────────────

pub struct GrepSearchTool {
    working_dir: PathBuf,
}

impl GrepSearchTool {
    pub fn new(working_dir: PathBuf) -> Self {
        Self { working_dir }
    }
}

#[async_trait]
impl ToolHandler for GrepSearchTool {
    fn name(&self) -> &str {
        "grep_search"
    }

    fn description(&self) -> &str {
        "Search file contents using regex. Returns matching lines with file paths and line numbers."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Regex pattern to search for"
                },
                "path": {
                    "type": "string",
                    "description": "File or directory to search (default: working directory)"
                },
                "include": {
                    "type": "string",
                    "description": "File glob filter, e.g. '*.rs'"
                }
            },
            "required": ["pattern"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, arguments: &str) -> foundation::Result<String> {
        let args: serde_json::Value = serde_json::from_str(arguments)?;
        let pattern = args["pattern"].as_str().unwrap_or("");
        let search_path = match args["path"].as_str() {
            Some(p) => safe_path(&self.working_dir, p)?,
            None => self.working_dir.clone(),
        };
        let include_filter = args["include"].as_str();

        let re = regex::Regex::new(pattern).map_err(|e| {
            foundation::FoundationError::Validation(format!("Invalid regex: {}", e))
        })?;

        let mut results: Vec<String> = Vec::new();
        let max_results = 100;

        let include_glob = include_filter
            .map(|inc| {
                glob::Pattern::new(inc).map_err(|e| {
                    foundation::FoundationError::Validation(format!(
                        "Invalid include pattern: {}",
                        e
                    ))
                })
            })
            .transpose()?;

        let walker = walkdir::WalkDir::new(&search_path)
            .max_depth(10)
            .follow_links(false);

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();

            // Skip binary files
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if matches!(
                    ext_str.as_str(),
                    "exe" | "bin" | "so" | "dylib" | "dll" | "png" | "jpg" | "jpeg"
                        | "gif" | "pdf" | "zip" | "tar" | "gz" | "woff" | "ttf"
                        | "pyc" | "class" | "o" | "a"
                ) {
                    continue;
                }
            }

            // Skip hidden directories
            if path
                .components()
                .any(|c| c.as_os_str().to_string_lossy().starts_with('.'))
            {
                continue;
            }

            // Apply include filter
            if let Some(ref glob_pat) = include_glob {
                let file_name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy();
                if !glob_pat.matches(&file_name) {
                    continue;
                }
            }

            // Search file content
            if let Ok(content) = std::fs::read_to_string(path) {
                for (line_num, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        let rel = path
                            .strip_prefix(&self.working_dir)
                            .unwrap_or(path);
                        results.push(format!("{}:{}:  {}", rel.display(), line_num + 1, line));
                        if results.len() >= max_results {
                            break;
                        }
                    }
                }
            }
            if results.len() >= max_results {
                break;
            }
        }

        if results.is_empty() {
            Ok(format!("No matches found for pattern: {}", pattern))
        } else {
            Ok(format!(
                "Found {} matches:\n{}",
                results.len(),
                results.join("\n")
            ))
        }
    }
}

// ── ClaudeCodeTool ──────────────────────────────────────────────────────

const CLAUDE_CODE_TIMEOUT_SECS: u64 = 300; // 5 minutes

pub struct ClaudeCodeTool {
    working_dir: PathBuf,
}

impl ClaudeCodeTool {
    pub fn new(working_dir: PathBuf) -> Self {
        Self { working_dir }
    }
}

#[async_trait]
impl ToolHandler for ClaudeCodeTool {
    fn name(&self) -> &str {
        "claude_code"
    }

    fn description(&self) -> &str {
        "Delegate a coding task to Claude Code, an autonomous coding agent. \
         Claude Code can read/write files, run commands, search code, and make complex multi-file changes. \
         Use this for tasks that require deep code understanding, multi-step refactoring, \
         or when you need a dedicated coding agent to handle implementation details. \
         Provide a clear, specific task description."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "task": {
                    "type": "string",
                    "description": "Clear description of the coding task for Claude Code to execute"
                },
                "allowed_tools": {
                    "type": "string",
                    "description": "Optional: restrict which tools Claude Code can use, e.g. 'Bash(git *),Edit,Read'"
                }
            },
            "required": ["task"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, arguments: &str) -> foundation::Result<String> {
        let args: serde_json::Value = serde_json::from_str(arguments)?;
        let task = args["task"]
            .as_str()
            .ok_or_else(|| foundation::FoundationError::Validation("Missing task parameter".into()))?;
        let allowed_tools = args["allowed_tools"].as_str();

        let mut cmd = tokio::process::Command::new("claude");
        cmd.arg("-p")
            .arg("--output-format")
            .arg("json")
            .arg("--verbose")
            .arg(task)
            .current_dir(&self.working_dir)
            .env("CLAUDE_CODE_ENTRYPOINT", "soul-banner-tool");

        if let Some(tools) = allowed_tools {
            cmd.arg("--allowedTools").arg(tools);
        }

        let result = tokio::time::timeout(
            Duration::from_secs(CLAUDE_CODE_TIMEOUT_SECS),
            cmd.output(),
        )
        .await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let exit_code = output.status.code().unwrap_or(-1);

                if exit_code != 0 && stdout.is_empty() {
                    return Err(foundation::FoundationError::Validation(format!(
                        "Claude Code exited with code {}: {}",
                        exit_code,
                        truncate_output(&stderr)
                    )));
                }

                // Try to parse JSON output for structured result
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    let result_text = json["result"]
                        .as_str()
                        .or_else(|| json["content"].as_str())
                        .unwrap_or(&stdout);
                    let mut output = truncate_output(result_text);
                    if !stderr.is_empty() {
                        output.push_str(&format!("\n\nstderr:\n{}", truncate_output(&stderr)));
                    }
                    Ok(output)
                } else {
                    // Plain text output
                    let mut output = truncate_output(&stdout);
                    if !stderr.is_empty() {
                        output.push_str(&format!("\n\nstderr:\n{}", truncate_output(&stderr)));
                    }
                    Ok(output)
                }
            }
            Ok(Err(e)) => Err(foundation::FoundationError::Io(std::io::Error::other(
                format!("Failed to run Claude Code: {}", e),
            ))),
            Err(_) => Err(foundation::FoundationError::Validation(format!(
                "Claude Code timed out after {} seconds",
                CLAUDE_CODE_TIMEOUT_SECS
            ))),
        }
    }
}
