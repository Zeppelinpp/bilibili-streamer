# Danmaku Floating Window Design Spec

## Overview

A compact, draggable, always-on-top floating window that displays live danmaku (chat messages) from Bilibili. The user can open it from the sidebar, drag it anywhere on screen, and keep it visible even when the main app is hidden in the tray or when focus is on another application.

## Goals

- Provide a persistent, compact danmaku overlay for streamers who want to monitor chat while using other apps.
- Reuse existing danmaku data pipeline — no backend changes to `DanmakuService`.
- Remember window position and size across app restarts.

## Architecture

### Approach: Shared Bundle, Second WebView Window

Create a single additional Tauri WebView window (`label: "danmaku-float"`) that loads the same frontend bundle (`index.html`). The frontend determines which UI to render by checking the current window label at runtime.

This avoids a second build target, separate entry point, or duplicated React tree management. Both windows consume the same globally-emitted `danmu-message` Tauri events.

### macOS Always-On-Top

Tauri 2.x `WindowBuilder::always_on_top(true)` sets the macOS `NSWindow.level` to `NSFloatingWindowLevel`. The window remains visible above other applications when they receive focus. No extra system permissions are required for this window level.

## Components

### Frontend

#### `DanmakuFloat.tsx` (new)

A compact overlay component (~320×450px by default) containing:

- **Custom title bar / drag handle** (`-webkit-app-region: drag`) with:
  - Left: "弹幕监控" label
  - Right: minimize (optional) and close buttons (`-webkit-app-region: no-drag`)
- **Scrollable danmaku list** — reuses the same bubble rendering logic as `DanmakuPanel`:
  - Self-messages (orange, right-aligned)
  - Gift messages (amber background)
  - Regular messages (left-aligned, slight transparency)
  - Interact events (centered, muted text)
  - Emoji parsing via `parseMessage` + `FALLBACK_EMOJI_MAP`
- **Compact input bar** at the bottom:
  - Text input + send button
  - Calls existing `sendDanmaku()` invoke

Styling:
- Slight transparency on the message list background so the desktop behind is faintly visible
- Follows system dark/light preference via `prefers-color-scheme` (no explicit toggle in the float window)
- No console button, no theme toggle, no sidebar

#### Shared Extract: `utils/danmaku.ts` (new)

Move `parseMessage()` and `FALLBACK_EMOJI_MAP` from `DanmakuPanel.tsx` into a shared utility so both `DanmakuPanel` and `DanmakuFloat` can import them.

#### `App.tsx` (modified)

Branch inside the `AppProvider` tree so both windows share the same contexts:

```tsx
function AppInner() {
  const label = WebviewWindow.getCurrent().label;
  if (label === 'danmaku-float') return <DanmakuFloat />;
  return <AppContent />;
}

function App() {
  return (
    <AppProvider>
      <AppInner />
    </AppProvider>
  );
}
```

Both windows get their own `DanmakuProvider` instance, so each maintains its own `danmakuList` state. Tauri events are broadcast to all windows, so both receive `danmu-message` independently.

> **Note:** `WebviewWindow.getCurrent()` is synchronous in Tauri 2.x.

#### `Sidebar.tsx` (modified)

Add a small icon button to the right of the "弹幕监控" nav item (inside the same row). The button is only visible when the "弹幕监控" tab is active. On hover, a tooltip shows "浮窗". Clicking it calls `openDanmakuFloat()`.

Only one float window is allowed at a time. If already open, the click can either focus the existing window or be a no-op.

### Backend

#### `commands/window.rs` (modified)

Add two new commands:

- `open_danmaku_float(app_handle, state)`:
  1. Check if a window with label `"danmaku-float"` already exists. If yes, focus it and return.
  2. Read saved position/size from `AppState.config` (or use defaults: 320×450, centered).
  3. Build the window:
     ```rust
     tauri::WebviewWindowBuilder::new(
         &app_handle,
         "danmaku-float",
         tauri::WebviewUrl::App("/".into())
     )
     .decorations(false)
     .always_on_top(true)
     .skip_taskbar(true)
     .transparent(true)
     .inner_size(width, height)
     .position(x, y)
     .build()?;
     ```
  4. Store the window handle if needed.

- `close_danmaku_float(app_handle, state)`:
  1. If the float window exists, read its current position and size.
  2. Save to `ConfigStore`.
  3. Close the window.

#### `models/config.rs` (modified)

Add to `AppConfig`:

```rust
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct FloatWindowState {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
```

And add `pub float_window: Option<FloatWindowState>` to `AppConfig`.

#### `hooks/useTauri.ts` (modified)

Add frontend wrappers:

```ts
export async function openDanmakuFloat(): Promise<void> {
  return await invoke('open_danmaku_float');
}

export async function closeDanmakuFloat(): Promise<void> {
  return await invoke('close_danmaku_float');
}
```

## Data Flow

```
Bilibili Danmaku WS
        │
        ▼
┌─────────────────┐
│ DanmakuService  │─── tokio::spawn WS loop
│ (existing)      │
└─────────────────┘
        │
        │ app_handle.emit("danmu-message", payload)
        ▼
┌──────────────────────────────────────────┐
│           Tauri Event Bus                │
└──────────────────────────────────────────┘
        │                       │
        ▼                       ▼
┌──────────────┐      ┌──────────────────┐
│  main window │      │ danmaku-float    │
│ DanmakuPanel │      │ DanmakuFloat     │
│   (listen)   │      │    (listen)      │
└──────────────┘      └──────────────────┘
```

- No new event types or backend channels.
- Both windows receive identical `danmu-message` payloads.
- Sending a danmaku from either window uses the same `send_danmaku` invoke path.

## Window Behavior & Lifecycle

| Scenario | Behavior |
|---|---|
| Click "浮窗" in sidebar | Open float window at saved position/size (fallback: 320×450, screen-centered). If already open, focus it. |
| Click × on float title bar | Close float window. Persist current position/size to config. |
| Drag float window by title bar | Native drag via `-webkit-app-region: drag`. Position is saved only on close. |
| Main window closes to tray | Float window remains visible (independent lifecycle). |
| App quits (tray → Quit / Cmd+Q) | Both windows close; existing cleanup flow stops live stream. |

## Error Handling

| Error | Handling |
|---|---|
| Float window already open | Focus existing window; do not create duplicate. |
| Config read failure on open | Use default position/size (320×450, centered). |
| Config write failure on close | Silent fail; window closes normally. Position is lost for next launch. |
| Window creation failure | Log error via tracing; show no UI (fail gracefully). |

## Out of Scope

- Multiple simultaneous float windows (explicitly limited to one).
- Resizing the float window by dragging edges (can be added later; initial scope is fixed size or title-bar-only drag).
- Custom opacity slider (fixed slight transparency; no user control).
- Pin/unpin behavior (always pinned / always-on-top while open).

## Files to Touch

| File | Change |
|---|---|
| `src/components/DanmakuFloat.tsx` | New — compact float window UI |
| `src/components/Sidebar.tsx` | Add float-launch icon button |
| `src/App.tsx` | Branch on window label |
| `src/utils/danmaku.ts` | New — shared `parseMessage` + emoji map |
| `src/hooks/useTauri.ts` | Add `openDanmakuFloat`, `closeDanmakuFloat` |
| `src-tauri/src/commands/window.rs` | Add `open_danmaku_float`, `close_danmaku_float` |
| `src-tauri/src/models/config.rs` | Add `FloatWindowState` to `AppConfig` |
| `src-tauri/src/main.rs` | Register new commands in `invoke_handler` |
| `src/components/DanmakuPanel.tsx` | Refactor to import shared utils |
