use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

const STATE_DIR: &str = "state";
const STATE_FILE: &str = "app-state.json";
const WEBVIEW_DATA_DIR: &str = "webview-data";

#[tauri::command]
pub fn load_app_state() -> Result<Option<Value>, String> {
    let path = app_state_path();
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let state = serde_json::from_str(&text).map_err(|error| error.to_string())?;
    Ok(Some(state))
}

#[tauri::command]
pub fn save_app_state(state: Value) -> Result<(), String> {
    let path = app_state_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let text = serde_json::to_string_pretty(&state).map_err(|error| error.to_string())?;
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, text).map_err(|error| error.to_string())?;
    if path.exists() {
        fs::remove_file(&path).map_err(|error| error.to_string())?;
    }
    fs::rename(tmp_path, path).map_err(|error| error.to_string())
}

pub fn configure_portable_webview_data() -> Result<(), String> {
    let dir = portable_root().join(WEBVIEW_DATA_DIR);
    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", dir);
    Ok(())
}

fn app_state_path() -> PathBuf {
    portable_root().join(STATE_DIR).join(STATE_FILE)
}

fn portable_root() -> PathBuf {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let exe_path = std::env::current_exe().ok();
    portable_root_from(&current_dir, exe_path.as_deref())
}

fn portable_root_from(current_dir: &Path, exe_path: Option<&Path>) -> PathBuf {
    let mut candidates = vec![current_dir.to_path_buf()];
    if let Some(parent) = current_dir.parent() {
        candidates.push(parent.to_path_buf());
    }
    if let Some(exe_dir) = exe_path.and_then(Path::parent) {
        candidates.push(exe_dir.to_path_buf());
        if let Some(parent) = exe_dir.parent() {
            candidates.push(parent.to_path_buf());
        }
    }
    for candidate in &candidates {
        if candidate.join("presets").exists() || candidate.join("exports").exists() {
            return candidate.clone();
        }
    }
    exe_path
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| current_dir.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn portable_root_prefers_current_parent_with_project_data() {
        let root = test_dir("state-parent-root");
        let child = root.join("src-tauri");
        fs::create_dir_all(root.join("presets")).unwrap();
        fs::create_dir_all(&child).unwrap();

        let selected = portable_root_from(&child, None);

        assert_eq!(selected, root);
    }

    #[test]
    fn portable_root_prefers_exe_dir_with_portable_data() {
        let root = test_dir("state-exe-root");
        let bin = root.join("retro-sound-studio.exe");
        fs::create_dir_all(root.join("exports")).unwrap();

        let selected = portable_root_from(Path::new("C:/unrelated"), Some(&bin));

        assert_eq!(selected, root);
    }

    #[test]
    fn portable_root_falls_back_to_exe_dir() {
        let root = test_dir("state-exe-fallback");
        let bin = root.join("retro-sound-studio.exe");

        let selected = portable_root_from(Path::new("C:/unrelated"), Some(&bin));

        assert_eq!(selected, root);
    }

    fn test_dir(name: &str) -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("retro-sound-studio-{name}-{id}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}
