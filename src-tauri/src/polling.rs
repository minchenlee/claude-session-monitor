use crate::session::{
    determine_status, parse_last_n_entries, parse_sessions_index, SessionDetector, SessionStatus,
};
use serde::Serialize;
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

    let mut sessions = Vec::new();

    for detected in detected_sessions {
        // Get session ID - if not found, skip this session
        let session_id = match &detected.session_id {
            Some(id) => id.clone(),
            None => {
                eprintln!(
                    "Skipping session with PID {} - no session ID found",
                    detected.pid
                );
                continue;
            }
        };

        // Parse sessions-index.json to get basic info
        let index_path = detected.project_path.join("sessions-index.json");
        let sessions_index = match parse_sessions_index(&index_path) {
            Ok(index) => index,
            Err(e) => {
                eprintln!(
                    "Failed to parse sessions-index.json for session {}: {}",
                    session_id, e
                );
                continue;
            }
        };

        // Find the matching entry in the index
        let session_entry = sessions_index
            .entries
            .iter()
            .find(|entry| entry.session_id == session_id);

        let (first_prompt, message_count, modified, git_branch) = match session_entry {
            Some(entry) => (
                entry.first_prompt.clone(),
                entry.message_count,
                entry.modified.clone(),
                Some(entry.git_branch.clone()),
            ),
            None => {
                eprintln!(
                    "Session {} not found in sessions-index.json",
                    session_id
                );
                continue;
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

        sessions.push(Session {
            id: session_id,
            pid: detected.pid,
            project_name: detected.project_name,
            project_path: detected.project_path.to_string_lossy().to_string(),
            git_branch,
            first_prompt,
            message_count,
            modified,
            status,
        });
    }

    Ok(sessions)
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
