use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
            let index_path = project_dir.join("sessions-index.json");

            if let Ok(content) = fs::read_to_string(&index_path) {
                if let Ok(index) = serde_json::from_str::<SessionsIndex>(&content) {
                    if let Some(entries) = &index.entries {
                        for entry in entries {
                            // Check if session file was recently modified
                            if let Some(full_path) = &entry.full_path {
                                let session_path = PathBuf::from(full_path);
                                if let Ok(metadata) = fs::metadata(&session_path) {
                                    if let Ok(modified) = metadata.modified() {
                                        if modified > thirty_mins_ago {
                                            // This is a recently active session
                                            let project_path = entry.project_path
                                                .as_ref()
                                                .map(PathBuf::from)
                                                .unwrap_or_else(|| project_dir.clone());

                                            let project_name = project_path
                                                .file_name()
                                                .and_then(|n| n.to_str())
                                                .unwrap_or("unknown")
                                                .to_string();

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
                                                session_id: Some(entry.session_id.clone()),
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
        }

        sessions
    }

    /// Finds all processes with name "claude"
    fn find_claude_processes(&self) -> Vec<ClaudeProcess> {
        let mut processes = Vec::new();

        for (pid, process) in self.system.processes() {
            // Check if the process name is "claude"
            let name = process.name().to_string_lossy();

            if name.contains("claude") && !name.contains("claude-session-monitor") {
                if let Some(cwd) = process.cwd() {
                    processes.push(ClaudeProcess {
                        pid: pid.as_u32(),
                        cwd: cwd.to_path_buf(),
                    });
                }
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

    /// Matches Claude processes to their corresponding session directories
    fn match_processes_to_sessions(
        &self,
        processes: Vec<ClaudeProcess>,
        project_dirs: Vec<PathBuf>,
    ) -> Vec<DetectedSession> {
        let mut sessions = Vec::new();

        // Create a map of project directories for faster lookup
        let project_map = self.build_project_map(&project_dirs);

        for process in processes {
            // Try to match this process to a project directory
            if let Some(project_path) = self.find_matching_project(&process.cwd, &project_map) {
                // Extract project name from cwd
                let project_name = process.cwd
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Try to find session ID from the project path
                let session_id = self.find_session_id_for_cwd(&project_path, &process.cwd);

                sessions.push(DetectedSession {
                    pid: process.pid,
                    cwd: process.cwd.clone(),
                    project_path: project_path.clone(),
                    session_id,
                    project_name,
                });
            }
        }

        sessions
    }

    /// Builds a map of working directories to project paths
    fn build_project_map(&self, project_dirs: &[PathBuf]) -> HashMap<PathBuf, PathBuf> {
        let mut map = HashMap::new();

        for project_path in project_dirs {
            // Read the sessions-index.json to find the working directory
            let index_path = project_path.join("sessions-index.json");

            if let Ok(content) = fs::read_to_string(&index_path) {
                if let Ok(index) = serde_json::from_str::<SessionsIndex>(&content) {
                    // Get project path from the first entry
                    if let Some(entries) = &index.entries {
                        if let Some(first_entry) = entries.first() {
                            if let Some(proj_path) = &first_entry.project_path {
                                map.insert(PathBuf::from(proj_path), project_path.clone());
                            }
                        }
                    }
                }
            }
        }

        map
    }

    /// Finds the project directory that matches the given working directory
    fn find_matching_project(
        &self,
        cwd: &Path,
        project_map: &HashMap<PathBuf, PathBuf>,
    ) -> Option<PathBuf> {
        // Direct match
        if let Some(project_path) = project_map.get(cwd) {
            return Some(project_path.clone());
        }

        // Check if cwd is a subdirectory of any project's cwd
        for (project_cwd, project_path) in project_map {
            if cwd.starts_with(project_cwd) {
                return Some(project_path.clone());
            }
        }

        None
    }

    /// Finds the session ID for a given working directory
    fn find_session_id_for_cwd(&self, project_path: &Path, cwd: &Path) -> Option<String> {
        let index_path = project_path.join("sessions-index.json");

        if let Ok(content) = fs::read_to_string(&index_path) {
            if let Ok(index) = serde_json::from_str::<SessionsIndex>(&content) {
                if let Some(entries) = &index.entries {
                    // Find sessions that match this cwd
                    for entry in entries {
                        if let Some(project_path_str) = &entry.project_path {
                            if PathBuf::from(project_path_str) == cwd {
                                return Some(entry.session_id.clone());
                            }
                        }
                    }

                    // If no exact match, return the most recent session
                    if let Some(latest) = entries.first() {
                        return Some(latest.session_id.clone());
                    }
                }
            }
        }

        None
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
    cwd: PathBuf,
}

/// Structure of sessions-index.json
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionsIndex {
    version: Option<u32>,
    entries: Option<Vec<SessionEntry>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionEntry {
    session_id: String,
    project_path: Option<String>,
    full_path: Option<String>,
    first_prompt: Option<String>,
    summary: Option<String>,
    message_count: Option<u32>,
    git_branch: Option<String>,
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
