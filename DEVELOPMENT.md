# Bilibili-Streamer 开发文档

## 技术栈

- **桌面框架**: Tauri 2.x (Rust + WebView)
- **前端**: React 18 + TypeScript + Vite + Tailwind CSS
- **后端**: Rust (Tokio async runtime)
- **目标平台**: macOS / Windows / Linux

---

## 启动方式

### 环境要求

- **Rust**: 1.70+ (建议通过 [rustup](https://rustup.rs/) 安装)
- **Node.js**: 18+
- **系统依赖**:
  - macOS: Xcode Command Line Tools
  - Linux: `libwebkit2gtk-4.1-dev`, `libssl-dev`, `pkg-config`
  - Windows: 无需额外依赖

### 开发模式

```bash
# 1. 安装前端依赖
npm install

# 2. 启动 Tauri dev（同时启动 Vite dev server + Rust 编译）
npm run tauri-dev
# 或
npx tauri dev
```

Dev 模式下：
- Vite 在 `http://localhost:1420` 提供 HMR 前端服务
- Tauri 启动 WebView 窗口加载上述地址
- Rust 后端代码变更会自动重新编译并重启

### 生产构建

```bash
# 构建前端 + Rust 并打包为平台原生安装包
npm run tauri-build
```

输出产物位于 `src-tauri/target/release/bundle/`：
- **macOS**: `.dmg` / `.app`
- **Windows**: `.msi` / `.exe`
- **Linux**: `.AppImage` / `.deb`

---

## 项目架构

```
bilibili_live_stream_code/
├── src/                          # 前端 (React + TypeScript)
│   ├── components/               # UI 组件
│   │   ├── Sidebar.tsx           # 左侧导航栏
│   │   ├── StreamPanel.tsx       # 推流码面板
│   │   ├── DanmakuPanel.tsx      # 弹幕面板
│   │   ├── AccountPanel.tsx      # 账号管理面板
│   │   ├── SettingsPanel.tsx     # 设置面板
│   │   └── ConsolePanel.tsx      # 底部日志控制台
│   ├── context/
│   │   └── AppContext.tsx        # React Context (User/Live/Danmaku/UI)
│   ├── hooks/
│   │   └── useTauri.ts           # 封装所有 tauri::invoke 调用
│   ├── types/
│   │   └── api.ts                # 前后端共享的 TypeScript 类型
│   ├── App.tsx                   # 根组件
│   └── main.tsx                  # React 挂载入口
│
├── src-tauri/
│   ├── src/
│   │   ├── main.rs               # Tauri 应用入口 (setup, tray, window event)
│   │   ├── lib.rs                # Library 入口 (供 main.rs 引用)
│   │   ├── state.rs              # AppState / SessionState 全局状态定义
│   │   ├── commands/             # Tauri Command (前端可调用的 Rust 函数)
│   │   │   ├── auth.rs           # 扫码登录相关
│   │   │   ├── user.rs           # 用户/账号管理
│   │   │   ├── live.rs           # 开播/停播/推流码
│   │   │   ├── danmaku.rs        # 弹幕监控启停/发送弹幕
│   │   │   ├── config.rs         # 应用配置读写
│   │   │   └── window.rs         # 窗口控制 (最小化/最大化/拖拽)
│   │   ├── services/             # 业务逻辑层
│   │   │   ├── bili_api.rs       # B站 HTTP API 客户端 (含 App Sign)
│   │   │   ├── danmaku_ws.rs     # B站弹幕 WebSocket 连接/解析
│   │   │   ├── live_service.rs   # 开播业务逻辑封装
│   │   │   ├── user_service.rs   # 用户会话管理
│   │   │   ├── auth_service.rs   # 扫码登录轮询
│   │   │   └── config_store.rs   # 本地 TOML 配置持久化
│   │   ├── models/               # 数据模型 (DTO)
│   │   └── utils/                # 工具函数 (MD5, 掩码等)
│   └── Cargo.toml
│
├── package.json                  # 前端依赖与脚本
├── vite.config.ts                # Vite 配置
├── tailwind.config.ts            # Tailwind 配置
└── tsconfig.json                 # TypeScript 配置
```

---

## 前后端通信

### 1. 前端调用后端 (invoke)

前端通过 `useTauri.ts` 中封装的函数，使用 `invoke('command_name', args)` 调用 Rust Command。

```ts
// 示例：获取推流码
const data = await invoke('start_live', { pName, sName });
```

对应 Rust 端：

```rust
#[tauri::command]
pub async fn start_live(...) -> Result<StreamCodeData, String> { ... }
```

### 2. 后端推送前端 (emit / listen)

后端通过 `app_handle.emit(event, payload)` 主动推送事件：

| Event | 方向 | 说明 |
|-------|------|------|
| `danmu-message` | Rust -> Frontend | 新弹幕/进场/礼物消息 |
| `danmu-disconnected` | Rust -> Frontend | 弹幕 WebSocket 断开 |
| `tray-start-live` | Tray -> Frontend | 托盘菜单点击“开始直播” |
| `tray-stop-live` | Tray -> Frontend | 托盘菜单点击“停止直播” |

前端通过 `listen(event, handler)` 订阅：

```ts
import { listen } from '@tauri-apps/api/event';

listen('danmu-message', (event) => {
  addDanmaku(event.payload);
});
```

---

## 核心模块说明

### BiliApi (`services/bili_api.rs`)

- 封装所有 B站 HTTP 请求（基于 `reqwest`）
- 实现 **App Sign** 签名算法（MD5 + 固定盐值）
- 管理 Cookie 与会话状态
- 提供接口：登录/开播/停播/获取推流码/发送弹幕/获取分区列表

### DanmakuService (`services/danmaku_ws.rs`)

- 通过 `tokio_tungstenite` 连接 B站弹幕 WebSocket 服务器
- 协议：自定义二进制包（Header 16 byte + Body），支持 zlib / brotli 解压
- 独立 Tokio Task 运行心跳（30s）与消息接收循环
- 对外暴露 `connect(room_id)` / `disconnect()` 命令通道

### AppState (`state.rs`)

全局共享状态，通过 Tauri `manage()` 注入：

```rust
pub struct AppState {
    pub config: tokio::sync::Mutex<ConfigStore>,   // 本地配置
    pub session: tokio::sync::Mutex<SessionState>, // 当前用户/房间/CSRF
    pub api: Arc<tokio::sync::Mutex<BiliApi>>,     // API 客户端
    pub danmaku: tokio::sync::Mutex<Option<DanmakuService>>,
}
```

**锁顺序规则**（防止死锁）：`api -> config -> session -> danmaku`

### 前端状态管理 (`context/AppContext.tsx`)

使用 React Context 拆分为 4 个独立上下文，避免无关渲染：

- `UserContext` — 当前登录用户信息
- `LiveContext` — 开播状态、推流码数据
- `DanmakuContext` — 弹幕列表（保留最近 500 条）
- `UIContext` — 主题、控制台日志（保留最近 200 条）、面板开关

---

## 注意事项

1. **macOS 主线程限制**：Tauri 的 `.setup()` 和 `on_window_event` 回调运行在 macOS 主线程，没有 Tokio Runtime 上下文。不要在这些回调里使用 `tokio::task::block_in_place` 或 `block_on`，应改用 `tauri::async_runtime::spawn` 或将需要的数据提前以同步方式缓存。

2. **WebSocket 运行时**：`DanmakuService` 内部的 WebSocket Task 使用 `tokio::spawn`，但 Service 本身的创建/外层循环使用 `tauri::async_runtime::spawn`，以避免 macOS release 构建下的 Runtime panic。

3. **窗口关闭行为**：在 `on_window_event(CloseRequested)` 中读取 `min_to_tray` 配置。若开启，则阻止关闭并隐藏窗口到托盘；托盘左键点击可恢复窗口。
