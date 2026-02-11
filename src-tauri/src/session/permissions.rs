use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Claude Code settings structure (partial - only what we need)
#[derive(Debug, Deserialize)]
pub struct ClaudeSettings {
    pub permissions: Option<Permissions>,
}

#[derive(Debug, Deserialize)]
pub struct Permissions {
    pub allow: Option<Vec<String>>,
}

/// Cached permissions for quick lookup
#[derive(Debug, Clone, Default)]
pub struct PermissionChecker {
    allowed_patterns: Vec<AllowPattern>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum AllowPattern {
    /// Bash command pattern, e.g., "git add" from "Bash(git add:*)"
    Bash { prefix: String, wildcard: bool },
    /// Full tool allow, e.g., "Read" means all Read operations are allowed
    Tool { name: String },
    /// MCP tool pattern
    Mcp { name: String },
    /// Skill pattern
    Skill { name: String },
}

impl PermissionChecker {
    /// Load permissions from settings file
    pub fn from_settings_file() -> Self {
        let home_dir = match dirs::home_dir() {
            Some(dir) => dir,
            None => return Self::default(),
        };

        let settings_path = home_dir.join(".claude").join("settings.json");
        Self::from_file(&settings_path)
    }

    /// Load permissions from a specific file
    pub fn from_file(path: &Path) -> Self {
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };

        let settings: ClaudeSettings = match serde_json::from_str(&content) {
            Ok(s) => s,
            Err(_) => return Self::default(),
        };

        let allowed = settings
            .permissions
            .and_then(|p| p.allow)
            .unwrap_or_default();

        let patterns = allowed
            .iter()
            .filter_map(|s| Self::parse_pattern(s))
            .collect();

        Self {
            allowed_patterns: patterns,
        }
    }

    /// Parse a permission pattern string into an AllowPattern
    fn parse_pattern(pattern: &str) -> Option<AllowPattern> {
        // Pattern formats:
        // - "Bash(command:*)" or "Bash(command)" - bash command
        // - "Read" - full tool access
        // - "mcp__server__tool" - MCP tool
        // - "Skill(name)" - skill

        if pattern.starts_with("Bash(") && pattern.ends_with(")") {
            // Extract the command pattern
            let inner = &pattern[5..pattern.len() - 1];

            // Check for wildcard
            if let Some(stripped) = inner.strip_suffix(":*") {
                Some(AllowPattern::Bash {
                    prefix: stripped.to_string(),
                    wildcard: true,
                })
            } else {
                Some(AllowPattern::Bash {
                    prefix: inner.to_string(),
                    wildcard: false,
                })
            }
        } else if pattern.starts_with("mcp__") {
            Some(AllowPattern::Mcp {
                name: pattern.to_string(),
            })
        } else if pattern.starts_with("Skill(") && pattern.ends_with(")") {
            let inner = &pattern[6..pattern.len() - 1];
            Some(AllowPattern::Skill {
                name: inner.to_string(),
            })
        } else if !pattern.contains('(') && !pattern.contains("__") {
            // Simple tool name like "Read", "Write", etc.
            Some(AllowPattern::Tool {
                name: pattern.to_string(),
            })
        } else {
            None
        }
    }

    /// Check if a tool use is auto-approved
    ///
    /// # Arguments
    /// * `tool_name` - The name of the tool (e.g., "Bash", "Read", "Glob")
    /// * `tool_input` - The tool input as a JSON value
    ///
    /// # Returns
    /// true if the tool is auto-approved, false if it needs user permission
    pub fn is_auto_approved(&self, tool_name: &str, tool_input: &serde_json::Value) -> bool {
        // These tools are always auto-approved (read-only operations)
        match tool_name {
            "Read" | "Glob" | "Grep" | "WebFetch" | "WebSearch" | "Task" | "TaskList"
            | "TaskGet" | "TaskCreate" | "TaskUpdate" | "AskUserQuestion" => {
                return true;
            }
            _ => {}
        }

        // For Bash, check against allowed patterns
        if tool_name == "Bash" {
            let command = tool_input
                .get("command")
                .and_then(|c| c.as_str())
                .unwrap_or("");

            return self.is_bash_allowed(command);
        }

        // For Write/Edit, check if explicitly allowed
        if tool_name == "Write" || tool_name == "Edit" || tool_name == "NotebookEdit" {
            // These typically need permission unless explicitly allowed
            return self.is_tool_allowed(tool_name);
        }

        // For MCP tools, check pattern
        if tool_name.starts_with("mcp__") {
            return self.is_mcp_allowed(tool_name);
        }

        // Default: assume needs permission
        false
    }

