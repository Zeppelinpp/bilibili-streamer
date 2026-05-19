# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

BiliLiveTool — a Bilibili live streaming desktop app. **Tauri 2.x** (Rust + WebView) with a React 18 + TypeScript + Vite + Tailwind CSS frontend.

## Commands

- `npm run tauri-dev` — dev mode (Vite on :1420 + Rust compile + WebView window)
- `npm run tauri-build` — production build with platform installer bundle
- `npm run dev` — frontend only
- `npm run build` — frontend assets only

No tests exist in this project.

## Mental Model

### Frontend

- **State**: 4 independent React Contexts in `src/context/AppContext.tsx` — `UserContext`, `LiveContext`, `DanmakuContext`, `UIContext`. Split to avoid unrelated re-renders.
- **Tauri calls**: Centralized in `src/hooks/useTauri.ts`. Never call `invoke` directly from components.
- **Events**: Frontend listens to Rust-emitted events: `danmu-message`, `danmu-disconnected`, `tray-start-live`, `tray-stop-live`.
- **Theme**: Tailwind `darkMode: 'class'`. Toggle by adding/removing `dark` on `document.documentElement`.

### Backend

- **Commands** (`src-tauri/src/commands/`): Tauri commands exposed to frontend. Each command acquires locks from `AppState`.
- **Lock ordering** (critical, prevents deadlocks): `api -> config -> session -> live -> danmaku`. Acquire in this order, drop promptly.
- **AppState** (`state.rs`): Injected via `manage()`. Holds `ConfigStore`, `SessionState`, `Arc<Mutex<BiliApi>>`, `DanmakuService`, `LiveService`, and an `exiting` atomic flag.
- **BiliApi** (`services/bili_api.rs`): Single HTTP client for all Bilibili APIs. Handles cookies, App Sign (MD5 + salt), and WBI signing.
- **DanmakuService** (`services/danmaku_ws.rs`): WebSocket to Bilibili danmaku server via `tokio_tungstenite`. Inner WS loop runs in `tokio::spawn`; outer command loop runs in `tauri::async_runtime::spawn`.

### macOS Threading

`.setup()` and `on_window_event` run on the macOS main thread — **no Tokio runtime**. Never use `block_on` or `block_in_place` there; always spawn with `tauri::async_runtime::spawn`.

