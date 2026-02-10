pub mod actions;
pub mod polling;
pub mod session;

use actions::{open_session as open_session_action, stop_session as stop_session_action};
use polling::{detect_and_enrich_sessions, start_polling, Session};
use session::{extract_messages, parse_all_entries, MessageType};
use serde::Serialize;
use std::time::Duration;
use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
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

    // Fast path: search for the JSONL file directly across all project directories
    let entries = std::fs::read_dir(&claude_projects_dir)
        .map_err(|e| format!("Failed to read projects directory: {}", e))?;

    let session_filename = format!("{}.jsonl", session_id);

    for entry in entries.flatten() {
        let project_path = entry.path();
        if !project_path.is_dir() {
            continue;
        }

        // Check if this project contains the session file directly
        let session_file = project_path.join(&session_filename);
        if session_file.exists() {
            // Found it - parse the full session file for conversation view
            let entries = parse_all_entries(&session_file)
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

    Err(format!("Session {} not found in any project directory", session_id))
}

/// Stop a session by process ID
#[tauri::command]
async fn stop_session(app: AppHandle, pid: u32) -> Result<(), String> {
    // Stop the session
    stop_session_action(pid)?;

    // Wait a brief moment for the process to terminate
    std::thread::sleep(Duration::from_millis(300));

    // Emit updated sessions immediately so UI reflects the change
    if let Ok(sessions) = detect_and_enrich_sessions() {
        let _ = app.emit("sessions-updated", &sessions);
    }

    Ok(())
}

/// Open a session in its parent application
#[tauri::command]
async fn open_session(pid: u32, project_path: String) -> Result<(), String> {
    open_session_action(pid, project_path)
}

/// Rename a session title
#[tauri::command]
async fn rename_session(app: AppHandle, session_id: String, new_name: String) -> Result<(), String> {
    let mut custom_titles = session::CustomTitles::load();
    custom_titles.set(session_id, new_name);
    custom_titles.save()?;

    // Emit updated sessions immediately
    if let Ok(sessions) = detect_and_enrich_sessions() {
        let _ = app.emit("sessions-updated", &sessions);
    }

    Ok(())
}

/// Get the terminal title for a session (iTerm2 only, macOS)
#[tauri::command]
async fn get_terminal_title(pid: u32) -> Result<Option<String>, String> {
    #[cfg(target_os = "macos")]
    {
        Ok(actions::get_iterm2_session_title(pid))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = pid;
        Ok(None)
    }
}

/// Show and focus the main application window
#[tauri::command]
async fn show_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;

        // Hide the popover if it's open
        if let Some(popover) = app.get_webview_window("popover") {
            let _ = popover.hide();
        }
    }
    Ok(())
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
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // Start the polling loop when the app starts
            start_polling(app.handle().clone());

            // Create the tray icon with click handler
            let app_handle = app.handle().clone();
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .icon_as_template(true)
                .tooltip("c9watch")
                .on_tray_icon_event(move |_tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        // Show/focus main window on tray click
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
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
            stop_session,
            open_session,
            rename_session,
            get_terminal_title,
            show_main_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
