fn main() {
    // Ensure build/ exists so rust-embed's derive macro doesn't fail
    // during `cargo check` when the frontend hasn't been built yet.
    let build_dir = std::path::Path::new("../build");
    if !build_dir.exists() {
        std::fs::create_dir_all(build_dir).ok();
    }
    tauri_build::build()
}
