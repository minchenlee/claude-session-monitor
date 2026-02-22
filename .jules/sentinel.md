## 2024-05-22 - Path Traversal in Session IDs
**Vulnerability:** `get_conversation_data` constructed file paths using unvalidated `session_id` inputs, allowing traversal out of the project directory.
**Learning:** The application assumes session IDs are safe because they are often generated internally, but they can be supplied by the user/frontend via WebSocket or Tauri commands.
**Prevention:** Always validate `session_id` using `validate_session_id` (alphanumeric whitelist) before using it in file system operations.
