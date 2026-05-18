use crate::models::live::StartLiveResponse;
use crate::state::AppState;
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub async fn get_partitions(state: State<'_, AppState>) -> Result<HashMap<String, Vec<String>>, String> {
    let api = state.api.lock().await;
    let mut live = crate::services::live_service::LiveService::new();
    live.refresh_partitions(&api).await.map_err(|e| e.to_string())?;
    let raw = live.get_partitions();
    let mut result = HashMap::with_capacity(raw.len());
    for (parent, subs) in raw {
        result.insert(parent, subs.into_keys().collect());
    }
    Ok(result)
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
) -> Result<StartLiveResponse, String> {
    let api = state.api.lock().await;
    let mut config = state.config.lock().await;
    let mut session = state.session.lock().await;
    let mut live = crate::services::live_service::LiveService::new();
    let result = live.start_live(&api, &mut session, &mut config, p_name, s_name).await.map_err(|e| e.to_string())?;

    if result.code == 0 {
        let room_id = session.room_id.clone();
        drop(api);
        drop(config);
        drop(session);

        if let Some(room_id) = room_id {
            if let Ok(room_id_num) = room_id.parse::<u64>() {
                let danmaku_opt = state.danmaku.lock().await;
                if let Some(danmaku) = danmaku_opt.as_ref() {
                    if !danmaku.is_running().await {
                        danmaku.connect(room_id_num).await;
                    }
                }
            }
        }
    }

    Ok(result)
}

#[tauri::command]
pub async fn stop_live(state: State<'_, AppState>) -> Result<(), String> {
    let api = state.api.lock().await;
    let mut session = state.session.lock().await;
    let mut live = crate::services::live_service::LiveService::new();
    live.stop_live(&api, &mut session).await.map_err(|e| e.to_string())?;
    drop(api);
    drop(session);

    let danmaku_opt = state.danmaku.lock().await;
    if let Some(danmaku) = danmaku_opt.as_ref() {
        if danmaku.is_running().await {
            danmaku.disconnect().await;
        }
    }

    Ok(())
}
