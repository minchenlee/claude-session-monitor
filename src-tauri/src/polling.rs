use crate::session::{
    determine_status, parse_last_n_entries, parse_sessions_index, SessionDetector, SessionStatus,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// Combined session information for the frontend
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub pid: u32,
    pub project_name: String,
    pub project_path: String,
    pub git_branch: Option<String>,
    pub first_prompt: String,
    pub message_count: u32,
    pub modified: String,
    pub status: SessionStatus,
}

/// Start the background polling loop
///
/// This function spawns a background thread that:
/// 1. Detects active Claude sessions every 2-3 seconds
/// 2. Enriches them with status information
/// 3. Emits "sessions-updated" events to the frontend
pub fn start_polling(app: AppHandle) {
    thread::spawn(move || {
        let app_handle = Arc::new(app);
        let poll_interval = Duration::from_secs(2);

        loop {
            // Detect and enrich sessions
            match detect_and_enrich_sessions() {
                Ok(sessions) => {
                    // Emit event to frontend
                    if let Err(e) = app_handle.emit("sessions-updated", &sessions) {
                        eprintln!("Failed to emit sessions-updated event: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Error detecting sessions: {}", e);
                    // Continue polling even on error
                }
            }

            thread::sleep(poll_interval);
        }
    });
}

/// Detect sessions and enrich them with status and conversation data
pub fn detect_and_enrich_sessions() -> Result<Vec<Session>, String> {
    let mut detector = SessionDetector::new()
        .map_err(|e| format!("Failed to create session detector: {}", e))?;

    let detected_sessions = detector
        .detect_sessions()
        .map_err(|e| format!("Failed to detect sessions: {}", e))?;

    let custom_names = crate::session::CustomNames::load();
    let mut sessions = Vec::new();
    let mut seen_ids: HashSet<String> = HashSet::new();

    for detected in detected_sessions {
        // Get session ID - if not found, skip this session
        let session_id = match &detected.session_id {
            Some(id) => id.clone(),
            None => {
                continue;
            }
        };

        // Skip duplicate session IDs (same session can appear in multiple project dirs)
        if seen_ids.contains(&session_id) {
            continue;
        }
        seen_ids.insert(session_id.clone());

        // Try to parse sessions-index.json to get basic info (optional)
        let index_path = detected.project_path.join("sessions-index.json");
        let sessions_index = parse_sessions_index(&index_path).ok();

        // Find the matching entry in the index (if index exists)
        let session_entry = sessions_index
            .as_ref()
            .and_then(|index| index.entries.iter().find(|entry| entry.session_id == session_id));

        let (first_prompt, message_count, modified, git_branch) = match session_entry {
            Some(entry) => (
                entry.first_prompt.clone(),
                entry.message_count,
                entry.modified.clone(),
                Some(entry.git_branch.clone()),
            ),
            None => {
                // Session not in index or index doesn't exist - use fallback values
                let session_file_path = detected.project_path.join(format!("{}.jsonl", session_id));

                // Try to get first prompt from JSONL file
                let first_prompt = get_first_prompt_from_jsonl(&session_file_path)
                    .unwrap_or_else(|| "(Active session)".to_string());

                // Count messages in the file
                let message_count = count_messages_in_jsonl(&session_file_path);

                // Get file modification time
                let modified = std::fs::metadata(&session_file_path)
                    .and_then(|m| m.modified())
                    .ok()
                    .map(|t| {
                        let datetime: DateTime<Utc> = t.into();
                        datetime.to_rfc3339()
                    })
                    .unwrap_or_default();

                (first_prompt, message_count, modified, None)
            }
        };

        // Parse the session JSONL file to determine status
        let session_file_path = detected.project_path.join(format!("{}.jsonl", session_id));
        let status = match parse_last_n_entries(&session_file_path, 20) {
            Ok(entries) => determine_status(&entries),
            Err(e) => {
                eprintln!(
                    "Failed to parse session file for {}: {}. Using default status.",
                    session_id, e
                );
                SessionStatus::Connecting
            }
        };

        // Skip empty sessions (0 messages) - these are likely sessions where user
        // immediately used /resume to switch to a different session
        if message_count == 0 {
            continue;
        }

        // Use custom name if available, otherwise use detected project name
        let project_name = custom_names
            .get(&session_id)
            .cloned()
            .unwrap_or(detected.project_name);

        sessions.push(Session {
            id: session_id,
            pid: detected.pid,
            project_name,
            project_path: detected.cwd.to_string_lossy().to_string(),
            git_branch,
            first_prompt,
            message_count,
            modified,
            status,
        });
    }

    Ok(sessions)
}

/// Extract the first user prompt from a session JSONL file
fn get_first_prompt_from_jsonl(path: &Path) -> Option<String> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);

    for line in reader.lines().take(50) {
        if let Ok(line) = line {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
                // Check if this is a user message
                if value.get("type").and_then(|t| t.as_str()) == Some("user") {
                    // Try to get the message content
                    if let Some(message) = value.get("message") {
                        if let Some(content) = message.get("content") {
                            // Content can be a string or array
                            if let Some(text) = content.as_str() {
                                return Some(truncate_string(text, 100));
                            } else if let Some(arr) = content.as_array() {
                                // Find the first text block
                                for item in arr {
                                    if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                                        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                            return Some(truncate_string(text, 100));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Truncate a string to a maximum length (character-safe for UTF-8)
fn truncate_string(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        format!("{}...", truncated)
    }
}

/// Count user/assistant messages in a JSONL file
fn count_messages_in_jsonl(path: &Path) -> u32 {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return 0,
    };
    let reader = BufReader::new(file);
    let mut count = 0u32;

    for line in reader.lines().flatten() {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
            if let Some(msg_type) = value.get("type").and_then(|t| t.as_str()) {
                if msg_type == "user" || msg_type == "assistant" {
                    count += 1;
                }
            }
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_and_enrich_sessions() {
        // This test will only work if there are active Claude sessions
        match detect_and_enrich_sessions() {
            Ok(sessions) => {
                println!("Detected {} sessions", sessions.len());
                for session in sessions {
                    println!(
                        "Session: {} - {} (PID: {}, Status: {:?})",
                        session.id, session.project_name, session.pid, session.status
                    );
                }
            }
            Err(e) => {
                println!("Error detecting sessions: {}", e);
            }
        }
    }
}
