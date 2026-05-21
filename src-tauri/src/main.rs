#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bilibili_streamer_lib::services::bili_api::BiliApi;
use bilibili_streamer_lib::services::config_store::ConfigStore;
use bilibili_streamer_lib::services::danmaku_ws::DanmakuService;
use bilibili_streamer_lib::services::live_service::LiveService;
use bilibili_streamer_lib::services::user_service::UserService;
use bilibili_streamer_lib::state::{AppState, SessionState};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{Emitter, Manager};

async fn cleanup_and_exit(app_handle: tauri::AppHandle) {
    let state = app_handle.state::<AppState>();
    if state.exiting.swap(true, Ordering::SeqCst) {
        // Another task is already cleaning up; wait for it and exit
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        std::process::exit(0);
    }
    let api = state.api.lock().await;
    let mut session = state.session.lock().await;
    if session.is_live {
        let mut live = LiveService::new();
        if let Err(e) = live.stop_live(&api, &mut session).await {
            tracing::error!("Failed to stop live on exit: {}", e);
        }
    }
    std::process::exit(0);
}

fn main() {
    tracing_subscriber::fmt()
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let app = tauri::Builder::default()
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
                live: tokio::sync::Mutex::new(LiveService::new()),
                exiting: AtomicBool::new(false),
            });

            // System tray
            let show_i =
                tauri::menu::MenuItem::with_id(app, "show", "显示主界面", true, None::<&str>)?;
            let start_i =
                tauri::menu::MenuItem::with_id(app, "start", "开始直播", true, None::<&str>)?;
            let stop_i =
                tauri::menu::MenuItem::with_id(app, "stop", "停止直播", true, None::<&str>)?;
            let sep = tauri::menu::PredefinedMenuItem::separator(app)?;
            let quit_i = tauri::menu::MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu =
                tauri::menu::Menu::with_items(app, &[&show_i, &start_i, &stop_i, &sep, &quit_i])?;

            #[cfg(target_os = "macos")]
            let tray_icon =
                tauri::image::Image::from_bytes(include_bytes!("../icons/tray-icon-macos.png"))
                    .expect("Failed to load macOS tray icon");
            #[cfg(not(target_os = "macos"))]
            let tray_icon = app
                .default_window_icon()
                .ok_or("No default window icon set")?
                .clone();

            tauri::tray::TrayIconBuilder::with_id("main-tray")
                .icon(tray_icon)
                .icon_as_template(cfg!(target_os = "macos"))
                .tooltip("Bilibili-Streamer")
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
                    "quit" => {
                        let handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            cleanup_and_exit(handle).await;
                        });
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

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = tokio::signal::ctrl_c().await {
                    tracing::error!("Failed to listen for Ctrl+C: {}", e);
                    return;
                }
                tracing::info!("Ctrl+C received, shutting down gracefully...");
                cleanup_and_exit(handle).await;
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "danmaku-float" {
                    api.prevent_close();
                    let handle = window.app_handle().clone();
                    let window_clone = window.clone();
                    tauri::async_runtime::spawn(async move {
                        if let (Ok(pos), Ok(size)) =
                            (window_clone.outer_position(), window_clone.inner_size())
                        {
                            let x = pos.x as f64;
                            let y = pos.y as f64;
                            let w = size.width as f64;
                            let h = size.height as f64;
                            // Sanity check: reject nonsense values before saving
                            if w > 0.0
                                && h > 0.0
                                && w < 5000.0
                                && h < 5000.0
                                && x.abs() < 10000.0
                                && y.abs() < 10000.0
                            {
                                let state = handle.state::<AppState>();
                                let mut config = state.config.lock().await;
                                config.data_mut().float_window =
                                    Some(bilibili_streamer_lib::models::config::FloatWindowState {
                                        x,
                                        y,
                                        width: w,
                                        height: h,
                                    });
                                let _ = config.save();
                            }
                        }
                        let _ = window_clone.destroy();
                    });
                    return;
                }

                let handle = window.app_handle().clone();
                let window_clone = window.clone();
                api.prevent_close();
                tauri::async_runtime::spawn(async move {
                    let state = handle.state::<AppState>();
                    if state.exiting.load(Ordering::SeqCst) {
                        return;
                    }
                    let config = state.config.lock().await;
                    let min_to_tray = config.data().min_to_tray;
                    drop(config);
                    if min_to_tray {
                        if let Err(e) = window_clone.hide() {
                            tracing::warn!("Failed to hide window: {}", e);
                        }
                    } else {
                        cleanup_and_exit(handle).await;
                    }
                });
            }
        })
        .invoke_handler(tauri::generate_handler![
            bilibili_streamer_lib::commands::auth::get_login_qrcode,
            bilibili_streamer_lib::commands::auth::poll_login_status,
            bilibili_streamer_lib::commands::user::load_saved_config,
            bilibili_streamer_lib::commands::user::refresh_current_user,
            bilibili_streamer_lib::commands::user::get_account_list,
            bilibili_streamer_lib::commands::user::switch_account,
            bilibili_streamer_lib::commands::user::logout,
            bilibili_streamer_lib::commands::user::clear_session,
            bilibili_streamer_lib::commands::live::get_partitions,
            bilibili_streamer_lib::commands::live::update_title,
            bilibili_streamer_lib::commands::live::update_area,
            bilibili_streamer_lib::commands::live::start_live,
            bilibili_streamer_lib::commands::live::stop_live,
            bilibili_streamer_lib::commands::danmaku::start_danmaku_monitor,
            bilibili_streamer_lib::commands::danmaku::stop_danmaku_monitor,
            bilibili_streamer_lib::commands::danmaku::send_danmaku,
            bilibili_streamer_lib::commands::danmaku::get_emote_list,
            bilibili_streamer_lib::commands::window::window_min,
            bilibili_streamer_lib::commands::window::window_max,
            bilibili_streamer_lib::commands::window::window_close,
            bilibili_streamer_lib::commands::window::window_drag,
            bilibili_streamer_lib::commands::window::set_window_background,
            bilibili_streamer_lib::commands::window::open_danmaku_float,
            bilibili_streamer_lib::commands::window::close_danmaku_float,
            bilibili_streamer_lib::commands::config::get_app_config,
            bilibili_streamer_lib::commands::config::set_app_config,
            bilibili_streamer_lib::commands::config::get_version,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::ExitRequested { api, .. } = event {
            let state = app_handle.state::<AppState>();
            if state.exiting.load(Ordering::SeqCst) {
                return;
            }
            api.prevent_exit();
            let handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                cleanup_and_exit(handle).await;
            });
        }
    });
}
