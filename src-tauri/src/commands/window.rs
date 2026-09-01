use crate::services::float_geometry::{self, WindowGeometry};
use crate::state::AppState;
use tauri::Manager;

#[tauri::command]
pub fn window_min(window: tauri::Window) {
    let _ = window.minimize();
}

#[tauri::command]
pub fn window_max(window: tauri::Window) -> Result<bool, String> {
    let is_max = window.is_maximized().map_err(|e| e.to_string())?;
    if is_max {
        window.unmaximize().map_err(|e| e.to_string())?;
    } else {
        window.maximize().map_err(|e| e.to_string())?;
    }
    Ok(!is_max)
}

#[tauri::command]
pub fn window_close(window: tauri::Window) {
    let _ = window.close();
}

#[tauri::command]
pub fn window_drag(window: tauri::Window, _x: i32, _y: i32) {
    let _ = window.start_dragging();
}

#[tauri::command]
pub fn set_window_background(
    window: tauri::Window,
    r: u8,
    g: u8,
    b: u8,
    a: Option<u8>,
    dark: bool,
) {
    let alpha = a.unwrap_or(255);
    let _ = window.set_background_color(Some(tauri::window::Color(r, g, b, alpha)));
    let theme = if dark {
        Some(tauri::Theme::Dark)
    } else {
        Some(tauri::Theme::Light)
    };
    let _ = window.set_theme(theme);
}

#[tauri::command]
pub async fn open_danmaku_float(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // If already open, focus and return
    if let Some(win) = app_handle.get_webview_window("danmaku-float") {
        let _ = win.set_focus();
        return Ok(());
    }

    let config = state.config.lock().await;
    let saved = config.data().float_window.clone();
    drop(config);

    let restore = float_geometry::restore_geometry(saved.as_ref(), &app_handle);

    // Build the window with a logical fallback size. The physical size will be
    // applied after creation so that restored values (which are physical) are
    // respected regardless of the monitor's scale factor.
    let mut builder = tauri::WebviewWindowBuilder::new(
        &app_handle,
        "danmaku-float",
        tauri::WebviewUrl::App("/".into()),
    )
    .title("Monitor")
    .decorations(true)
    .always_on_top(true)
    .skip_taskbar(true)
    .transparent(true)
    .shadow(false)
    .resizable(true)
    .inner_size(320.0, 450.0);

    #[cfg(target_os = "macos")]
    {
        builder = builder.title_bar_style(tauri::TitleBarStyle::Transparent);
    }

    let window = builder.build().map_err(|e| e.to_string())?;

    // Restore size first so that center() works with the correct dimensions.
    let _ = window.set_size(tauri::Size::Physical(restore.size));

    // Restore position if it is still visible; otherwise center on the primary monitor.
    if let Some(position) = restore.position {
        let _ = window.set_position(tauri::Position::Physical(position));
    } else {
        let _ = window.center();
    }

    Ok(())
}

#[tauri::command]
pub async fn close_danmaku_float(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let Some(win) = app_handle.get_webview_window("danmaku-float") else {
        return Ok(());
    };

    // Read current geometry and persist it through the shared module so both
    // write paths apply identical validation.
    let (x, y, w, h) = win.float_geometry().map_err(|e| e.to_string())?;
    float_geometry::save_geometry(&state.config, x, y, w, h).await;

    // Destroy the window directly to avoid re-triggering CloseRequested
    let _ = win.destroy();
    Ok(())
}
