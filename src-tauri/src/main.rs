#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bili_live_tool_lib::services::bili_api::BiliApi;
use bili_live_tool_lib::services::config_store::ConfigStore;
use bili_live_tool_lib::services::danmaku_ws::DanmakuService;
use bili_live_tool_lib::services::user_service::UserService;
use bili_live_tool_lib::state::{AppState, SessionState};
use std::sync::Arc;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let config = ConfigStore::new().expect("Failed to load config");
            let mut api = BiliApi::new().expect("Failed to create API client");
            let mut session = SessionState::default();
            UserService::init_current_user(&config, &mut session, &mut api);
            let api = Arc::new(tokio::sync::Mutex::new(api));
            let danmaku = DanmakuService::new(api.clone(), app.handle().clone());

            app.manage(AppState {
                config: tokio::sync::Mutex::new(config),
                session: tokio::sync::Mutex::new(session),
                api,
                danmaku: tokio::sync::Mutex::new(Some(danmaku)),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bili_live_tool_lib::commands::auth::get_login_qrcode,
            bili_live_tool_lib::commands::auth::poll_login_status,
            bili_live_tool_lib::commands::user::load_saved_config,
            bili_live_tool_lib::commands::user::refresh_current_user,
            bili_live_tool_lib::commands::user::get_account_list,
            bili_live_tool_lib::commands::user::switch_account,
            bili_live_tool_lib::commands::user::logout,
            bili_live_tool_lib::commands::live::get_partitions,
            bili_live_tool_lib::commands::live::update_title,
            bili_live_tool_lib::commands::live::update_area,
            bili_live_tool_lib::commands::live::start_live,
            bili_live_tool_lib::commands::live::stop_live,
            bili_live_tool_lib::commands::danmaku::start_danmaku_monitor,
            bili_live_tool_lib::commands::danmaku::stop_danmaku_monitor,
            bili_live_tool_lib::commands::danmaku::send_danmaku,
            bili_live_tool_lib::commands::window::window_min,
            bili_live_tool_lib::commands::window::window_max,
            bili_live_tool_lib::commands::window::window_close,
            bili_live_tool_lib::commands::window::window_drag,
            bili_live_tool_lib::commands::config::get_app_config,
            bili_live_tool_lib::commands::config::set_app_config,
            bili_live_tool_lib::commands::config::get_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
