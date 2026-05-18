use crate::state::AppState;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub async fn get_app_config(state: State<'_, AppState>) -> Result<Value, String> {
    let config = state.config.lock().await;
    Ok(serde_json::json!({
        "min_to_tray": config.data().min_to_tray,
    }))
}

#[tauri::command]
pub async fn set_app_config(
    key: String,
    value: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.config.lock().await;
    if key == "min_to_tray" {
        config.data_mut().min_to_tray = value;
    }
    config.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
