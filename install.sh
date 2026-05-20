#!/bin/bash

npm run tauri build && rm -rf /Applications/Bilibili-Streamer.app && cp -R src-tauri/target/release/bundle/macos/Bilibili-Streamer.app /Applications/
