use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use sysinfo::{ProcessesToUpdate, System};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SessionDetectorError {
    #[error("Failed to read directory: {0}")]
    DirectoryRead(#[from] std::io::Error),

    #[error("Failed to get home directory")]
    HomeDirectoryNotFound,

    #[error("Failed to refresh process information")]
    ProcessRefreshError,
}

/// Information about a detected Claude Code session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedSession {
    /// Process ID of the running claude process
    pub pid: u32,

    /// Current working directory of the process
    pub cwd: PathBuf,

    /// Path to the session's project directory in ~/.claude/projects/
    pub project_path: PathBuf,

    /// Session ID (UUID from session file)
    pub session_id: Option<String>,

    /// Project name (derived from cwd)
    pub project_name: String,
}

/// Session detector that finds running Claude processes and matches them to session files
pub struct SessionDetector {
    system: System,
    claude_projects_dir: PathBuf,
}

impl SessionDetector {
    /// Creates a new SessionDetector
    pub fn new() -> Result<Self, SessionDetectorError> {
        let home_dir = dirs::home_dir()
            .ok_or(SessionDetectorError::HomeDirectoryNotFound)?;

        let claude_projects_dir = home_dir.join(".claude").join("projects");

        Ok(Self {
            system: System::new_all(),
            claude_projects_dir,
        })
    }

    /// Detects all active Claude Code sessions
    pub fn detect_sessions(&mut self) -> Result<Vec<DetectedSession>, SessionDetectorError> {
        // Refresh process information
        self.system.refresh_processes(ProcessesToUpdate::All, true);

        // Find all running Claude processes
        let claude_processes = self.find_claude_processes();

        // If no Claude processes are running, return empty
        if claude_processes.is_empty() {
            return Ok(Vec::new());
        }

        // Get all session project directories
        let project_dirs = self.enumerate_project_directories()?;

        // Find recently active sessions (modified in last 30 minutes)
        // and associate them with running processes
        let sessions = self.find_active_sessions(&claude_processes, &project_dirs);

        Ok(sessions)
    }

    /// Find sessions that are likely active based on recent modification
    fn find_active_sessions(
        &self,
        processes: &[ClaudeProcess],
        project_dirs: &[PathBuf],
    ) -> Vec<DetectedSession> {
        let mut sessions = Vec::new();
        let now = std::time::SystemTime::now();
        let thirty_mins_ago = now - std::time::Duration::from_secs(30 * 60);

        for project_dir in project_dirs {
            // First, find all JSONL files in the project directory that were recently modified
            // This catches sessions that aren't in the index yet (currently running)
            if let Ok(entries) = fs::read_dir(project_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    // Check if it's a JSONL file (not in subagents directory)
                    if path.is_file()
                        && path.extension().map_or(false, |ext| ext == "jsonl")
                    {
                        if let Ok(metadata) = fs::metadata(&path) {
                            if let Ok(modified) = metadata.modified() {
                                if modified > thirty_mins_ago {
                                    // Extract session ID from filename
                                    if let Some(session_id) = path.file_stem()
                                        .and_then(|s| s.to_str())
                                        .map(|s| s.to_string())
                                    {
                                        // Try to get project info from sessions-index.json
                                        let (project_path, project_name) = self
                                            .get_project_info_from_index(project_dir, &session_id)
                                            .unwrap_or_else(|| {
                                                // Fallback: derive from project directory name
                                                let name = project_dir
                                                    .file_name()
                                                    .and_then(|n| n.to_str())
                                                    .unwrap_or("unknown")
                                                    .to_string();
                                                (project_dir.clone(), name)
                                            });

                                        // Assign a process (round-robin if multiple)
                                        let pid = if !processes.is_empty() {
                                            processes[sessions.len() % processes.len()].pid
                                        } else {
                                            0
                                        };

                                        sessions.push(DetectedSession {
                                            pid,
                                            cwd: project_path.clone(),
                                            project_path: project_dir.clone(),
                                            session_id: Some(session_id),
                                            project_name,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        sessions
    }

    /// Get project info from sessions-index.json for a given session ID
    fn get_project_info_from_index(
        &self,
        project_dir: &Path,
        session_id: &str,
    ) -> Option<(PathBuf, String)> {
        let index_path = project_dir.join("sessions-index.json");

        if let Ok(content) = fs::read_to_string(&index_path) {
            if let Ok(index) = serde_json::from_str::<SessionsIndex>(&content) {
                if let Some(entries) = &index.entries {
                    for entry in entries {
                        if entry.session_id == session_id {
                            if let Some(proj_path) = &entry.project_path {
                                let path = PathBuf::from(proj_path);
                                let name = path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("unknown")
                                    .to_string();
                                return Some((path, name));
                            }
                        }
                    }

                    // If session not found in index, use first entry's project path as fallback
                    if let Some(first) = entries.first() {
                        if let Some(proj_path) = &first.project_path {
                            let path = PathBuf::from(proj_path);
                            let name = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                                .to_string();
                            return Some((path, name));
                        }
                    }
                }
            }
        }

        None
    }

    /// Finds all processes with name "claude"
    fn find_claude_processes(&self) -> Vec<ClaudeProcess> {
        let mut processes = Vec::new();

        for (pid, process) in self.system.processes() {
            // Check if the process name is "claude"
            let name = process.name().to_string_lossy();

            if name.contains("claude") && !name.contains("claude-session-monitor") {
                processes.push(ClaudeProcess {
                    pid: pid.as_u32(),
                });
            }
        }

        processes
    }

    /// Enumerates all project directories in ~/.claude/projects/
    fn enumerate_project_directories(&self) -> Result<Vec<PathBuf>, SessionDetectorError> {
        let mut project_dirs = Vec::new();

        // Check if the claude projects directory exists
        if !self.claude_projects_dir.exists() {
            return Ok(project_dirs);
        }

        // Read all entries in the projects directory
        let entries = fs::read_dir(&self.claude_projects_dir)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Only include directories
            if path.is_dir() {
                project_dirs.push(path);
            }
        }

        Ok(project_dirs)
    }

}

impl Default for SessionDetector {
    fn default() -> Self {
        Self::new().expect("Failed to create SessionDetector")
    }
}

/// Internal representation of a Claude process
#[derive(Debug)]
struct ClaudeProcess {
    pid: u32,
}

/// Structure of sessions-index.json
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionsIndex {
    #[allow(dead_code)]
    version: Option<u32>,
    entries: Option<Vec<SessionEntry>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionEntry {
    session_id: String,
    project_path: Option<String>,
    #[allow(dead_code)]
    full_path: Option<String>,
    #[allow(dead_code)]
    first_prompt: Option<String>,
    #[allow(dead_code)]
    summary: Option<String>,
    #[allow(dead_code)]
    message_count: Option<u32>,
    #[allow(dead_code)]
    git_branch: Option<String>,
    #[allow(dead_code)]
    modified: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let result = SessionDetector::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_claude_processes() {
        let detector = SessionDetector::new().unwrap();
        let processes = detector.find_claude_processes();
        // This test will vary based on whether claude is running
        println!("Found {} claude processes", processes.len());
    }

    #[test]
    fn test_enumerate_project_directories() {
        let detector = SessionDetector::new().unwrap();
        let result = detector.enumerate_project_directories();
        assert!(result.is_ok());

        if let Ok(dirs) = result {
            println!("Found {} project directories", dirs.len());
        }
    }
}
