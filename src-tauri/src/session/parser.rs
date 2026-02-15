use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Represents the sessions-index.json file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsIndex {
    pub version: u32,
    pub entries: Vec<SessionIndexEntry>,
}

/// Individual session entry from sessions-index.json
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionIndexEntry {
    pub session_id: String,
    pub full_path: PathBuf,
    pub file_mtime: u64,
    pub first_prompt: String,
    pub summary: Option<String>,
    pub message_count: u32,
    pub created: String,
    pub modified: String,
    pub git_branch: String,
    pub project_path: PathBuf,
    pub is_sidechain: bool,
}

/// A single line entry from a session JSONL file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SessionEntry {
    User {
        #[serde(flatten)]
        base: SessionEntryBase,
        message: UserMessage,
    },
    Assistant {
        #[serde(flatten)]
        base: SessionEntryBase,
        message: AssistantMessage,
    },
    #[serde(rename = "file-history-snapshot")]
    FileHistorySnapshot {
        #[serde(rename = "messageId")]
        message_id: String,
        snapshot: serde_json::Value,
        #[serde(rename = "isSnapshotUpdate")]
        is_snapshot_update: bool,
    },
    Summary {
        summary: String,
        #[serde(rename = "leafUuid")]
        leaf_uuid: String,
    },
    #[serde(other)]
    Unknown,
}

/// Common fields shared across session entries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionEntryBase {
    pub uuid: String,
    pub timestamp: String,
    pub session_id: Option<String>,
    pub cwd: Option<PathBuf>,
    pub version: Option<String>,
    pub git_branch: Option<String>,
    pub parent_uuid: Option<String>,
    pub is_sidechain: Option<bool>,
    pub slug: Option<String>,
}

/// User message structure
///
/// In Claude Code's JSONL format, user message content can be either:
/// - A plain string (for actual user prompts)
/// - An array of content blocks (for tool results sent back to Claude)
#[derive(Debug, Clone, Serialize)]
pub struct UserMessage {
    pub role: String,
    pub content: String,
    /// Whether this user entry is a tool result rather than an actual user prompt
    pub is_tool_result: bool,
}

impl<'de> Deserialize<'de> for UserMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde_json::Value;

        let value = Value::deserialize(deserializer)?;
        let role = value
            .get("role")
            .and_then(|r| r.as_str())
            .unwrap_or("user")
            .to_string();

        let content_value = value.get("content");

        let (content, is_tool_result) = match content_value {
            Some(Value::String(s)) => (s.clone(), false),
            Some(Value::Array(arr)) => {
                let mut parts = Vec::new();
                for item in arr {
                    match item.get("type").and_then(|t| t.as_str()) {
                        Some("tool_result") => {
                            if let Some(content) = item.get("content") {
                                match content {
                                    Value::String(s) => parts.push(s.clone()),
                                    Value::Array(inner) => {
                                        for block in inner {
                                            if let Some(text) =
                                                block.get("text").and_then(|t| t.as_str())
                                            {
                                                parts.push(text.to_string());
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Some("text") => {
                            if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                parts.push(text.to_string());
                            }
                        }
                        _ => {}
                    }
                }
                let text = if parts.is_empty() {
                    "[tool result]".to_string()
                } else {
                    parts.join("\n")
                };
                (text, true)
            }
            _ => (String::new(), false),
        };

        Ok(UserMessage {
            role,
            content,
            is_tool_result,
        })
    }
}

/// Assistant message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub model: String,
    pub id: String,
    pub role: String,
    pub content: Vec<MessageContent>,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: Option<Usage>,
}

/// Content block within an assistant message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContent {
    Text {
        text: String,
    },
    Thinking {
        thinking: String,
        signature: Option<String>,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        is_error: Option<bool>,
    },
    #[serde(other)]
    Unknown,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub cache_creation_input_tokens: Option<u32>,
    pub cache_read_input_tokens: Option<u32>,
}

/// Parse a sessions-index.json file
pub fn parse_sessions_index<P: AsRef<Path>>(path: P) -> Result<SessionsIndex, String> {
    let file = File::open(path.as_ref())
        .map_err(|e| format!("Failed to open sessions-index.json: {}", e))?;

    let reader = BufReader::new(file);
    serde_json::from_reader(reader)
        .map_err(|e| format!("Failed to parse sessions-index.json: {}", e))
}

/// Read the last N lines from a JSONL file efficiently
///
/// This function uses a reverse-reading strategy to avoid loading
/// the entire file into memory for large files.
pub fn read_last_n_lines<P: AsRef<Path>>(path: P, n: usize) -> Result<Vec<String>, String> {
    let file =
        File::open(path.as_ref()).map_err(|e| format!("Failed to open JSONL file: {}", e))?;

    let metadata = file
        .metadata()
        .map_err(|e| format!("Failed to read file metadata: {}", e))?;

    let file_size = metadata.len();

    // If file is empty, return empty vec
    if file_size == 0 {
        return Ok(vec![]);
    }

    // For small files, just read everything
    if file_size < 10_000 {
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader
            .lines()
            .filter_map(|line| line.ok())
            .filter(|line| !line.trim().is_empty())
            .collect();

        let start = if lines.len() > n { lines.len() - n } else { 0 };
        return Ok(lines[start..].to_vec());
    }

    // For larger files, read from the end
    // Estimate: average line is ~1KB, so read last n*1KB + buffer
    let chunk_size = (n * 1024 * 2).min(file_size as usize);
    let mut file = file;

    file.seek(SeekFrom::End(-(chunk_size as i64)))
        .map_err(|e| format!("Failed to seek in file: {}", e))?;

    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| !line.trim().is_empty())
        .collect();

    let start = if lines.len() > n { lines.len() - n } else { 0 };
    Ok(lines[start..].to_vec())
}

