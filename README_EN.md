[简体中文](README.md) | English

# Bilibili Streamer

A third-party Bilibili live streaming desktop app rebuilt with [Tauri](https://tauri.app/). Supports QR-code login, stream key retrieval, danmaku monitoring and sending.

![Main Interface](screenshot-stream.png)

## Features

1. Log in to Bilibili via QR code, with support for switching between multiple accounts;
2. Retrieve third-party stream keys (RTMP / SRT) for use in OBS and other streaming software;
3. Set stream title and category when going live;
4. Monitor danmaku (including messages, user entries, and gift events) and send danmaku;
5. System tray integration with close-to-tray behavior;
6. Dark / light theme following the system.

## Tech Stack

- **Desktop Framework**: Tauri 2.x (Rust)
- **Frontend**: React 18 + TypeScript + Vite + Tailwind CSS
- **Backend**: Rust (Tokio async runtime)
- **Target Platforms**: macOS / Windows / Linux

## Download & Install

### macOS

1. Go to the [Releases](https://github.com/Zeppelinpp/bilibili-streamer/releases/latest) page and download the latest `.dmg`;
2. Open the `.dmg` and drag `Bilibili-Streamer.app` into the **Applications** folder;
3. If a warning says the app cannot be opened on first launch, go to **System Settings → Privacy & Security** and click **Open Anyway**.

## Usage

1. Log in to your Bilibili account via QR code;
2. Fill in the title and select a category (first-time users need to click `Sync`);
3. Click `Start Live` to begin streaming;
4. Copy the stream URL and stream key from the **Stream Key** section into your third-party streaming tool;
5. View and send danmaku in the **Danmaku** panel;
6. Click `Stop Live` or close the app to stop streaming. **Stopping the stream in OBS will NOT stop the live broadcast.**

## Build from Source

### Requirements

- **Rust**: 1.77.2+ (recommended to install via [rustup](https://rustup.rs/))
- **Node.js**: 18+

### Development Mode

```bash
# Install frontend dependencies
npm install

# Start Tauri dev (launches Vite dev server + Rust compile)
npm run tauri-dev
```

### Production Build

```bash
# Build frontend + Rust and package into a platform-native installer
npm run tauri-build
```

Output artifacts are located in `src-tauri/target/release/bundle/`:

- **macOS**: `.dmg` / `.app`
- **Windows**: `.msi` / `.exe`
- **Linux**: `.AppImage`

## Acknowledgements

This project is rebuilt based on [ChaceQC/bilibili_live_stream_code](https://github.com/ChaceQC/bilibili_live_stream_code). Thanks to the original author for the foundational implementation and open-source contribution.

## License

[Apache License 2.0](LICENSE.txt)
