//! Owns the Monitor (danmaku-float) window geometry lifecycle: reading,
//! validating, persisting, and restoring. All write paths delegate here so
//! validation exists exactly once.

use crate::models::config::FloatWindowState;
use crate::services::config_store::ConfigStore;

/// Sanity bounds for persisted geometry: positive, non-absurd size and
/// position. Rejects values that would restore the window off-screen or
/// with a degenerate size.
pub fn is_valid_geometry(x: f64, y: f64, width: f64, height: f64) -> bool {
    width > 0.0
        && height > 0.0
        && width < 5000.0
        && height < 5000.0
        && x.abs() < 10000.0
        && y.abs() < 10000.0
}

/// Uniform geometry reading for `tauri::Window` and `tauri::WebviewWindow`.
pub trait WindowGeometry {
    /// Returns (x, y, width, height): outer position and inner size as f64.
    fn float_geometry(&self) -> tauri::Result<(f64, f64, f64, f64)>;
}

macro_rules! impl_window_geometry {
    ($ty:ty) => {
        impl WindowGeometry for $ty {
            fn float_geometry(&self) -> tauri::Result<(f64, f64, f64, f64)> {
                let pos = self.outer_position()?;
                let size = self.inner_size()?;
                Ok((
                    pos.x as f64,
                    pos.y as f64,
                    size.width as f64,
                    size.height as f64,
                ))
            }
        }
    };
}

impl_window_geometry!(tauri::Window);
impl_window_geometry!(tauri::WebviewWindow);

/// Check whether a window with the given outer position and size
/// overlaps any available monitor.
pub fn is_position_visible(
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

/// Validate and persist the Monitor window geometry. Returns whether the
/// values were accepted and saved.
pub async fn save_geometry(
    config: &tokio::sync::Mutex<ConfigStore>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> bool {
    if !is_valid_geometry(x, y, width, height) {
        return false;
    }
    let mut config = config.lock().await;
    config.data_mut().float_window = Some(FloatWindowState {
        x,
        y,
        width,
        height,
    });
    let _ = config.save();
    true
}

/// Size and optional position to restore the Monitor window with.
pub struct RestoreGeometry {
    pub size: tauri::PhysicalSize<u32>,
    pub position: Option<tauri::PhysicalPosition<i32>>,
}

/// Compute the restore target for the Monitor window: the saved size clamped
/// to sane bounds (or the 640x900 physical default, matching the logical
/// 320x450 default at a 2x scale factor), and the saved position only if it
/// still overlaps a connected monitor.
pub fn restore_geometry(
    saved: Option<&FloatWindowState>,
    app_handle: &tauri::AppHandle,
) -> RestoreGeometry {
    let size = saved
        .map(|s| tauri::PhysicalSize {
            width: s.width.clamp(200.0, 800.0) as u32,
            height: s.height.clamp(200.0, 1200.0) as u32,
        })
        .unwrap_or(tauri::PhysicalSize {
            width: 640,
            height: 900,
        });
    let position = saved
        .filter(|s| {
            is_position_visible(s.x, s.y, size.width as f64, size.height as f64, app_handle)
        })
        .map(|s| tauri::PhysicalPosition {
            x: s.x as i32,
            y: s.y as i32,
        });
    RestoreGeometry { size, position }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_geometry() {
        assert!(is_valid_geometry(100.0, 200.0, 640.0, 900.0));
        assert!(is_valid_geometry(-500.0, -500.0, 320.0, 450.0));
    }

    #[test]
    fn rejects_zero_or_negative_size() {
        assert!(!is_valid_geometry(0.0, 0.0, 0.0, 900.0));
        assert!(!is_valid_geometry(0.0, 0.0, 640.0, 0.0));
        assert!(!is_valid_geometry(0.0, 0.0, -10.0, 900.0));
        assert!(!is_valid_geometry(0.0, 0.0, 640.0, -10.0));
    }

    #[test]
    fn rejects_oversized_geometry() {
        assert!(!is_valid_geometry(0.0, 0.0, 5000.0, 900.0));
        assert!(!is_valid_geometry(0.0, 0.0, 640.0, 5000.0));
        assert!(!is_valid_geometry(0.0, 0.0, 8000.0, 900.0));
    }

    #[test]
    fn rejects_off_screen_position() {
        assert!(!is_valid_geometry(10000.0, 0.0, 640.0, 900.0));
        assert!(!is_valid_geometry(-10000.0, 0.0, 640.0, 900.0));
        assert!(!is_valid_geometry(0.0, 15000.0, 640.0, 900.0));
        assert!(!is_valid_geometry(0.0, -15000.0, 640.0, 900.0));
    }
}
