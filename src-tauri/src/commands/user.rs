use crate::models::config::UserConfig;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn load_saved_config(state: State<'_, AppState>) -> Result<Option<UserConfig>, String> {
    let config = state.config.lock().await;
    let uid = config.data().current_uid;
    if let Some(uid) = uid {
        let uid_str = uid.to_string();
        Ok(config.data().users.get(&uid_str).cloned())
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn refresh_current_user(state: State<'_, AppState>) -> Result<UserConfig, String> {
    let mut api = state.api.lock().await;
    let mut config = state.config.lock().await;
    let mut session = state.session.lock().await;
    crate::services::user_service::UserService::refresh_current_user(
        &mut api,
        &mut config,
        &mut session,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_account_list(state: State<'_, AppState>) -> Result<Vec<UserConfig>, String> {
    let config = state.config.lock().await;
    Ok(crate::services::user_service::UserService::get_account_list(&config))
}

#[tauri::command]
pub async fn switch_account(uid: u64, state: State<'_, AppState>) -> Result<UserConfig, String> {
    let mut api = state.api.lock().await;
    let mut config = state.config.lock().await;
    let mut session = state.session.lock().await;
    crate::services::user_service::UserService::switch_account(
        &mut config,
        &mut session,
        &mut api,
        uid,
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn logout(uid: u64, state: State<'_, AppState>) -> Result<(), String> {
    let mut api = state.api.lock().await;
    let mut config = state.config.lock().await;
    let mut session = state.session.lock().await;
    crate::services::user_service::UserService::logout(&mut config, &mut session, &mut api, uid)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_session(state: State<'_, AppState>) -> Result<(), String> {
    let mut api = state.api.lock().await;
    let mut config = state.config.lock().await;
    let mut session = state.session.lock().await;
    session.uid = None;
    session.room_id = None;
    session.csrf = None;
    session.is_live = false;
    api.update_cookies(std::collections::HashMap::new());
    config.data_mut().current_uid = None;
    config.save().map_err(|e| e.to_string())
}
