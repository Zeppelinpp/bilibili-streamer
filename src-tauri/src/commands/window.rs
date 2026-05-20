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
pub fn set_window_background(window: tauri::Window, r: u8, g: u8, b: u8, is_dark: bool) {
    let _ = window.set_background_color(Some(tauri::window::Color(r, g, b, 255)));
    let theme = if is_dark {
        tauri::Theme::Dark
    } else {
        tauri::Theme::Light
    };
    let _ = window.set_theme(Some(theme));

    #[cfg(target_os = "macos")]
    unsafe {
        use objc::runtime::{Class, Object, YES};
        use objc::{msg_send, sel, sel_impl};

        if let Ok(ns_window_ptr) = window.ns_window() {
            let ns_window = ns_window_ptr as *mut Object;

            // 1. Set NSWindow appearance directly
            let appearance_name = if is_dark {
                "NSAppearanceNameDarkAqua\0"
            } else {
                "NSAppearanceNameAqua\0"
            };
            let ns_string_cls = Class::get("NSString").expect("NSString not found");
            let ns_string: *mut Object =
                msg_send![ns_string_cls, stringWithUTF8String: appearance_name.as_ptr()];
            let appearance_cls = Class::get("NSAppearance").expect("NSAppearance not found");
            let appearance: *mut Object =
                msg_send![appearance_cls, appearanceNamed: ns_string];
            let () = msg_send![ns_window, setAppearance: appearance];

            // 2. Force titlebar to re-read appearance by toggling titlebarAppearsTransparent
            let current_style: u64 = msg_send![ns_window, styleMask];
            let titlebar_transparent_mask: u64 = 1 << 11; // NSWindowStyleMaskFullSizeContentView = 1 << 15, titlebarAppearsTransparent is a property not styleMask
            // Actually titlebarAppearsTransparent is a property, let's toggle it
            let was_transparent: bool = msg_send![ns_window, titlebarAppearsTransparent];
            let () = msg_send![ns_window, setTitlebarAppearsTransparent: false];
            let () = msg_send![ns_window, setTitlebarAppearsTransparent: was_transparent];

            // 3. Force redraw
            let content_view: *mut Object = msg_send![ns_window, contentView];
            let () = msg_send![content_view, setNeedsDisplay: YES];
            let () = msg_send![ns_window, display];
        }
    }
}
