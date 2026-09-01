use crate::models::user::{LoginResult, QrCodeData};
use crate::state::AppState;
use std::collections::HashMap;
use tauri::State;

#[tauri::command]
pub async fn get_login_qrcode(state: State<'_, AppState>) -> Result<QrCodeData, String> {
    crate::services::auth_service::AuthService::get_login_qrcode(&state.api)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn poll_login_status(
    key: String,
    state: State<'_, AppState>,
) -> Result<LoginResult, String> {
    let result = crate::services::auth_service::AuthService::poll_login_status(&state.api, &key)
        .await
        .map_err(|e| e.to_string())?;

    if result.code == 0 {
        if let Some(ref user) = result.user {
            let cookies = parse_cookie_str(&user.cookie);
            state.api.update_cookies(cookies);
        }
    }

    if result.code == 0 {
        if let Some(ref user) = result.user {
            let mut session = state.session.lock().await;
            session.uid = Some(user.uid);
            session.csrf = Some(user.csrf.clone());
        }
    }

    Ok(result)
}

fn parse_cookie_str(s: &str) -> HashMap<String, String> {
    s.split(';')
        .filter_map(|part| {
            let mut kv = part.trim().splitn(2, '=');
            let k = kv.next()?;
            let v = kv.next()?;
            Some((k.to_string(), v.to_string()))
        })
        .collect()
}
