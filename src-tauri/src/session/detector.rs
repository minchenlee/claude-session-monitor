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

        // Get all session project directories
        let project_dirs = self.enumerate_project_directories()?;

        // Match processes to sessions
        let sessions = self.match_processes_to_sessions(claude_processes, project_dirs);

        Ok(sessions)
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
                    if let Some(cwd) = index.cwd {
                        map.insert(PathBuf::from(cwd), project_path.clone());
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
                if let Some(sessions) = &index.sessions {
                    // Find sessions that match this cwd
                    for session in sessions {
                        if let Some(session_cwd) = &session.cwd {
                            if PathBuf::from(session_cwd) == cwd {
                                return Some(session.id.clone());
                            }
                        }
                    }

                    // If no exact match, return the most recent session
                    if let Some(latest) = sessions.first() {
                        return Some(latest.id.clone());
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
struct SessionsIndex {
    cwd: Option<String>,
    sessions: Option<Vec<SessionInfo>>,
}

#[derive(Debug, Deserialize)]
struct SessionInfo {
    id: String,
    cwd: Option<String>,
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
