## 2024-03-24 - Path Traversal in File Operations
**Vulnerability:** Session IDs were used directly in file paths without validation, allowing traversal (e.g., `../`).
**Learning:** External inputs used in file operations must always be validated against a strict allowlist.
**Prevention:** Use `validate_session_id` (alphanumeric, -, _) before any file operation involving session IDs.
