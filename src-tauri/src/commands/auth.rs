use crate::models::user::{LoginResult, QrCodeData};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_login_qrcode(state: State<'_, AppState>) -> Result<QrCodeData, String> {
    let api = state.api.lock().await;
    crate::services::auth_service::AuthService::get_login_qrcode(&api).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn poll_login_status(key: String, state: State<'_, AppState>) -> Result<LoginResult, String> {
    let api = state.api.lock().await;
    crate::services::auth_service::AuthService::poll_login_status(&api, &key).await.map_err(|e| e.to_string())
}
