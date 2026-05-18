use crate::models::danmaku::SendDanmakuResult;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn start_danmaku_monitor(state: State<'_, AppState>) -> Result<(), String> {
    let session = state.session.lock().await;
    let room_id = session.room_id.clone().ok_or("未登录")?;
    let room_id_num = room_id.parse::<u64>().map_err(|_| "房间号无效")?;
    let uid = session.uid;
    drop(session);

    let danmaku_opt = state.danmaku.lock().await;
    if let Some(danmaku) = danmaku_opt.as_ref() {
        if danmaku.is_running().await {
            return Ok(());
        }
        danmaku.connect(room_id_num, uid).await;
    }
    Ok(())
}

#[tauri::command]
pub async fn stop_danmaku_monitor(state: State<'_, AppState>) -> Result<(), String> {
    let danmaku_opt = state.danmaku.lock().await;
    if let Some(danmaku) = danmaku_opt.as_ref() {
        danmaku.disconnect().await;
    }
    Ok(())
}

#[tauri::command]
pub async fn send_danmaku(
    msg: String,
    state: State<'_, AppState>,
) -> Result<SendDanmakuResult, String> {
    let api = state.api.lock().await;
    let session = state.session.lock().await;
    let room_id = session.room_id.clone().ok_or("未登录")?;
    let room_id_num = room_id.parse::<u64>().map_err(|_| "房间号无效")?;
    let csrf = session.csrf.clone().ok_or("未获取CSRF")?;
    drop(session);
    let res = api
        .send_danmaku(room_id_num, &msg, &csrf)
        .await
        .map_err(|e| e.to_string())?;
    let code = res["code"].as_i64().unwrap_or(-1) as i32;
    let msg_text = match code {
        0 => "发送成功",
        1003212 => "超出限制长度",
        -101 => "未登录",
        -400 => "参数错误",
        10031 => "发送频率过高",
        _ => res["msg"].as_str().unwrap_or("未知错误"),
    };
    Ok(SendDanmakuResult {
        code,
        msg: msg_text.to_string(),
    })
}

#[tauri::command]
pub async fn get_emote_list(
    state: State<'_, AppState>,
) -> Result<std::collections::HashMap<String, String>, String> {
    let api = state.api.lock().await;
    let session = state.session.lock().await;
    let room_id = session.room_id.clone().unwrap_or_default();
    drop(session);
    let room_id_num = if room_id.is_empty() {
        None
    } else {
        room_id.parse::<u64>().ok()
    };
    api.get_emote_list(room_id_num)
        .await
        .map_err(|e| e.to_string())
}
