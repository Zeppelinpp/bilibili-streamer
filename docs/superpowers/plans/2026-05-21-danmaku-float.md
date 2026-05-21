# Danmaku Floating Window Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a compact, draggable, always-on-top floating window that mirrors the main window's danmaku feed, launched from the sidebar.

**Architecture:** A second Tauri WebView window (`label: "danmaku-float"`) loads the same frontend bundle. The frontend branches on `getCurrentWebviewWindow().label` to render either the full app or the compact float UI. Both windows consume the same globally-emitted `danmu-message` events from the existing `DanmakuService`. Window position/size is persisted to the existing `ConfigStore`.

**Tech Stack:** Tauri 2.x (Rust), React 18 + TypeScript + Tailwind CSS, `@tauri-apps/api` 2.11.0

---

## File Map

| File | Responsibility |
|---|---|
| `src/utils/danmaku.ts` *(new)* | Shared `parseMessage()` + `FALLBACK_EMOJI_MAP` extracted from `DanmakuPanel` |
| `src/components/DanmakuFloat.tsx` *(new)* | Compact float window UI: drag bar, danmaku list, input bar |
| `src/components/DanmakuPanel.tsx` *(modify)* | Refactor to import shared utils from `@/utils/danmaku` |
| `src/App.tsx` *(modify)* | Branch inside `AppProvider` to render `DanmakuFloat` when window label is `danmaku-float` |
| `src/components/Sidebar.tsx` *(modify)* | Add icon button next to "弹幕监控" nav item to open the float window |
| `src/hooks/useTauri.ts` *(modify)* | Add `openDanmakuFloat()` and `closeDanmakuFloat()` wrappers |
| `src-tauri/src/models/config.rs` *(modify)* | Add `FloatWindowState` struct and `float_window` field to `AppConfig` |
| `src-tauri/src/commands/window.rs` *(modify)* | Add `open_danmaku_float` and `close_danmaku_float` commands |
| `src-tauri/src/main.rs` *(modify)* | Register new commands in `invoke_handler` |

---

### Task 1: Extract shared danmaku utilities

**Files:**
- Create: `src/utils/danmaku.ts`
- Modify: `src/components/DanmakuPanel.tsx`

- [ ] **Step 1: Create shared utility file**

Create `src/utils/danmaku.ts` with the exact code extracted from `DanmakuPanel.tsx`:

```ts
import type { ReactNode } from 'react';

export const FALLBACK_EMOJI_MAP: Record<string, string> = {
  dog: 'https://i0.hdslb.com/bfs/emote/3087d273a78ccaff4bb1e9972e2ba2a7583c9f11.png',
  妙啊: '👍',
  吃瓜: '🍉',
  呲牙: '😁',
  打call: '📣',
  酸了: '🍋',
  大哭: '😭',
  喜极而泣: '😂',
  笑哭: '😂',
  偷笑: '🤭',
  爱心: '❤️',
  胜利: '✌️',
  保佑: '🙏',
  灵魂出窍: '😇',
  OK: '👌',
  点赞: '👍',
  捂脸: '🤦',
  尴尬: '😅',
  黑洞: '🕳️',
  跪了: '🧎',
  给心心: '🫶',
  惊讶: '😲',
  再见: '👋',
  惊喜: '🤩',
  鼓掌: '👏',
};

export function parseMessage(msg: string, emoteMap: Record<string, string>): ReactNode[] {
  const segments: ReactNode[] = [];
  const regex = /\[([^\]]+)\]/g;
  let lastIndex = 0;
  let match: RegExpExecArray | null;
  let key = 0;

  while ((match = regex.exec(msg)) !== null) {
    const textBefore = msg.slice(lastIndex, match.index);
    if (textBefore) {
      segments.push(<span key={key++}>{textBefore}</span>);
    }

    const code = match[1];
    const fullCode = `[${code}]`;
    const url = emoteMap[fullCode];
    if (url && url.startsWith('http')) {
      segments.push(
        <img
          key={key++}
          src={url}
          alt={fullCode}
          className="inline-block w-5 h-5 align-text-bottom"
          loading="lazy"
        />
      );
    } else if (FALLBACK_EMOJI_MAP[code]) {
      const fb = FALLBACK_EMOJI_MAP[code];
      if (fb.startsWith('http')) {
        segments.push(
          <img
            key={key++}
            src={fb}
            alt={fullCode}
            className="inline-block w-5 h-5 align-text-bottom"
            loading="lazy"
          />
        );
      } else {
        segments.push(<span key={key++}>{fb}</span>);
      }
    } else {
      segments.push(<span key={key++}>{fullCode}</span>);
    }

    lastIndex = regex.lastIndex;
  }

  const textAfter = msg.slice(lastIndex);
  if (textAfter) {
    segments.push(<span key={key++}>{textAfter}</span>);
  }

  return segments;
}
```

