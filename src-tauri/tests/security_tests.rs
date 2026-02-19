use std::env;
use std::fs;

#[test]
fn test_path_traversal() {
    // Create a temporary directory for HOME
    let temp_dir = std::env::temp_dir().join(format!("c9watch_test_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();

    // Save original HOME
    let original_home = env::var("HOME").ok();

    // Set HOME to temp_dir
    // unsafe { env::set_var("HOME", &temp_dir); } // set_var is not unsafe, but has race conditions
    env::set_var("HOME", &temp_dir);

    // Create .claude/projects/project1
    let projects_dir = temp_dir.join(".claude").join("projects");
    let project_dir = projects_dir.join("project1");
    fs::create_dir_all(&project_dir).unwrap();

    // Create a secret file at HOME/secret.jsonl
    // Path from project1 to HOME/secret.jsonl: ../../../secret.jsonl
    let secret_file = temp_dir.join("secret.jsonl");

    // Valid JSONL content
    let secret_content = r#"{"type":"user","uuid":"1","timestamp":"2023-01-01","message":{"role":"user","content":"SECRET_DATA"}}"#;
    fs::write(&secret_file, secret_content).unwrap();

    // Attempt traversal
    let session_id = "../../../secret";

    let result = c9watch_lib::get_conversation_data(session_id);

    // Cleanup
    if let Some(h) = original_home {
        env::set_var("HOME", h);
    } else {
        env::remove_var("HOME"); // This might be wrong if HOME wasn't set, but unlikely
    }
    fs::remove_dir_all(&temp_dir).unwrap_or(());

    // Validate that the request failed with the expected error
    match result {
        Ok(_) => panic!("Vulnerability confirmed: Read file outside project directory!"),
        Err(e) => {
            assert_eq!(e, "Invalid session ID format", "Should reject invalid session ID");
        }
    }
}
