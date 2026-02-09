use std::process::Command;

/// Open a session by focusing its terminal or IDE window
///
/// This finds the parent application of the Claude process and activates it.
/// On macOS: Works with Terminal, iTerm2, Zed, VS Code, Cursor, and other applications.
/// On Windows: Works with Windows Terminal, PowerShell, CMD, VS Code, Cursor, and others.
pub fn open_session(pid: u32, project_path: String) -> Result<(), String> {
    // Find the parent application by walking up the process tree
    let app_name = find_parent_app(pid)?;

    // Extract project name from path for window matching
    let project_name = std::path::Path::new(&project_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    eprintln!(
        "[open_session] App: {}, Project: {}, Path: {}",
        app_name, project_name, project_path
    );

    // Try to use app-specific CLI to open/focus the correct window
    if let Some(cli_path) = get_app_cli(&app_name) {
        eprintln!(
            "[open_session] Using CLI: {} to open: {}",
            cli_path, project_path
        );

        let output = if app_name == "Visual Studio Code"
            || app_name == "Cursor"
            || app_name == "Windsurf"
        {
            // VS Code family uses -r flag to reuse window
            Command::new(&cli_path)
                .arg("-r")
                .arg("-g")
                .arg(&project_path)
                .output()
        } else {
            // Zed and others just take the path
            Command::new(&cli_path).arg(&project_path).output()
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

    // Platform-specific fallback to activate the parent application
    activate_app_fallback(&app_name)?;

    Ok(())
}

/// Activate an application as a fallback when CLI is not available
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

/// Activate an application as a fallback on Windows
#[cfg(target_os = "windows")]
fn activate_app_fallback(app_name: &str) -> Result<(), String> {
    // Map app names to Windows process names for activation
    let process_name = match app_name {
        "Visual Studio Code" => "Code.exe",
        "Cursor" => "Cursor.exe",
        "Windsurf" => "Windsurf.exe",
        "Windows Terminal" => "WindowsTerminal.exe",
        "PowerShell" => "pwsh.exe",
        "Command Prompt" => "cmd.exe",
        _ => {
            eprintln!(
                "[open_session] No Windows activation mapping for: {}",
                app_name
            );
            return Ok(());
        }
    };

    // Use PowerShell to bring the window to the foreground
    let ps_script = format!(
        r#"
        $proc = Get-Process -Name '{}' -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($proc) {{
            $hwnd = $proc.MainWindowHandle
            if ($hwnd -ne 0) {{
                Add-Type -TypeDefinition @'
                using System;
                using System.Runtime.InteropServices;
                public class Win32 {{
                    [DllImport("user32.dll")]
                    public static extern bool SetForegroundWindow(IntPtr hWnd);
                    [DllImport("user32.dll")]
                    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
                }}
'@
                [Win32]::ShowWindow($hwnd, 9)
                [Win32]::SetForegroundWindow($hwnd)
            }}
        }}
        "#,
        process_name.trim_end_matches(".exe")
    );

    let output = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-NonInteractive")
        .arg("-Command")
        .arg(&ps_script)
        .output()
        .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        eprintln!("[open_session] PowerShell activation error: {}", error);
    }
    Ok(())
}

/// Activate an application as a fallback on Linux
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn activate_app_fallback(app_name: &str) -> Result<(), String> {
    eprintln!(
        "[open_session] No fallback activation available for: {} on this platform",
        app_name
    );
    Ok(())
}

/// Get the CLI path for an application if available (macOS)
#[cfg(target_os = "macos")]
fn get_app_cli(app_name: &str) -> Option<String> {
    let cli_paths: &[(&str, &[&str])] = &[
        ("Zed", &["/Applications/Zed.app/Contents/MacOS/cli"]),
        (
            "Visual Studio Code",
            &[
                "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code",
                "/usr/local/bin/code",
            ],
        ),
        (
            "Cursor",
            &[
                "/Applications/Cursor.app/Contents/Resources/app/bin/cursor",
                "/Applications/Cursor.app/Contents/Resources/app/bin/code",
                "/usr/local/bin/cursor",
            ],
        ),
        (
            "Windsurf",
            &[
                "/Applications/Windsurf.app/Contents/Resources/app/bin/windsurf",
                "/Applications/Windsurf.app/Contents/Resources/app/bin/code",
            ],
        ),
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

/// Get the CLI path for an application if available (Windows)
#[cfg(target_os = "windows")]
fn get_app_cli(app_name: &str) -> Option<String> {
    let local_app_data = std::env::var("LOCALAPPDATA").ok()?;
    let program_files = std::env::var("PROGRAMFILES").ok()?;

    let cli_paths: Vec<(&str, Vec<String>)> = vec![
        (
            "Visual Studio Code",
            vec![
                format!(
                    r"{}\Programs\Microsoft VS Code\bin\code.cmd",
                    local_app_data
                ),
                format!(r"{}\Microsoft VS Code\bin\code.cmd", program_files),
                // Scoop / winget install locations
                format!(
                    r"{}\Microsoft VS Code\bin\code.cmd",
                    std::env::var("PROGRAMFILES(X86)").unwrap_or_default()
                ),
            ],
        ),
        (
            "Cursor",
            vec![
                format!(r"{}\Programs\cursor\resources\app\bin\cursor.cmd", local_app_data),
                format!(r"{}\Cursor\resources\app\bin\cursor.cmd", local_app_data),
            ],
        ),
        (
            "Windsurf",
            vec![
                format!(r"{}\Programs\Windsurf\bin\windsurf.cmd", local_app_data),
                format!(r"{}\Windsurf\bin\windsurf.cmd", program_files),
            ],
        ),
        (
            "Zed",
            vec![
                format!(r"{}\Zed\zed.exe", local_app_data),
                format!(r"{}\Programs\Zed\zed.exe", local_app_data),
            ],
        ),
    ];

    for (name, paths) in &cli_paths {
        if *name == app_name {
            for path in paths {
                if std::path::Path::new(path).exists() {
                    return Some(path.clone());
                }
            }
        }
    }

    None
}

/// Get the CLI path for an application if available (Linux)
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn get_app_cli(app_name: &str) -> Option<String> {
    let cli_paths: &[(&str, &[&str])] = &[
        ("Visual Studio Code", &["/usr/bin/code", "/usr/local/bin/code", "/snap/bin/code"]),
        ("Cursor", &["/usr/bin/cursor", "/usr/local/bin/cursor"]),
        ("Zed", &["/usr/bin/zed", "/usr/local/bin/zed"]),
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

/// Find the parent GUI application for a given process ID (macOS/Linux)
#[cfg(unix)]
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

        let comm = String::from_utf8_lossy(&comm_output.stdout)
            .trim()
            .to_string();
        eprintln!(
            "[open_session] Step {}: PID {} -> comm: {}",
            i, current_pid, comm
        );

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

        let ppid_str = String::from_utf8_lossy(&ppid_output.stdout)
            .trim()
            .to_string();
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

    eprintln!("[open_session] Falling back to Terminal");
    // Fallback to Terminal
    Ok("Terminal".to_string())
}

/// Find the parent GUI application for a given process ID (Windows)
///
/// On Windows, we use WMIC or PowerShell to walk the process tree.
/// Claude Code on Windows typically runs under:
///   - Windows Terminal (WindowsTerminal.exe)
///   - PowerShell (pwsh.exe / powershell.exe)
///   - Command Prompt (cmd.exe)
///   - VS Code integrated terminal (Code.exe)
///   - Cursor integrated terminal (Cursor.exe)
#[cfg(target_os = "windows")]
fn find_parent_app(pid: u32) -> Result<String, String> {
    let mut current_pid = pid;

    eprintln!("[open_session] Starting with PID: {}", pid);

    // Walk up the process tree using WMIC
    for i in 0..20 {
        // Get process name and parent PID using WMIC
        let output = Command::new("wmic")
            .arg("process")
            .arg("where")
            .arg(format!("ProcessId={}", current_pid))
            .arg("get")
            .arg("Name,ParentProcessId")
            .arg("/format:csv")
            .output()
            .map_err(|e| format!("Failed to execute wmic: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!(
            "[open_session] Step {}: PID {} -> wmic output: {}",
            i,
            current_pid,
            stdout.trim()
        );

        // Parse CSV output: Node,Name,ParentProcessId
        let mut process_name = String::new();
        let mut parent_pid: u32 = 0;

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("Node") {
                continue;
            }
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 3 {
                process_name = parts[1].trim().to_string();
                parent_pid = parts[2].trim().parse().unwrap_or(0);
            }
        }

        if process_name.is_empty() {
            break;
        }

        eprintln!(
            "[open_session] Process: {} (PPID: {})",
            process_name, parent_pid
        );

        // Check if this is a known GUI application
        if let Some(app_name) = get_app_name(&process_name) {
            eprintln!("[open_session] Found app: {}", app_name);
            return Ok(app_name.to_string());
        }

        // Move to parent
        if parent_pid <= 1 || parent_pid == current_pid {
            break;
        }
        current_pid = parent_pid;
    }

    eprintln!("[open_session] Falling back to Windows Terminal");
    Ok("Windows Terminal".to_string())
}

/// Map process command names to application names
#[cfg(target_os = "macos")]
fn get_app_name(comm: &str) -> Option<&'static str> {
    let comm_lower = comm.to_lowercase();

    // Check for .app bundle paths first (e.g., /Applications/Zed.app/Contents/MacOS/zed)
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

    // Extract the base name from the path
    let base_name = comm.rsplit('/').next().unwrap_or(comm);

    match base_name.to_lowercase().as_str() {
        // Terminals
        "terminal" => Some("Terminal"),
        "iterm2" | "iterm" => Some("iTerm"),
        "alacritty" => Some("Alacritty"),
        "kitty" => Some("kitty"),
        "warp" => Some("Warp"),
        "hyper" => Some("Hyper"),
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

/// Map process names to application names (Windows)
#[cfg(target_os = "windows")]
fn get_app_name(comm: &str) -> Option<&'static str> {
    let name_lower = comm.to_lowercase();

    // Strip .exe suffix for matching
    let base = name_lower.trim_end_matches(".exe");

    match base {
        // Terminals
        "windowsterminal" => Some("Windows Terminal"),
        "pwsh" | "powershell" => Some("PowerShell"),
        "cmd" => Some("Command Prompt"),
        "alacritty" => Some("Alacritty"),
        "kitty" => Some("kitty"),
        "hyper" => Some("Hyper"),
        "wezterm" | "wezterm-gui" => Some("WezTerm"),
        // IDEs
        "code" => Some("Visual Studio Code"),
        "cursor" => Some("Cursor"),
        "windsurf" => Some("Windsurf"),
        "zed" => Some("Zed"),
        // Other editors
        "sublime_text" => Some("Sublime Text"),
        _ => None,
    }
}

/// Map process names to application names (Linux)
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn get_app_name(comm: &str) -> Option<&'static str> {
    let base_name = comm.rsplit('/').next().unwrap_or(comm);

    match base_name.to_lowercase().as_str() {
        // Terminals
        "alacritty" => Some("Alacritty"),
        "kitty" => Some("kitty"),
        "hyper" => Some("Hyper"),
        "wezterm" | "wezterm-gui" => Some("WezTerm"),
        "gnome-terminal-server" | "gnome-terminal" => Some("GNOME Terminal"),
        "konsole" => Some("Konsole"),
        "xterm" => Some("XTerm"),
        // IDEs
        "code" => Some("Visual Studio Code"),
        "cursor" => Some("Cursor"),
        "windsurf" => Some("Windsurf"),
        "zed" | "zed-editor" => Some("Zed"),
        // Other editors
        "sublime_text" | "subl" => Some("Sublime Text"),
        _ => None,
    }
}

/// Stop a session by sending a termination signal to the process
///
/// On Unix: Sends SIGTERM (signal 15) for graceful termination.
/// On Windows: Uses taskkill for graceful process termination.
#[cfg(unix)]
pub fn stop_session(pid: u32) -> Result<(), String> {
    eprintln!("[stop_session] Stopping PID: {}", pid);

    // Send SIGTERM (signal 15) - graceful termination
    let output = Command::new("kill")
        .arg("-15") // SIGTERM
        .arg(pid.to_string())
        .output()
        .map_err(|e| format!("Failed to execute kill command: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        eprintln!("[stop_session] SIGTERM failed: {}", error);
        return Err(format!("Failed to stop process {}: {}", pid, error));
    }

    eprintln!("[stop_session] SIGTERM sent successfully");
    Ok(())
}

/// Stop a session on Windows using taskkill
///
/// First attempts graceful termination, then force-kills if needed.
#[cfg(target_os = "windows")]
pub fn stop_session(pid: u32) -> Result<(), String> {
    eprintln!("[stop_session] Stopping PID: {} on Windows", pid);

    // First try graceful termination with taskkill (sends WM_CLOSE)
    let output = Command::new("taskkill")
        .arg("/PID")
        .arg(pid.to_string())
        .output()
        .map_err(|e| format!("Failed to execute taskkill: {}", e))?;

    if output.status.success() {
        eprintln!("[stop_session] Graceful termination succeeded");
        return Ok(());
    }

    // If graceful fails, try force kill
    eprintln!("[stop_session] Graceful termination failed, trying force kill");
    let output = Command::new("taskkill")
        .arg("/F")
        .arg("/PID")
        .arg(pid.to_string())
        .output()
        .map_err(|e| format!("Failed to execute taskkill /F: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        eprintln!("[stop_session] Force kill failed: {}", error);
        return Err(format!("Failed to stop process {}: {}", pid, error));
    }

    eprintln!("[stop_session] Force kill succeeded");
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
    fn test_get_app_name_known_apps() {
        // Test that known applications are recognized
        #[cfg(target_os = "windows")]
        {
            assert_eq!(get_app_name("Code.exe"), Some("Visual Studio Code"));
            assert_eq!(get_app_name("WindowsTerminal.exe"), Some("Windows Terminal"));
            assert_eq!(get_app_name("pwsh.exe"), Some("PowerShell"));
            assert_eq!(get_app_name("cmd.exe"), Some("Command Prompt"));
            assert_eq!(get_app_name("Cursor.exe"), Some("Cursor"));
        }

        #[cfg(target_os = "macos")]
        {
            assert_eq!(get_app_name("code"), Some("Visual Studio Code"));
            assert_eq!(get_app_name("terminal"), Some("Terminal"));
            assert_eq!(get_app_name("zed"), Some("Zed"));
        }
    }

    #[test]
    fn test_get_app_name_unknown() {
        assert_eq!(get_app_name("unknown_process"), None);
    }
}
