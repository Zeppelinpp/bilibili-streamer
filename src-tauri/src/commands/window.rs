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

/// Return the primary monitor's center position for a window of the given size.
fn center_on_primary_monitor(
    width: f64,
    height: f64,
    app_handle: &tauri::AppHandle,
) -> Option<(f64, f64)> {
    let m = app_handle.primary_monitor().ok().flatten()?;
    let pos = m.position();
    let size = m.size();
    Some((
        pos.x as f64 + ((size.width as f64) - width).max(0.0) / 2.0,
        pos.y as f64 + ((size.height as f64) - height).max(0.0) / 2.0,
    ))
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
pub fn set_window_background(window: tauri::Window, r: u8, g: u8, b: u8, a: Option<u8>) {
    let alpha = a.unwrap_or(255);
    let _ = window.set_background_color(Some(tauri::window::Color(r, g, b, alpha)));
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

    // Read saved state or use defaults
    let config = state.config.lock().await;
    let saved = config.data().float_window.clone();
    drop(config);

    let (mut phys_width, mut phys_height) = saved
        .as_ref()
        .map(|f| (f.width, f.height))
        .unwrap_or((640.0, 900.0));

    // Clamp to reasonable bounds
    phys_width = phys_width.clamp(200.0, 800.0);
    phys_height = phys_height.clamp(200.0, 1200.0);

    // Build with a default logical size; we restore the physical size after creation
    // because builder.inner_size/position accept logical pixels, while saved values
    // from inner_size()/outer_position() are physical pixels.
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

    // Use saved position only if it is still visible on some monitor.
    // Otherwise fall back to centering on the primary monitor.
    let pos = saved.as_ref().and_then(|s| {
        if is_position_visible(s.x, s.y, phys_width, phys_height, &app_handle) {
            Some((s.x, s.y))
        } else {
            None
        }
    });

    if let Some((x, y)) = pos {
        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: x as i32,
            y: y as i32,
        }));
    } else if let Some((cx, cy)) =
        center_on_primary_monitor(phys_width, phys_height, &app_handle)
    {
        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: cx as i32,
            y: cy as i32,
        }));
    }

    // Restore physical size only when we have a saved state;
    // otherwise leave the builder's default logical size in place.
    if saved.is_some() {
        let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: phys_width as u32,
            height: phys_height as u32,
        }));
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