    /// Check if a bash command matches any allowed pattern
    fn is_bash_allowed(&self, command: &str) -> bool {
        let command_trimmed = command.trim();

        for pattern in &self.allowed_patterns {
            if let AllowPattern::Bash { prefix, wildcard } = pattern {
                if *wildcard {
                    // Prefix match with wildcard
                    if command_trimmed.starts_with(prefix) {
                        return true;
                    }
                } else {
                    // Exact match
                    if command_trimmed == prefix {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Check if a tool is explicitly allowed
    fn is_tool_allowed(&self, tool_name: &str) -> bool {
        for pattern in &self.allowed_patterns {
            if let AllowPattern::Tool { name } = pattern {
                if name == tool_name {
                    return true;
                }
            }
        }
        false
    }

    /// Check if an MCP tool is allowed
    fn is_mcp_allowed(&self, tool_name: &str) -> bool {
        for pattern in &self.allowed_patterns {
            if let AllowPattern::Mcp { name } = pattern {
                if name == tool_name {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bash_pattern_with_wildcard() {
        let pattern = PermissionChecker::parse_pattern("Bash(git add:*)");
        assert!(
            matches!(pattern, Some(AllowPattern::Bash { prefix, wildcard: true }) if prefix == "git add")
        );
    }

    #[test]
    fn test_parse_bash_pattern_exact() {
        let pattern = PermissionChecker::parse_pattern("Bash(npm ci)");
        assert!(
            matches!(pattern, Some(AllowPattern::Bash { prefix, wildcard: false }) if prefix == "npm ci")
        );
    }

    #[test]
    fn test_parse_mcp_pattern() {
        let pattern = PermissionChecker::parse_pattern("mcp__atlassian__getJiraIssue");
        assert!(
            matches!(pattern, Some(AllowPattern::Mcp { name }) if name == "mcp__atlassian__getJiraIssue")
        );
    }

    #[test]
    fn test_always_allowed_tools() {
        let checker = PermissionChecker::default();

        assert!(checker.is_auto_approved("Read", &serde_json::json!({})));
        assert!(checker.is_auto_approved("Glob", &serde_json::json!({})));
        assert!(checker.is_auto_approved("Grep", &serde_json::json!({})));
    }

    #[test]
    fn test_bash_command_matching() {
        let checker = PermissionChecker {
            allowed_patterns: vec![
                AllowPattern::Bash {
                    prefix: "git add".to_string(),
                    wildcard: true,
                },
                AllowPattern::Bash {
                    prefix: "npm ci".to_string(),
                    wildcard: false,
                },
            ],
        };

        // Should match git add with wildcard
        assert!(checker.is_auto_approved("Bash", &serde_json::json!({"command": "git add ."})));

        // Should match exact npm ci
        assert!(checker.is_auto_approved("Bash", &serde_json::json!({"command": "npm ci"})));

        // Should NOT match npm ci with arguments (exact match required)
        assert!(!checker.is_auto_approved(
            "Bash",
            &serde_json::json!({"command": "npm ci --legacy-peer-deps"})
        ));

        // Should NOT match random command
        assert!(!checker.is_auto_approved("Bash", &serde_json::json!({"command": "rm -rf /"})));
    }

    #[test]
    fn test_load_from_real_settings() {
        // This test uses the real settings file if available
        let checker = PermissionChecker::from_settings_file();

        // Just verify it loads without crashing
        println!("Loaded {} patterns", checker.allowed_patterns.len());
    }
}