/// Parse JSONL lines into SessionEntry structs
pub fn parse_jsonl_entries(lines: Vec<String>) -> Vec<SessionEntry> {
    lines
        .iter()
        .filter_map(|line| serde_json::from_str::<SessionEntry>(line).ok())
        .collect()
}

/// Parse the last N entries from a session JSONL file
pub fn parse_last_n_entries<P: AsRef<Path>>(
    path: P,
    n: usize,
) -> Result<Vec<SessionEntry>, String> {
    let lines = read_last_n_lines(path, n)?;
    Ok(parse_jsonl_entries(lines))
}

/// Parse all entries from a session JSONL file
pub fn parse_all_entries<P: AsRef<Path>>(path: P) -> Result<Vec<SessionEntry>, String> {
    let file =
        File::open(path.as_ref()).map_err(|e| format!("Failed to open JSONL file: {}", e))?;

    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .filter(|line| !line.trim().is_empty())
        .collect();

    Ok(parse_jsonl_entries(lines))
}

/// Get all user and assistant messages from session entries
pub fn extract_messages(entries: &[SessionEntry]) -> Vec<(String, MessageType, String)> {
    let mut messages = Vec::new();

    for entry in entries {
        match entry {
            SessionEntry::User { base, message } => {
                if message.is_tool_result {
                    // Tool result entries should be shown as ToolResult, not User
                    messages.push((
                        base.timestamp.clone(),
                        MessageType::ToolResult,
                        message.content.clone(),
                    ));
                } else {
                    messages.push((
                        base.timestamp.clone(),
                        MessageType::User,
                        message.content.clone(),
                    ));
                }
            }
            SessionEntry::Assistant { base, message } => {
                for content in &message.content {
                    match content {
                        MessageContent::Text { text } => {
                            messages.push((
                                base.timestamp.clone(),
                                MessageType::Assistant,
                                text.clone(),
                            ));
                        }
                        MessageContent::Thinking { thinking, .. } => {
                            messages.push((
                                base.timestamp.clone(),
                                MessageType::Thinking,
                                thinking.clone(),
                            ));
                        }
                        MessageContent::ToolUse { id, name, input } => {
                            let tool_desc = format!(
                                "[{}] {} - {}",
                                name,
                                id,
                                serde_json::to_string_pretty(input).unwrap_or_default()
                            );
                            messages.push((
                                base.timestamp.clone(),
                                MessageType::ToolUse,
                                tool_desc,
                            ));
                        }
                        MessageContent::ToolResult {
                            tool_use_id,
                            content,
                            is_error,
                        } => {
                            let result_type = if is_error.unwrap_or(false) {
                                "Error"
                            } else {
                                "Result"
                            };
                            let tool_desc =
                                format!("[{}] {}: {}", result_type, tool_use_id, content);
                            messages.push((
                                base.timestamp.clone(),
                                MessageType::ToolResult,
                                tool_desc,
                            ));
                        }
                        MessageContent::Unknown => {}
                    }
                }
            }
            _ => {}
        }
    }

    messages
}

