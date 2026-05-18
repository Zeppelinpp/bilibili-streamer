#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bili_live_tool_lib::services::bili_api::BiliApi;
use bili_live_tool_lib::services::config_store::ConfigStore;
use bili_live_tool_lib::services::danmaku_ws::DanmakuService;
use bili_live_tool_lib::services::user_service::UserService;
use bili_live_tool_lib::state::{AppState, SessionState};
use std::sync::Arc;
use tauri::{Emitter, Manager};

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

            // System tray
            let show_i = tauri::menu::MenuItem::with_id(app, "show", "显示主界面", true, None::<&str>)?;
            let start_i = tauri::menu::MenuItem::with_id(app, "start", "开始直播", true, None::<&str>)?;
            let stop_i = tauri::menu::MenuItem::with_id(app, "stop", "停止直播", true, None::<&str>)?;
            let sep = tauri::menu::PredefinedMenuItem::separator(app)?;
            let quit_i = tauri::menu::PredefinedMenuItem::quit(app, Some("退出"))?;
            let menu = tauri::menu::Menu::with_items(app, &[&show_i, &start_i, &stop_i, &sep, &quit_i])?;

            tauri::tray::TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().ok_or("No default window icon set")?.clone())
                .tooltip("BiliLiveTool")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(win) = app.get_webview_window("main") {
                            if let Err(e) = win.show() {
                                tracing::warn!("Failed to show window: {}", e);
                            }
                            if let Err(e) = win.set_focus() {
                                tracing::warn!("Failed to focus window: {}", e);
                            }
                        }
                    }
                    "start" => {
                        let _ = app.emit("tray-start-live", ());
                    }
                    "stop" => {
                        let _ = app.emit("tray-stop-live", ());
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(win) = app.get_webview_window("main") {
                            if let Err(e) = win.unminimize() {
                                tracing::warn!("Failed to unminimize window: {}", e);
                            }
                            if let Err(e) = win.show() {
                                tracing::warn!("Failed to show window: {}", e);
                            }
                            if let Err(e) = win.set_focus() {
                                tracing::warn!("Failed to focus window: {}", e);
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let handle = window.app_handle().clone();
                let state = handle.state::<AppState>();
                // SAFETY: block_in_place + block_on is safe here because the config lock
                // is held only for a brief synchronous read (min_to_tray boolean), with no
                // nested async calls or other locks acquired.
                let config = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(state.config.lock())
                });
                if config.data().min_to_tray {
                    api.prevent_close();
                    if let Err(e) = window.hide() {
                        tracing::warn!("Failed to hide window: {}", e);
                    }
                }
            }
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