- [ ] **Step 2: Refactor DanmakuPanel to import shared utils**

In `src/components/DanmakuPanel.tsx`:

1. Remove the local `FALLBACK_EMOJI_MAP` and `parseMessage` definitions (lines 10-93).
2. Add import at the top:
   ```ts
   import { parseMessage, FALLBACK_EMOJI_MAP } from '@/utils/danmaku';
   ```
3. Keep the `getEmoteList` call logic that merges API emotes with `FALLBACK_EMOJI_MAP` — it already passes the merged map to `parseMessage`, so no change needed there.

Verify the file still compiles by checking that `parseMessage` and `FALLBACK_EMOJI_MAP` are no longer defined locally and are imported instead.

- [ ] **Step 3: Commit**

```bash
git add src/utils/danmaku.ts src/components/DanmakuPanel.tsx
git commit -m "refactor: extract shared danmaku parsing utilities"
```

---

### Task 2: Create DanmakuFloat component

**Files:**
- Create: `src/components/DanmakuFloat.tsx`

- [ ] **Step 1: Implement the compact float window component**

Create `src/components/DanmakuFloat.tsx`:

```tsx
import { useEffect, useRef, useState } from 'react';
import { useDanmaku, useUI, useUser } from '@/context/AppContext';
import { sendDanmaku, closeDanmakuFloat, getEmoteList } from '@/hooks/useTauri';
import { Send, Trash2, X } from 'lucide-react';
import { parseMessage, FALLBACK_EMOJI_MAP } from '@/utils/danmaku';
import { invoke } from '@tauri-apps/api/core';

export default function DanmakuFloat() {
  const { danmakuList, clearDanmaku } = useDanmaku();
  const { addLog } = useUI();
  const { user } = useUser();
  const [input, setInput] = useState('');
  const [emoteMap, setEmoteMap] = useState<Record<string, string>>({});
  const scrollRef = useRef<HTMLDivElement>(null);
  const isAtBottomRef = useRef(true);

  useEffect(() => {
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const applyTheme = (e: MediaQueryList | MediaQueryListEvent) => {
      if (e.matches) {
        document.documentElement.classList.add('dark');
        invoke('set_window_background', { r: 45, g: 42, b: 46 }).catch(() => {});
      } else {
        document.documentElement.classList.remove('dark');
        invoke('set_window_background', { r: 247, g: 245, b: 242 }).catch(() => {});
      }
    };
    applyTheme(mq);
    mq.addEventListener('change', applyTheme);
    return () => mq.removeEventListener('change', applyTheme);
  }, []);

  useEffect(() => {
    if (!user) return;
    getEmoteList()
      .then((map) => {
        setEmoteMap(map);
        if (Object.keys(map).length === 0) {
          addLog('[表情] 未获取到官方表情，将使用 unicode 兜底');
        }
      })
      .catch((e) => {
        addLog(`[表情] 获取官方表情失败: ${e}`);
      });
  }, [user, addLog]);

  useEffect(() => {
    if (scrollRef.current && isAtBottomRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [danmakuList]);

  const handleScroll = () => {
    const el = scrollRef.current;
    if (!el) return;
    isAtBottomRef.current = el.scrollHeight - el.scrollTop - el.clientHeight < 30;
  };

  const handleSend = async () => {
    if (!input.trim()) return;
    try {
      const res = await sendDanmaku(input.trim());
      if (res.code !== 0) {
        addLog(`[弹幕] 发送失败: ${res.msg}`);
      }
      if (res.code === 0) setInput('');
    } catch (e: any) {
      addLog(`[弹幕] 发送失败: ${e}`);
    }
  };

  const handleClose = async () => {
    try {
      await closeDanmakuFloat();
    } catch (e: any) {
      addLog(`[浮窗] 关闭失败: ${e}`);
    }
  };

  return (
    <div className="flex flex-col h-screen bg-stone-950/95 text-stone-200 overflow-hidden select-none">
      {/* Drag handle / title bar */}
      <div
        className="flex items-center justify-between px-3 h-7 shrink-0 bg-stone-900/80 border-b border-stone-800"
        style={{ WebkitAppRegion: 'drag' }}
      >
        <span className="text-[11px] font-medium text-stone-400">Monitor</span>
        <div className="flex items-center gap-1" style={{ WebkitAppRegion: 'no-drag' }}>
          <button
            onClick={handleClose}
            className="w-5 h-5 rounded flex items-center justify-center text-stone-500 hover:text-stone-200 hover:bg-stone-800 transition"
            title="关闭"
          >
            <X size={11} />
          </button>
        </div>
      </div>

      {/* Danmaku list */}
      <div
        ref={scrollRef}
        onScroll={handleScroll}
        className="flex-1 overflow-y-auto px-3 py-2 space-y-1"
      >
        {danmakuList.map((item) => {
          const isSelf = item.data.is_self;
          if (item.data.type === 'interact') {
            const uname = item.data.uname || '';
            const rest = (item.data.msg || '').replace(uname, '').trimStart();
            return (
              <div key={item.id} className="flex justify-center py-1 px-2">
                <span className="text-[11px] text-stone-500">
                  {uname && (
                    <span className="font-medium text-stone-300">{uname}</span>
                  )}
                  {uname && ' '}
                  {rest}
                </span>
              </div>
            );
          }
          let msgClass: string;
          if (isSelf) {
            msgClass = 'bg-stone-600 text-white';
          } else if (item.data.type === 'gift') {
            msgClass = 'bg-amber-900/40 text-amber-400';
          } else {
            msgClass = 'bg-stone-800/80 text-stone-200';
          }
          return (
            <div
              key={item.id}
              className={`flex py-1 px-2 rounded-md transition ${isSelf ? 'justify-end' : 'justify-start'}`}
            >
              <div className={`flex items-start gap-1.5 max-w-[90%] ${isSelf ? 'flex-row-reverse' : 'flex-row'}`}>
                {item.data.uname && (
                  <span className="text-[11px] font-medium text-stone-500 mt-0.5 shrink-0">
                    {item.data.uname}
                  </span>
                )}
                <span className={`text-[12px] px-2 py-1 rounded-md ${msgClass}`}>
                  {parseMessage(item.data.msg || '', emoteMap)}
                </span>
              </div>
            </div>
          );
        })}
      </div>

      {/* Input bar */}
      <div className="px-3 py-2 shrink-0 border-t border-stone-800 bg-stone-950/90">
        <div className="flex gap-1.5">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault();
                handleSend();
              }
            }}
            placeholder="发送弹幕..."
            className="flex-1 h-7 px-2 rounded-md bg-stone-900 border border-stone-800 text-[12px] text-stone-200 placeholder:text-stone-600 focus:outline-none focus:ring-1 focus:ring-stone-600 transition"
          />
          <button
            onClick={clearDanmaku}
            className="w-7 h-7 rounded-md flex items-center justify-center text-stone-500 hover:text-stone-300 hover:bg-stone-800 transition"
            title="清空"
          >
            <Trash2 size={12} />
          </button>
          <button
            onClick={handleSend}
            className="w-7 h-7 rounded-md flex items-center justify-center bg-[#D4652A] text-white hover:opacity-90 transition"
            title="发送"
          >
            <Send size={12} />
          </button>
        </div>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add src/components/DanmakuFloat.tsx
git commit -m "feat(float): add compact DanmakuFloat component"
```

