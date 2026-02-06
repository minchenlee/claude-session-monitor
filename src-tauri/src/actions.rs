use std::io::Write;
use std::process::{Command, Stdio};

/// Open a session by focusing its terminal or IDE window
///
/// This attempts to find and focus the parent application of the Claude process.
/// On macOS, this typically means opening Terminal.app or the IDE where Claude is running.
pub fn open_session(_session_id: String) -> Result<(), String> {
    // For now, we'll use AppleScript to activate Terminal
    // In the future, we could detect the parent process and activate that instead
    let script = r#"
        tell application "Terminal"
            activate
        end tell
    "#;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| format!("Failed to execute osascript: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to activate Terminal: {}", error));
    }

    Ok(())
}

/// Stop a session by sending SIGINT to the process
///
/// This gracefully terminates the Claude process by sending a SIGINT signal,
/// which is equivalent to pressing Ctrl+C.
pub fn stop_session(pid: u32) -> Result<(), String> {
    // Use the kill command to send SIGINT (signal 2)
    let output = Command::new("kill")
        .arg("-2") // SIGINT
        .arg(pid.to_string())
        .output()
        .map_err(|e| format!("Failed to execute kill command: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to stop process {}: {}", pid, error));
    }

    Ok(())
}

/// Send a prompt to a session by spawning the Claude CLI
///
/// This function:
/// 1. Spawns `claude` with --continue and --session-id flags
/// 2. Pipes the prompt to stdin
/// 3. Returns immediately (doesn't wait for response)
pub fn send_prompt(session_id: String, prompt: String) -> Result<(), String> {
    // Spawn the claude command with --continue and --session-id
    let mut child = Command::new("claude")
        .arg("--continue")
        .arg("--session-id")
        .arg(&session_id)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn claude command: {}", e))?;

    // Get stdin handle and write the prompt
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(prompt.as_bytes())
            .map_err(|e| format!("Failed to write prompt to stdin: {}", e))?;

        stdin
            .write_all(b"\n")
            .map_err(|e| format!("Failed to write newline to stdin: {}", e))?;
    } else {
        return Err("Failed to open stdin for claude process".to_string());
    }

    // Don't wait for the process to complete - it will run in the background
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
        let result = open_session("test-session-id".to_string());
        // This should activate Terminal
        println!("Result: {:?}", result);
    }

    #[test]
    #[ignore] // This test requires a real session ID and claude CLI
    fn test_send_prompt() {
        let result = send_prompt(
            "test-session-id".to_string(),
            "Hello from the test!".to_string(),
        );
        println!("Result: {:?}", result);
    }
}
