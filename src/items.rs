use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CommandExecutionStatus {
    InProgress,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PatchChangeKind {
    Add,
    Delete,
    Update,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PatchApplyStatus {
    Completed,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum McpToolCallStatus {
    InProgress,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CommandExecutionItem {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub command: String,
    pub aggregated_output: String,
    pub exit_code: Option<i32>,
    pub status: CommandExecutionStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FileUpdateChange {
    pub path: String,
    pub kind: PatchChangeKind,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FileChangeItem {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub changes: Vec<FileUpdateChange>,
    pub status: PatchApplyStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct McpToolCallResult {
    pub content: Vec<Value>,
    pub structured_content: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct McpToolCallError {
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct McpToolCallItem {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub server: String,
    pub tool: String,
    pub arguments: Value,
    pub result: Option<McpToolCallResult>,
    pub error: Option<McpToolCallError>,
    pub status: McpToolCallStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AgentMessageItem {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReasoningItem {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WebSearchItem {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub query: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ErrorItem {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TodoItem {
    pub text: String,
    pub completed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TodoListItem {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub items: Vec<TodoItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ThreadItem {
    #[serde(rename = "agent_message")]
    AgentMessage { id: String, text: String },
    #[serde(rename = "reasoning")]
    Reasoning { id: String, text: String },
    #[serde(rename = "command_execution")]
    CommandExecution {
        id: String,
        command: String,
        aggregated_output: String,
        exit_code: Option<i32>,
        status: CommandExecutionStatus,
    },
    #[serde(rename = "file_change")]
    FileChange {
        id: String,
        changes: Vec<FileUpdateChange>,
        status: PatchApplyStatus,
    },
    #[serde(rename = "mcp_tool_call")]
    McpToolCall {
        id: String,
        server: String,
        tool: String,
        arguments: Value,
        result: Option<McpToolCallResult>,
        error: Option<McpToolCallError>,
        status: McpToolCallStatus,
    },
    #[serde(rename = "web_search")]
    WebSearch { id: String, query: String },
    #[serde(rename = "todo_list")]
    TodoList { id: String, items: Vec<TodoItem> },
    #[serde(rename = "error")]
    Error { id: String, message: String },
}