---

### Task 3: Branch App.tsx on window label

**Files:**
- Modify: `src/App.tsx`

- [ ] **Step 1: Add window label branch inside AppProvider**

Modify `src/App.tsx`:

1. Add import near the top:
   ```ts
   import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
   import DanmakuFloat from '@/components/DanmakuFloat';
   ```

2. Keep `AppContent` as-is. Add a new `AppInner` component that branches:

   ```tsx
   function AppInner() {
     const label = getCurrentWebviewWindow().label;
     if (label === 'danmaku-float') {
       return <DanmakuFloat />;
     }
     return <AppContent />;
   }
   ```

3. Change `App()` to wrap `AppInner` instead of `AppContent`:

   ```tsx
   function App() {
     return (
       <AppProvider>
         <AppInner />
       </AppProvider>
     );
   }
   ```

Verify that the file still imports `AppContent` (used inside `AppInner`) and that `DanmakuFloat` is imported.

- [ ] **Step 2: Commit**

```bash
git add src/App.tsx
git commit -m "feat(float): branch App.tsx on window label for float view"
```

---

### Task 4: Add float-launch button to Sidebar

**Files:**
- Modify: `src/components/Sidebar.tsx`

- [ ] **Step 1: Add imports and button**

In `src/components/Sidebar.tsx`:

1. Add `PanelTop` (or `MessageSquare` fallback) to the `lucide-react` import and import `openDanmakuFloat`:
   ```ts
   import { RadioTower, MessageSquare, User, Settings, PanelTop } from 'lucide-react';
   import { logout, clearSession, openDanmakuFloat } from '@/hooks/useTauri';
   ```

2. Inside the `navItems.map` render for the `danmaku` item, add a small icon button to the right side of the row. The button should only appear when the item is active.

   Replace the existing `navItems.map` return block (lines 116-128) with:

   ```tsx
   <button
     key={item.id}
     onClick={() => onTabChange(item.id)}
     className={`w-full flex items-center gap-3 px-3 py-2 rounded-lg text-[13px] font-medium transition ${
       isActive
         ? 'bg-stone-200 dark:bg-[#363236] text-stone-900 dark:text-stone-100'
         : 'text-stone-500 dark:text-stone-400 hover:bg-stone-200 dark:hover:bg-[#363236] hover:text-stone-900 dark:hover:text-stone-100'
     }`}
   >
     <item.icon size={16} />
     <span className="flex-1 text-left">{item.label}</span>
     {item.id === 'danmaku' && isActive && (
       <button
         onClick={(e) => {
           e.stopPropagation();
           openDanmakuFloat().catch(() => {});
         }}
         className="w-6 h-6 rounded flex items-center justify-center text-stone-500 dark:text-stone-400 hover:text-stone-900 dark:hover:text-stone-100 hover:bg-stone-300 dark:hover:bg-[#4a454d] transition"
         title="浮窗"
       >
         <PanelTop size={13} />
       </button>
     )}
   </button>
   ```

   This places the float-launch icon on the right side of the active "弹幕监控" nav row. `e.stopPropagation()` prevents triggering the parent tab change.

- [ ] **Step 2: Commit**

```bash
git add src/components/Sidebar.tsx
git commit -m "feat(float): add float window launch button to sidebar"
```

---

### Task 5: Add frontend Tauri hooks

**Files:**
- Modify: `src/hooks/useTauri.ts`

- [ ] **Step 1: Add open/close float window wrappers**

Add to `src/hooks/useTauri.ts` after the existing window functions:

```ts
export async function openDanmakuFloat(): Promise<void> {
  return await invoke('open_danmaku_float');
}

export async function closeDanmakuFloat(): Promise<void> {
  return await invoke('close_danmaku_float');
}
```

- [ ] **Step 2: Commit**

```bash
git add src/hooks/useTauri.ts
git commit -m "feat(float): add open/close float window tauri hooks"
```

---

### Task 6: Add FloatWindowState to Rust config model

**Files:**
- Modify: `src-tauri/src/models/config.rs`

- [ ] **Step 1: Add the struct and field**

Modify `src-tauri/src/models/config.rs` to add:

```rust
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct FloatWindowState {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
```

And add the field to `AppConfig`:

```rust
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AppConfig {
    pub current_uid: Option<u64>,
    pub users: HashMap<String, UserConfig>,
    #[serde(default = "default_min_to_tray")]
    pub min_to_tray: bool,
    #[serde(default)]
    pub float_window: Option<FloatWindowState>,
}
```

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/models/config.rs
git commit -m "feat(float): add FloatWindowState to AppConfig"
```

---

### Task 7: Implement Rust window commands

**Files:**
- Modify: `src-tauri/src/commands/window.rs`

- [ ] **Step 1: Add open_danmaku_float command**

Add to `src-tauri/src/commands/window.rs`:

```rust
use crate::state::AppState;

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
    let (width, height, x, y) = config
        .data()
        .float_window
        .as_ref()
        .map(|f| (f.width, f.height, f.x, f.y))
        .unwrap_or((320.0, 450.0, 0.0, 0.0));
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
    .inner_size(width, height);

    // Only set position if we have a saved state; otherwise center
    if width > 0.0 && height > 0.0 && x != 0.0 && y != 0.0 {
        builder = builder.position(x, y);
    } else {
        builder = builder.center();
    }

    builder.build().map_err(|e| e.to_string())?;
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

    // Close the window
    let _ = win.close();
    Ok(())
}
```

- [ ] **Step 2: Verify imports and compilation**

Ensure `src-tauri/src/commands/window.rs` has access to `AppState` and `crate::models::config::FloatWindowState`. The `use crate::state::AppState;` import should be at the top of the file.

Check that `ConfigStore` has a `data_mut()` method. If it doesn't, you may need to use whatever mutable accessor the existing code uses (e.g., `config.config.float_window = ...` or similar). Inspect `src-tauri/src/services/config_store.rs` to find the correct API.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands/window.rs
git commit -m "feat(float): add open/close danmaku float window commands"
```

