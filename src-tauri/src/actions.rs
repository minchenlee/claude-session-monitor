use std::process::Command;

/// Open a session by focusing its terminal or IDE window
///
/// This finds the parent application of the Claude process and activates it.
/// Works with Terminal, iTerm2, Zed, VS Code, Cursor, and other applications.
pub fn open_session(pid: u32, project_path: String) -> Result<(), String> {
    // Find the parent application by walking up the process tree
    let app_name = find_parent_app(pid)?;

    // Extract project name from path for window matching
    let project_name = std::path::Path::new(&project_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    eprintln!("[open_session] App: {}, Project: {}, Path: {}", app_name, project_name, project_path);

    // Try to use app-specific CLI to open/focus the correct window
    if let Some(cli_path) = get_app_cli(&app_name) {
        eprintln!("[open_session] Using CLI: {} to open: {}", cli_path, project_path);

        // VS Code family uses -r flag to reuse window, -g to not open new if exists
        let output = if app_name == "Visual Studio Code" || app_name == "Cursor" || app_name == "Windsurf" {
            Command::new(&cli_path)
                .arg("-r")  // Reuse existing window
                .arg("-g")  // Don't grab focus for new file (but we want focus)
                .arg(&project_path)
                .output()
        } else {
            // Zed and others just take the path
            Command::new(&cli_path)
                .arg(&project_path)
                .output()
        };

        match output {
            Ok(out) => {
                if out.status.success() {
                    eprintln!("[open_session] CLI succeeded");
                    return Ok(());
                } else {
                    let error = String::from_utf8_lossy(&out.stderr);
                    eprintln!("[open_session] CLI error: {}", error);
                }
            }
            Err(e) => {
                eprintln!("[open_session] Failed to run CLI: {}", e);
            }
        }
    }

    // Platform-specific fallback to activate the app
    activate_app_fallback(&app_name)?;

    Ok(())
}

/// Platform-specific fallback to activate/focus an application
#[cfg(target_os = "macos")]
fn activate_app_fallback(app_name: &str) -> Result<(), String> {
    let script = format!(r#"tell application "{}" to activate"#, app_name);
    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("Failed to execute osascript: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        eprintln!("[open_session] AppleScript error: {}", error);
    }
    Ok(())
}

/// Linux fallback: try xdg-open or xdotool to raise window
#[cfg(target_os = "linux")]
fn activate_app_fallback(app_name: &str) -> Result<(), String> {
    // Try xdotool to find and activate a window by name
    let search_name = match app_name {
        "Visual Studio Code" => "Visual Studio Code",
        "Cursor" => "Cursor",
        "Windsurf" => "Windsurf",
        "Zed" => "Zed",
        "Sublime Text" => "Sublime Text",
        _ => app_name,
    };

    let output = Command::new("xdotool")
        .arg("search")
        .arg("--name")
        .arg(search_name)
        .arg("windowactivate")
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                eprintln!("[open_session] xdotool activated window for: {}", search_name);
                return Ok(());
            }
            eprintln!("[open_session] xdotool failed, window not found for: {}", search_name);
        }
        Err(_) => {
            eprintln!("[open_session] xdotool not available");
        }
    }

    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn activate_app_fallback(_app_name: &str) -> Result<(), String> {
    Ok(())
}

/// Get the CLI path for an application if available
#[cfg(target_os = "macos")]
fn get_app_cli(app_name: &str) -> Option<String> {
    let cli_paths: &[(&str, &[&str])] = &[
        ("Zed", &["/Applications/Zed.app/Contents/MacOS/cli"]),
        ("Visual Studio Code", &[
            "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code",
            "/usr/local/bin/code",
        ]),
        ("Cursor", &[
            "/Applications/Cursor.app/Contents/Resources/app/bin/cursor",
            "/Applications/Cursor.app/Contents/Resources/app/bin/code",
            "/usr/local/bin/cursor",
        ]),
        ("Windsurf", &[
            "/Applications/Windsurf.app/Contents/Resources/app/bin/windsurf",
            "/Applications/Windsurf.app/Contents/Resources/app/bin/code",
        ]),
    ];

    for (name, paths) in cli_paths {
        if *name == app_name {
            for path in *paths {
                if std::path::Path::new(path).exists() {
                    return Some(path.to_string());
                }
            }
        }
    }

    None
}

