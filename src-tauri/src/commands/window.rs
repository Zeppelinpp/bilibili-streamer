use crate::state::AppState;
use tauri::Manager;

/// Check whether a window with the given outer position and size
/// overlaps any available monitor.
fn is_position_visible(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    app_handle: &tauri::AppHandle,
) -> bool {
    let Ok(monitors) = app_handle.available_monitors() else {
        return false;
    };
    monitors.iter().any(|m| {
        let pos = m.position();
        let size = m.size();
        let ml = pos.x as f64;
        let mt = pos.y as f64;
        let mr = ml + size.width as f64;
        let mb = mt + size.height as f64;
        let wr = x + width;
        let wb = y + height;
        // Standard AABB overlap check
        wr > ml && x < mr && wb > mt && y < mb
    })
}

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

    // Decide target physical size. When there is no saved state we use 640×900
    // physical pixels, which matches the logical default of 320×450 at a 2×
    // scale factor (the primary target platform).
    let target_size = saved
        .as_ref()
        .map(|s| tauri::PhysicalSize {
            width: s.width.clamp(200.0, 800.0) as u32,
            height: s.height.clamp(200.0, 1200.0) as u32,
        })
        .unwrap_or(tauri::PhysicalSize {
            width: 640,
            height: 900,
        });

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
    let _ = window.set_size(tauri::Size::Physical(target_size));

    // Restore position if it is still visible; otherwise center on the primary monitor.
    let visible = saved.as_ref().is_some_and(|s| {
        is_position_visible(
            s.x,
            s.y,
            target_size.width as f64,
            target_size.height as f64,
            &app_handle,
        )
    });

    if visible {
        let s = saved.unwrap(); // safe: we just checked is_some_and
        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: s.x as i32,
            y: s.y as i32,
        }));
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

    // Read current position and size
    let pos = win.outer_position().map_err(|e| e.to_string())?;
    let size = win.inner_size().map_err(|e| e.to_string())?;

    // Save to config
    let mut config = state.config.lock().await;
    config.data_mut().float_window = Some(crate::models::config::FloatWindowState {
        x: pos.x as f64,
        y: pos.y as f64,
        width: size.width as f64,
        height: size.height as f64,
    });
    let _ = config.save();
    drop(config);

    // Destroy the window directly to avoid re-triggering CloseRequested
    let _ = win.destroy();
    Ok(())
}
