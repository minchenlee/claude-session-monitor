pub mod actions;
pub mod polling;
pub mod session;

use actions::{open_session as open_session_action, send_prompt as send_prompt_action, stop_session as stop_session_action};
use polling::{start_polling, Session};
use session::{
    extract_messages, parse_last_n_entries, parse_sessions_index, MessageType,
};
use serde::Serialize;
use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Get all active Claude sessions
#[tauri::command]
async fn get_sessions() -> Result<Vec<Session>, String> {
    // Use the same detection logic as the polling loop
    polling::detect_and_enrich_sessions()
}

/// Get the conversation history for a specific session
#[tauri::command]
async fn get_conversation(session_id: String) -> Result<Conversation, String> {
    // Find the session's project directory
    let home_dir = dirs::home_dir().ok_or("Failed to get home directory")?;
    let claude_projects_dir = home_dir.join(".claude").join("projects");

    // Enumerate all project directories
    let entries = std::fs::read_dir(&claude_projects_dir)
        .map_err(|e| format!("Failed to read projects directory: {}", e))?;

    // Find the project containing this session
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let project_path = entry.path();

        if !project_path.is_dir() {
            continue;
        }

        // Check if this project contains our session
        let index_path = project_path.join("sessions-index.json");
        if let Ok(sessions_index) = parse_sessions_index(&index_path) {
            let has_session = sessions_index
                .entries
                .iter()
                .any(|e| e.session_id == session_id);

            if has_session {
                // Found the project - parse the session file
                let session_file = project_path.join(format!("{}.jsonl", session_id));

                if !session_file.exists() {
                    return Err(format!("Session file does not exist: {:?}", session_file));
                }

                let entries = parse_last_n_entries(&session_file, 100)
                    .map_err(|e| format!("Failed to parse session file: {}", e))?;

                let messages = extract_messages(&entries);

                // Convert to frontend format
                let conversation_messages: Vec<ConversationMessage> = messages
                    .into_iter()
                    .map(|(timestamp, msg_type, content)| ConversationMessage {
                        timestamp,
                        message_type: msg_type,
                        content,
                    })
                    .collect();

                return Ok(Conversation {
                    session_id,
                    messages: conversation_messages,
                });
            }
        }
    }

    Err(format!("Session {} not found in any project directory", session_id))
}

/// Send a prompt to a session
#[tauri::command]
async fn send_prompt(session_id: String, prompt: String) -> Result<(), String> {
    send_prompt_action(session_id, prompt)
}

/// Stop a session by process ID
#[tauri::command]
async fn stop_session(pid: u32) -> Result<(), String> {
    stop_session_action(pid)
}

/// Open a session in its parent application
#[tauri::command]
async fn open_session(session_id: String) -> Result<(), String> {
    open_session_action(session_id)
}

/// Conversation structure for the frontend
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Conversation {
    pub session_id: String,
    pub messages: Vec<ConversationMessage>,
}

/// Individual message in a conversation
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationMessage {
    pub timestamp: String,
    pub message_type: MessageType,
    pub content: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Start the polling loop when the app starts
            start_polling(app.handle().clone());

            // Create the tray icon with click handler
            let app_handle = app.handle().clone();
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .icon_as_template(true)
                .tooltip("Claude Session Monitor")
                .on_tray_icon_event(move |_tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        // Toggle window visibility on left click
                        if let Some(window) = app_handle.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_sessions,
            get_conversation,
            send_prompt,
            stop_session,
            open_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
