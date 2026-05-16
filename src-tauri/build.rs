fn main() {
    if std::path::Path::new("icons/icon.ico").exists() {
        tauri_build::build()
    }
}
