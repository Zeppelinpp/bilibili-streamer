use std::sync::Arc;

#[derive(Default)]
pub struct SessionState {
    pub uid: Option<u64>,
    pub room_id: Option<String>,
    pub csrf: Option<String>,
    pub is_live: bool,
    pub current_area_id: Option<u64>,
    pub current_area_names: Vec<String>,
}

pub struct AppState {
    pub config: tokio::sync::Mutex<crate::services::config_store::ConfigStore>,
    pub session: tokio::sync::Mutex<SessionState>,
    pub api: Arc<tokio::sync::Mutex<crate::services::bili_api::BiliApi>>,
    pub danmaku: tokio::sync::Mutex<Option<crate::services::danmaku_ws::DanmakuService>>,
}
