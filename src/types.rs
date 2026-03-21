use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Default)]
pub struct StdinData {
    pub model: Option<Model>,
    pub context_window: Option<ContextWindow>,
    pub transcript_path: Option<String>,
    pub cwd: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Model {
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ContextWindow {
    pub current_usage: Option<CurrentUsage>,
    pub context_window_size: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct CurrentUsage {
    pub input_tokens: Option<u64>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub separator: Option<String>,
    pub colors: Option<bool>,
    pub path_levels: Option<u8>,
    pub context_warning: Option<u8>,
    pub context_critical: Option<u8>,
}

impl Config {
    pub fn separator(&self) -> &str {
        self.separator.as_deref().unwrap_or("  ")
    }

    pub fn colors(&self) -> bool {
        self.colors.unwrap_or(true)
    }

    pub fn path_levels(&self) -> u8 {
        self.path_levels.unwrap_or(1).clamp(1, 3)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct State {
    pub version: u32,
    pub byte_offset: u64,
    pub inode: u64,
    pub file_size: u64,
    pub tools: HashMap<String, ToolEntry>,
    pub agents: HashMap<String, AgentEntry>,
    pub todos: Vec<TodoItem>,
    pub tasks: HashMap<String, TaskItem>,
    pub session_start: Option<i64>,
    pub git_index_mtime: Option<i64>,
    pub usage_cache: Option<UsageCache>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolEntry {
    pub name: String,
    pub target: Option<String>,
    pub completed: bool,
    pub error: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentEntry {
    pub subagent_type: Option<String>,
    pub model: Option<String>,
    pub description: Option<String>,
    pub start_time: Option<i64>,
    pub completed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TodoItem {
    pub content: String,
    pub completed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskItem {
    pub status: TaskStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UsageCache {
    pub fetched_at: i64,
    pub success: bool,
    pub usage_5h: Option<f64>,
    pub usage_7d: Option<f64>,
    pub reset_5h: Option<i64>,
}

#[derive(Debug, Default)]
pub struct TranscriptData {
    pub tools: HashMap<String, ToolEntry>,
    pub agents: HashMap<String, AgentEntry>,
    pub todos: Vec<TodoItem>,
    pub tasks: HashMap<String, TaskItem>,
    pub session_start: Option<i64>,
}