/// Get the CLI path for an application on Linux
#[cfg(target_os = "linux")]
fn get_app_cli(app_name: &str) -> Option<String> {
    let cli_paths: &[(&str, &[&str])] = &[
        ("Zed", &[
            "/usr/bin/zed",
            "/usr/local/bin/zed",
        ]),
        ("Visual Studio Code", &[
            "/usr/bin/code",
            "/usr/local/bin/code",
            "/snap/bin/code",
        ]),
        ("Cursor", &[
            "/usr/bin/cursor",
            "/usr/local/bin/cursor",
        ]),
        ("Windsurf", &[
            "/usr/bin/windsurf",
            "/usr/local/bin/windsurf",
        ]),
        ("Sublime Text", &[
            "/usr/bin/subl",
            "/usr/local/bin/subl",
            "/snap/bin/subl",
        ]),
    ];

    for (name, paths) in cli_paths {
        if *name == app_name {
            for path in *paths {
                if std::path::Path::new(path).exists() {
                    return Some(path.to_string());
                }
            }
        }
    }

    // Fallback: try to find the binary via `which`
    let bin_name = match app_name {
        "Zed" => Some("zed"),
        "Visual Studio Code" => Some("code"),
        "Cursor" => Some("cursor"),
        "Windsurf" => Some("windsurf"),
        "Sublime Text" => Some("subl"),
        _ => None,
    };

    if let Some(name) = bin_name {
        if let Ok(output) = Command::new("which").arg(name).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(path);
                }
            }
        }
    }

    None
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn get_app_cli(_app_name: &str) -> Option<String> {
    None
}

/// Find the parent GUI application for a given process ID
fn find_parent_app(pid: u32) -> Result<String, String> {
    let mut current_pid = pid;

    eprintln!("[open_session] Starting with PID: {}", pid);

    // Walk up the process tree to find a GUI application
    for i in 0..20 {
        // Get the command/path for current process
        let comm_output = Command::new("ps")
            .arg("-o")
            .arg("comm=")
            .arg("-p")
            .arg(current_pid.to_string())
            .output()
            .map_err(|e| format!("Failed to execute ps: {}", e))?;

        let comm = String::from_utf8_lossy(&comm_output.stdout).trim().to_string();
        eprintln!("[open_session] Step {}: PID {} -> comm: {}", i, current_pid, comm);

        // Check if this is a known GUI application
        if let Some(app_name) = get_app_name(&comm) {
            eprintln!("[open_session] Found app: {}", app_name);
            return Ok(app_name.to_string());
        }

        // Get parent PID
        let ppid_output = Command::new("ps")
            .arg("-o")
            .arg("ppid=")
            .arg("-p")
            .arg(current_pid.to_string())
            .output()
            .map_err(|e| format!("Failed to execute ps: {}", e))?;

        let ppid_str = String::from_utf8_lossy(&ppid_output.stdout).trim().to_string();
        let ppid: u32 = ppid_str.parse().unwrap_or(1);
        eprintln!("[open_session] Parent PID: {}", ppid);

        // Move to parent
        if ppid <= 1 {
            eprintln!("[open_session] Reached root, checking current comm one more time");
            // Check current process one more time before giving up
            if let Some(app_name) = get_app_name(&comm) {
                eprintln!("[open_session] Found app at root: {}", app_name);
                return Ok(app_name.to_string());
            }
            break;
        }
        current_pid = ppid;
    }

    // Platform-specific fallback
    #[cfg(target_os = "macos")]
    {
        eprintln!("[open_session] Falling back to Terminal");
        Ok("Terminal".to_string())
    }
    #[cfg(target_os = "linux")]
    {
        eprintln!("[open_session] Falling back to xterm");
        Ok("xterm".to_string())
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Ok("Terminal".to_string())
    }
}

