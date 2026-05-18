use crate::models::live::{PartitionMap, StreamCodeData};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_partitions(state: State<'_, AppState>) -> Result<PartitionMap, String> {
    let api = state.api.lock().await;
    let mut live = crate::services::live_service::LiveService::new();
    live.refresh_partitions(&api).await.map_err(|e| e.to_string())?;
    Ok(live.get_partitions())
}

#[tauri::command]
pub async fn update_title(title: String, state: State<'_, AppState>) -> Result<(), String> {
    let api = state.api.lock().await;
    let mut config = state.config.lock().await;
    let session = state.session.lock().await;
    crate::services::live_service::LiveService::update_title(&api, &session, &mut config, &title).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_area(p_name: String, s_name: String, state: State<'_, AppState>) -> Result<(), String> {
    let api = state.api.lock().await;
    let mut config = state.config.lock().await;
    let mut session = state.session.lock().await;
    let mut live = crate::services::live_service::LiveService::new();
    live.update_area(&api, &mut session, &mut config, &p_name, &s_name).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_live(
    p_name: Option<String>,
    s_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<StreamCodeData, String> {
    let api = state.api.lock().await;
    let mut config = state.config.lock().await;
    let mut session = state.session.lock().await;
    let mut live = crate::services::live_service::LiveService::new();
    let result = live.start_live(&api, &mut session, &mut config, p_name, s_name).await.map_err(|e| e.to_string())?;
    Ok(result)
}

#[tauri::command]
pub async fn stop_live(state: State<'_, AppState>) -> Result<(), String> {
    let api = state.api.lock().await;
    let mut session = state.session.lock().await;
    let mut live = crate::services::live_service::LiveService::new();
    live.stop_live(&api, &mut session).await.map_err(|e| e.to_string())
}
