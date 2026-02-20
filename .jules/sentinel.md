## 2026-01-08 - Path Traversal via Unvalidated Session IDs
**Vulnerability:** The application used user-provided `session_id` directly in file path construction (`format!("{}.jsonl", session_id)`), allowing path traversal (e.g., `../secret`) to access arbitrary files ending in `.jsonl`.
**Learning:** Even when appending a fixed extension, path traversal is possible if the base identifier is not validated. `Path::join` resolves `..` components.
**Prevention:** Validate all user-provided identifiers against a strict allowlist (e.g., alphanumeric only) before using them in file system operations.
