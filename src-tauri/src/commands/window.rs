use crate::state::AppState;
use tauri::Manager;

#[cfg(target_os = "macos")]
use objc::runtime::{Object, BOOL, NO, YES};
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};

#[cfg(target_os = "macos")]
unsafe fn apply_corner_radius(view: *mut Object, clear_color: *mut Object, radius: f64) {
    let _: () = msg_send![view, setWantsLayer: YES];
    let layer: *mut Object = msg_send![view, layer];
    if !layer.is_null() {
        let cg_color: *mut Object = msg_send![clear_color, CGColor];
        let _: () = msg_send![layer, setBackgroundColor: cg_color];
        let _: () = msg_send![layer, setCornerRadius: radius];
        let _: () = msg_send![layer, setMasksToBounds: YES];
    }
    let subviews: *mut Object = msg_send![view, subviews];
    let count: usize = msg_send![subviews, count];
    for i in 0..count {
        let subview: *mut Object = msg_send![subviews, objectAtIndex: i];
        apply_corner_radius(subview, clear_color, radius);
    }
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
pub fn set_window_background(window: tauri::Window, r: u8, g: u8, b: u8) {
    let _ = window.set_background_color(Some(tauri::window::Color(r, g, b, 255)));
}

#[tauri::command]
pub async fn open_danmaku_float(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_ , AppState>,
) -> Result<(), String> {
    // If already open, focus and return
    if let Some(win) = app_handle.get_webview_window("danmaku-float") {
        let _ = win.set_focus();
        return Ok(());
    }

    // Read saved state or use defaults
    let config = state.config.lock().await;
    let (width, height, x, y) = config
        .data()
        .float_window
        .as_ref()
        .map(|f| (f.width, f.height, f.x, f.y))
        .unwrap_or((320.0, 450.0, -1.0, -1.0));
    drop(config);

    let mut builder = tauri::WebviewWindowBuilder::new(
        &app_handle,
        "danmaku-float",
        tauri::WebviewUrl::App("/".into()),
    )
    .title("Monitor")
    .decorations(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .transparent(true)
    .shadow(false)
    .resizable(false)
    .inner_size(width, height);

    // Only set position if we have a saved state (x >= 0 && y >= 0); otherwise center
    if x >= 0.0 && y >= 0.0 {
        let clamped_x = x.max(0.0);
        let clamped_y = y.max(25.0);
        builder = builder.position(clamped_x, clamped_y);
    } else {
        // Manually center on the primary monitor to avoid platform quirks with .center()
        if let Ok(Some(monitor)) = app_handle.primary_monitor() {
            let m_pos = monitor.position();
            let m_size = monitor.size();
            let m_x = m_pos.x as f64;
            let m_y = m_pos.y as f64;
            let m_w = m_size.width as f64;
            let m_h = m_size.height as f64;
            let center_x = m_x + (m_w - width) / 2.0;
            let center_y = m_y + (m_h - height) / 2.0;
            builder = builder.position(center_x, center_y);
        }
    }

    let win = builder.build().map_err(|e| e.to_string())?;

    #[cfg(target_os = "macos")]
    {
        let ns_window = win.ns_window().map_err(|e| e.to_string())? as *mut Object;
        let ns_view = win.ns_view().map_err(|e| e.to_string())? as *mut Object;

        unsafe {
            let clear_color: *mut Object =
                msg_send![objc::runtime::Class::get("NSColor").unwrap(), clearColor];

            let _: () = msg_send![ns_window, setBackgroundColor: clear_color];
            let _: () = msg_send![ns_window, setOpaque: NO];
            let _: () = msg_send![ns_window, setHasShadow: NO];

            let _: () = msg_send![ns_view, setOpaque: NO];
            let sel_set_draws_bg = sel!(setDrawsBackground:);
            let responds: BOOL = msg_send![ns_view, respondsToSelector: sel_set_draws_bg];
            if responds != NO {
                let _: () = msg_send![ns_view, setDrawsBackground: NO];
            }

            // Apply corner radius + clipping to every layer in the view hierarchy
            // so WKWebView's internal compositing layers are also clipped.
            apply_corner_radius(ns_view, clear_color, 12.0);

            let content_view: *mut Object = msg_send![ns_window, contentView];
            if !content_view.is_null() {
                apply_corner_radius(content_view, clear_color, 12.0);
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn close_danmaku_float(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_ , AppState>,
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

    // Close the window
    let _ = win.close();
    Ok(())
}