/// Message type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    User,
    Assistant,
    Thinking,
    ToolUse,
    ToolResult,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_user_message() {
        let json = r#"{
            "type": "user",
            "uuid": "test-uuid",
            "timestamp": "2026-01-08T15:23:03.096Z",
            "sessionId": "test-session",
            "cwd": "/Users/test",
            "message": {
                "role": "user",
                "content": "Hello Claude"
            }
        }"#;

        let entry: Result<SessionEntry, _> = serde_json::from_str(json);
        assert!(entry.is_ok());

        if let Ok(SessionEntry::User { base, message }) = entry {
            assert_eq!(base.uuid, "test-uuid");
            assert_eq!(message.content, "Hello Claude");
        } else {
            panic!("Failed to parse user message");
        }
    }

    #[test]
    fn test_parse_assistant_message() {
        let json = r#"{
            "type": "assistant",
            "uuid": "test-uuid",
            "timestamp": "2026-01-08T15:23:03.096Z",
            "message": {
                "model": "claude-opus-4-5-20251101",
                "id": "msg_test",
                "role": "assistant",
                "content": [
                    {
                        "type": "text",
                        "text": "Hello user"
                    }
                ],
                "stop_reason": null,
                "stop_sequence": null
            }
        }"#;

        let entry: Result<SessionEntry, _> = serde_json::from_str(json);
        assert!(entry.is_ok());

        if let Ok(SessionEntry::Assistant { base, message }) = entry {
            assert_eq!(base.uuid, "test-uuid");
            assert_eq!(message.model, "claude-opus-4-5-20251101");
            assert_eq!(message.content.len(), 1);
        } else {
            panic!("Failed to parse assistant message");
        }
    }

    #[test]
    fn test_parse_tool_use() {
        let json = r#"{
            "type": "assistant",
            "uuid": "test-uuid",
            "timestamp": "2026-01-08T15:23:03.096Z",
            "message": {
                "model": "claude-opus-4-5-20251101",
                "id": "msg_test",
                "role": "assistant",
                "content": [
                    {
                        "type": "tool_use",
                        "id": "toolu_123",
                        "name": "Read",
                        "input": {"file_path": "/path/to/file.txt"}
                    }
                ],
                "stop_reason": "tool_use"
            }
        }"#;

        let entry: Result<SessionEntry, _> = serde_json::from_str(json);
        assert!(entry.is_ok());

        if let Ok(SessionEntry::Assistant { message, .. }) = entry {
            assert_eq!(message.content.len(), 1);
            if let MessageContent::ToolUse { id, name, .. } = &message.content[0] {
                assert_eq!(id, "toolu_123");
                assert_eq!(name, "Read");
            } else {
                panic!("Expected ToolUse content");
            }
        } else {
            panic!("Failed to parse tool use");
        }
    }

    #[test]
    fn test_parse_user_message_with_tool_result_content() {
        // In Claude Code's JSONL, tool result messages have content as an array
        let json = r#"{
            "type": "user",
            "uuid": "test-uuid",
            "timestamp": "2026-01-08T15:23:03.096Z",
            "sessionId": "test-session",
            "message": {
                "role": "user",
                "content": [
                    {
                        "type": "tool_result",
                        "tool_use_id": "toolu_123",
                        "content": "command output here"
                    }
                ]
            }
        }"#;

        let entry: Result<SessionEntry, _> = serde_json::from_str(json);
        assert!(
            entry.is_ok(),
            "Should parse user message with array content"
        );

        if let Ok(SessionEntry::User { message, .. }) = entry {
            assert!(message.content.contains("command output here"));
        } else {
            panic!("Expected User entry");
        }
    }

    #[test]
    fn test_parse_user_message_with_nested_tool_result() {
        // tool_result content can also be an array of content blocks
        let json = r#"{
            "type": "user",
            "uuid": "test-uuid",
            "timestamp": "2026-01-08T15:23:03.096Z",
            "sessionId": "test-session",
            "message": {
                "role": "user",
                "content": [
                    {
                        "type": "tool_result",
                        "tool_use_id": "toolu_456",
                        "content": [
                            {"type": "text", "text": "file contents here"}
                        ]
                    }
                ]
            }
        }"#;

        let entry: Result<SessionEntry, _> = serde_json::from_str(json);
        assert!(
            entry.is_ok(),
            "Should parse user message with nested array tool_result content"
        );

        if let Ok(SessionEntry::User { message, .. }) = entry {
            assert!(message.content.contains("file contents here"));
        } else {
            panic!("Expected User entry");
        }
    }

    #[test]
    fn test_parse_progress_entry() {
        // Progress entries should parse as Unknown (not cause errors)
        let json = r#"{
            "type": "progress",
            "uuid": "test-uuid",
            "timestamp": "2026-01-08T15:23:03.096Z",
            "data": {"type": "bash_progress"},
            "toolUseID": "toolu_123"
        }"#;

        let entry: Result<SessionEntry, _> = serde_json::from_str(json);
        assert!(entry.is_ok(), "Progress entries should parse as Unknown");
        assert!(matches!(entry.unwrap(), SessionEntry::Unknown));
    }
}
