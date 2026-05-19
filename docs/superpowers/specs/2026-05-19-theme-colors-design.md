# Theme Color Update Design

## Overview
Change app theme colors: dark mode to Monokai Pro classic dark, light mode to warm white, and make the macOS title bar blend in.

## Colors
- **Dark background**: `#2d2a2e` (Monokai Pro classic dark)
- **Light background**: `#f7f5f2` (warm white)
- Text and accent colors remain using the existing `stone` scale.

## Changes

### Frontend
1. **`tailwind.config.ts`**: Override `stone.950` to `#2d2a2e`.
2. **`src/styles/globals.css`**: Change `body` light background from `bg-white` to `bg-[#f7f5f2]`.
3. **`src/App.tsx`**: Change top-level `bg-white` to `bg-[#f7f5f2]`.
4. **`src/context/AppContext.tsx` & `src/App.tsx`**: When toggling theme, call `invoke('set_window_background', { r, g, b })` with the matching RGB values.
5. **`src/hooks/useTauri.ts`**: Export helper or add the invoke call inline.

### Tauri Config
6. **`src-tauri/tauri.conf.json`**: Add `transparent: true` and `titleBarStyle: "Transparent"` to the window config (macOS title bar becomes transparent and uses the window background color).

### Rust Backend
7. **`src-tauri/Cargo.toml`**: No new crates needed; use `tauri::webview::Color` and `window.set_background_color()`.
8. **`src-tauri/src/commands/window.rs`**: Add `set_window_background(window, r, g, b)` command. On macOS, call `window.set_background_color(Some(Color { r, g, b, a: 255 }))`.
9. **`src-tauri/src/main.rs`**: Register the new command in `generate_handler!`.

## Verification
- App dark mode shows `#2d2a2e` background.
- App light mode shows `#f7f5f2` background.
- macOS title bar color matches the current mode background.
- Theme toggle updates both CSS and title bar instantly.
