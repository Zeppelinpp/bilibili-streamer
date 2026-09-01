use crate::models::live::StartLiveResponse;
use crate::services::live_session::LiveSession;
use crate::state::AppState;
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub async fn get_partitions(
    state: State<'_, AppState>,
) -> Result<HashMap<String, Vec<String>>, String> {
    let mut live = state.live.lock().await;
    if live.get_partitions().is_empty() {
        live.refresh_partitions(&state.api)
            .await
            .map_err(|e| e.to_string())?;
    }
    let raw = live.get_partitions();
    let mut result = HashMap::with_capacity(raw.len());
    for (parent, subs) in raw {
        result.insert(parent, subs.into_keys().collect());
    }
    Ok(result)
}

#[tauri::command]
pub async fn update_title(title: String, state: State<'_, AppState>) -> Result<(), String> {
    LiveSession::new(&state)
        .update_title(&title)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_area(
    p_name: String,
    s_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    LiveSession::new(&state)
        .update_area(&p_name, &s_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_live(
    p_name: Option<String>,
    s_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<StartLiveResponse, String> {
    LiveSession::new(&state)
        .start_live(p_name, s_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_live(state: State<'_, AppState>) -> Result<(), String> {
    LiveSession::new(&state)
        .stop_live()
        .await
        .map_err(|e| e.to_string())
}
