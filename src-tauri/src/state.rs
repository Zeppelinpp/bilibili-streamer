use std::sync::atomic::AtomicBool;
use std::sync::Arc;

// Lock ordering rule (config -> session -> live -> danmaku) is owned and
// documented by services::live_session::LiveSession, which orchestrates all
// multi-lock live operations. `api` is not part of the ordering: BiliApi
// guards its own mutable state internally and is shared lock-free.

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
    pub api: Arc<crate::services::bili_api::BiliApi>,
    pub danmaku: tokio::sync::Mutex<Option<crate::services::danmaku_ws::DanmakuService>>,
    pub live: tokio::sync::Mutex<crate::services::live_service::LiveService>,
    pub exiting: AtomicBool,
    pub cleanup_complete: tokio::sync::Notify,
}