---

### Task 8: Register new commands in main.rs

**Files:**
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Add commands to invoke_handler**

In `src-tauri/src/main.rs`, add these two lines to the `.invoke_handler(tauri::generate_handler![...])` block:

```rust
bilibili_streamer_lib::commands::window::open_danmaku_float,
bilibili_streamer_lib::commands::window::close_danmaku_float,
```

Place them near the existing window commands (`window_min`, `window_max`, etc.).

- [ ] **Step 2: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat(float): register danmaku float window commands"
```

---

### Task 9: Build and manual verification

**Files:**
- None (verification only)

- [ ] **Step 1: Run dev build**

```bash
npm run tauri-dev
```

Wait for the app to launch. If it fails to compile, read the Rust and TypeScript error messages, fix the issues, and rerun.

- [ ] **Step 2: Manual verification checklist**

With the app running:

1. **Sidebar button**: Navigate to "弹幕监控" tab. Verify a small icon button appears on the right side of the nav row. Hover shows "浮窗".
2. **Open float**: Click the button. A compact 320×450 window appears, titled "Monitor", with no native title bar.
3. **Always on top**: Click into another app (e.g., a browser). Verify the float window stays visible above it.
4. **Draggable**: Drag the window by its custom title bar.
5. **Danmaku sync**: Start a live stream (or simulate danmaku). Verify messages appear in both the main window's `DanmakuPanel` and the float window.
6. **Send from float**: Type a message in the float window's input bar and send. Verify it appears in both windows.
7. **Close float**: Click the × button. The float window closes.
8. **Reopen remembers position**: Move the float window, close it, reopen it. Verify it reopens at the last position.
9. **Main window to tray**: Close the main window (should minimize to tray if configured). Verify the float window remains visible.
10. **Quit app**: Quit from the tray menu. Both windows close gracefully.

- [ ] **Step 3: Final commit if any fixes were needed**

If any bugs were found and fixed during verification:

```bash
git add -A
git commit -m "fix(float): address verification issues"
```

---

## Self-Review Checklist

### 1. Spec coverage

| Spec requirement | Task |
|---|---|
| Shared bundle, second WebView window | Task 3 (App.tsx branch), Task 7 (Rust window builder) |
| macOS always-on-top | Task 7 (`.always_on_top(true)`) |
| Compact float UI with drag bar + close | Task 2 (DanmakuFloat.tsx) |
| Reuse danmaku bubble rendering | Task 1 (extract utils), Task 2 (import in Float) |
| Compact input bar | Task 2 |
| Slight transparency | Task 2 (`bg-stone-950/95`) |
| Sidebar launch button | Task 4 |
| Persist position/size | Task 6 (model), Task 7 (save on close) |
| Single instance only | Task 7 (check `get_webview_window` before creating) |
| Independent lifecycle from main window | Task 7 (no coupling to main window lifecycle) |

No gaps found.

### 2. Placeholder scan

- No "TBD", "TODO", "implement later", "fill in details" found.
- No vague "add error handling" steps — error handling is explicit in Task 7.
- No "similar to Task N" references.
- All code blocks contain complete, runnable code.

### 3. Type consistency

- `FloatWindowState` uses `f64` for all fields consistently (Task 6, Task 7).
- `open_danmaku_float` and `close_danmaku_float` signatures match the `invoke` calls in Task 5.
- `getCurrentWebviewWindow().label` is a synchronous `string` in Tauri API 2.11.0, used correctly in Task 3.