/// Map process command names to application names
fn get_app_name(comm: &str) -> Option<&'static str> {
    // macOS: Check for .app bundle paths (e.g., /Applications/Zed.app/Contents/MacOS/zed)
    #[cfg(target_os = "macos")]
    {
        let comm_lower = comm.to_lowercase();
        if comm_lower.contains(".app/") || comm_lower.contains(".app") {
            if comm_lower.contains("zed.app") {
                return Some("Zed");
            }
            if comm_lower.contains("visual studio code.app") || comm_lower.contains("code.app") {
                return Some("Visual Studio Code");
            }
            if comm_lower.contains("cursor.app") {
                return Some("Cursor");
            }
            if comm_lower.contains("windsurf.app") {
                return Some("Windsurf");
            }
            if comm_lower.contains("iterm.app") || comm_lower.contains("iterm2.app") {
                return Some("iTerm");
            }
            if comm_lower.contains("terminal.app") {
                return Some("Terminal");
            }
            if comm_lower.contains("alacritty.app") {
                return Some("Alacritty");
            }
            if comm_lower.contains("kitty.app") {
                return Some("kitty");
            }
            if comm_lower.contains("warp.app") {
                return Some("Warp");
            }
            if comm_lower.contains("hyper.app") {
                return Some("Hyper");
            }
            if comm_lower.contains("sublime text.app") {
                return Some("Sublime Text");
            }
        }
    }

    // Extract the base name from the path
    let base_name = comm.rsplit('/').next().unwrap_or(comm);

    match base_name.to_lowercase().as_str() {
        // Terminals (cross-platform names)
        "terminal" => Some("Terminal"),
        "iterm2" | "iterm" => Some("iTerm"),
        "alacritty" => Some("Alacritty"),
        "kitty" => Some("kitty"),
        "warp" => Some("Warp"),
        "hyper" => Some("Hyper"),
        "gnome-terminal-server" | "gnome-terminal" => Some("GNOME Terminal"),
        "konsole" => Some("Konsole"),
        "xfce4-terminal" => Some("Xfce Terminal"),
        "xterm" => Some("xterm"),
        "foot" => Some("foot"),
        "wezterm" | "wezterm-gui" => Some("WezTerm"),
        "tilix" => Some("Tilix"),
        "terminator" => Some("Terminator"),
        "ghostty" => Some("Ghostty"),

        // IDEs
        "zed" | "zed-editor" => Some("Zed"),
        "code" | "code helper" | "electron" => Some("Visual Studio Code"),
        "cursor" => Some("Cursor"),
        "windsurf" => Some("Windsurf"),

        // Other editors
        "sublime_text" | "subl" => Some("Sublime Text"),
        "atom" => Some("Atom"),

        _ => None,
    }
}

/// Stop a session by sending SIGTERM to the process
///
/// This gracefully terminates the Claude process by sending a SIGTERM signal.
/// SIGTERM is preferred over SIGINT as Claude Code may trap SIGINT for its own use.
pub fn stop_session(pid: u32) -> Result<(), String> {
    eprintln!("[stop_session] Stopping PID: {}", pid);

    // First try SIGTERM (signal 15) - graceful termination
    let output = Command::new("kill")
        .arg("-15") // SIGTERM
        .arg(pid.to_string())
        .output()
        .map_err(|e| format!("Failed to execute kill command: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        eprintln!("[stop_session] SIGTERM failed: {}", error);

        // If SIGTERM fails, the process might not exist or we don't have permission
        return Err(format!("Failed to stop process {}: {}", pid, error));
    }

    eprintln!("[stop_session] SIGTERM sent successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stop_session_invalid_pid() {
        // Try to stop a non-existent process
        let result = stop_session(999999);
        assert!(result.is_err());
    }

    #[test]
    #[ignore] // This test requires manual verification
    fn test_open_session() {
        // Use current process PID for testing
        let result = open_session(std::process::id(), "/tmp".to_string());
        println!("Result: {:?}", result);
    }

    #[test]
    fn test_get_app_name_terminals() {
        assert_eq!(get_app_name("alacritty"), Some("Alacritty"));
        assert_eq!(get_app_name("kitty"), Some("kitty"));
        assert_eq!(get_app_name("/usr/bin/kitty"), Some("kitty"));
        assert_eq!(get_app_name("ghostty"), Some("Ghostty"));
    }

    #[test]
    fn test_get_app_name_ides() {
        assert_eq!(get_app_name("code"), Some("Visual Studio Code"));
        assert_eq!(get_app_name("zed"), Some("Zed"));
        assert_eq!(get_app_name("cursor"), Some("Cursor"));
    }
}
