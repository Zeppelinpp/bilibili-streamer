use crate::models::danmaku::{DanmakuMessage, InteractWordV2};
use crate::services::bili_api::BiliApi;
use base64::Engine;
use futures::{SinkExt, StreamExt};
use http::Request as HttpRequest;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, Mutex};
use tokio::time::interval;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as WsMessage};

#[derive(Debug, Clone)]
pub enum DanmakuCommand {
    Connect { room_id: u64 },
    Disconnect,
}

pub struct DanmakuService {
    tx: mpsc::Sender<DanmakuCommand>,
    running: Arc<Mutex<bool>>,
    self_uid: Arc<std::sync::Mutex<Option<u64>>>,
}

impl DanmakuService {
    pub fn new(api: Arc<tokio::sync::Mutex<BiliApi>>, app_handle: AppHandle) -> Self {
        let (tx, mut rx) = mpsc::channel::<DanmakuCommand>(32);
        let running = Arc::new(Mutex::new(false));
        let running_clone = running.clone();
        let self_uid = Arc::new(std::sync::Mutex::new(None));
        let self_uid_clone = self_uid.clone();

        tauri::async_runtime::spawn(async move {
            let mut ws_task: Option<tokio::task::JoinHandle<()>> = None;

            while let Some(cmd) = rx.recv().await {
                match cmd {
                    DanmakuCommand::Connect { room_id } => {
                        if let Some(handle) = ws_task.take() {
                            handle.abort();
                        }
                        *running_clone.lock().await = true;
                        let api_clone = api.clone();
                        let running_inner = running_clone.clone();
                        let app_handle_inner = app_handle.clone();
                        let self_uid_inner = self_uid_clone.clone();
                        ws_task = Some(tokio::spawn(async move {
                            if let Err(e) = connect_and_run(
                                api_clone,
                                room_id,
                                running_inner,
                                app_handle_inner,
                                self_uid_inner,
                            )
                            .await
                            {
                                tracing::error!("Danmaku error: {}", e);
                            }
                        }));
                    }
                    DanmakuCommand::Disconnect => {
                        *running_clone.lock().await = false;
                        if let Some(handle) = ws_task.take() {
                            handle.abort();
                        }
                    }
                }
            }
        });

        Self {
            tx,
            running,
            self_uid,
        }
    }

    pub async fn connect(&self, room_id: u64, uid: Option<u64>) {
        if let Ok(mut guard) = self.self_uid.lock() {
            *guard = uid;
        }
        let _ = self.tx.send(DanmakuCommand::Connect { room_id }).await;
    }

    pub async fn disconnect(&self) {
        let _ = self.tx.send(DanmakuCommand::Disconnect).await;
    }

    pub async fn is_running(&self) -> bool {
        *self.running.lock().await
    }
}

async fn connect_and_run(
    api: Arc<tokio::sync::Mutex<BiliApi>>,
    room_id: u64,
    running: Arc<Mutex<bool>>,
    app_handle: AppHandle,
    self_uid: Arc<std::sync::Mutex<Option<u64>>>,
) -> anyhow::Result<()> {
    let mut api_guard = api.lock().await;
    let danmaku_info = api_guard.get_danmaku_info(room_id).await?;
    drop(api_guard);

    let token = danmaku_info["data"]["token"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("no token"))?;
    let host_list = danmaku_info["data"]["host_list"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("no host list"))?;
    let host = host_list[0]["host"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("no host"))?;
    let wss_port = host_list[0]["wss_port"].as_u64().unwrap_or(443) as u16;

    let ws_url = format!("wss://{}:{}/sub", host, wss_port);

    // Generate Sec-WebSocket-Key (base64 of 16 random bytes)
    let mut nonce = [0u8; 16];
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    nonce.copy_from_slice(&ts.to_le_bytes()[..16]);
    let sec_key = base64::engine::general_purpose::STANDARD.encode(&nonce);

    // Build full WebSocket handshake request with custom headers
    let req = HttpRequest::builder()
        .uri(&ws_url)
        .header("Host", format!("{}:{}", host, wss_port))
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", sec_key)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("Referer", "https://live.bilibili.com")
        .header("Origin", "https://live.bilibili.com")
        .body(())?;
    let (ws_stream, _) = connect_async(req).await?;
    let (mut write, mut read) = ws_stream.split();

    let uid = self_uid.lock().ok().and_then(|g| *g).unwrap_or(0);

    // Send auth packet
    let auth = serde_json::json!({
        "uid": uid,
        "roomid": room_id,
        "protover": 3,
        "platform": "web",
        "type": 2,
        "key": token,
    });
    let auth_body = auth.to_string();
    let auth_packet = build_packet(7, &auth_body);
    write.send(WsMessage::Binary(auth_packet)).await?;

    let mut heartbeat = interval(Duration::from_secs(30));
    let write = Arc::new(Mutex::new(write));
    let write_clone = write.clone();

    let result: anyhow::Result<()> = loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                let packet = build_packet(2, "");
                if let Err(e) = write.lock().await.send(WsMessage::Binary(packet)).await {
                    tracing::error!("Heartbeat failed: {}", e);
                    break Ok(());
                }
            }
            msg = read.next() => {
                match msg {
                    Some(Ok(WsMessage::Binary(data))) => {
                        process_packet(&data, &app_handle, &self_uid);
                    }
                    Some(Ok(WsMessage::Close(_))) | None => {
                        tracing::info!("WebSocket closed");
                        break Ok(());
                    }
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error: {}", e);
                        break Err(anyhow::anyhow!(e));
                    }
                    _ => {}
                }
            }
        }
    };

    let _ = write_clone.lock().await.send(WsMessage::Close(None)).await;
    *running.lock().await = false;
    let _ = app_handle.emit("danmu-disconnected", ()).ok();
    result
}

fn build_packet(op: u32, body: &str) -> Vec<u8> {
    let body_bytes = body.as_bytes();
    let len = 16 + body_bytes.len() as u32;
    let mut packet = Vec::with_capacity(len as usize);
    packet.extend_from_slice(&len.to_be_bytes());
    packet.extend_from_slice(&16u16.to_be_bytes());
    packet.extend_from_slice(&1u16.to_be_bytes());
    packet.extend_from_slice(&op.to_be_bytes());
    packet.extend_from_slice(&1u32.to_be_bytes());
    packet.extend_from_slice(body_bytes);
    packet
}

fn process_packet(
    data: &[u8],
    app_handle: &AppHandle,
    self_uid: &Arc<std::sync::Mutex<Option<u64>>>,
) {
    process_packet_inner(data, app_handle, 0, self_uid);
}

const MAX_DECOMPRESS_DEPTH: u8 = 8;

fn process_packet_inner(
    data: &[u8],
    app_handle: &AppHandle,
    depth: u8,
    self_uid: &Arc<std::sync::Mutex<Option<u64>>>,
) {
    if depth > MAX_DECOMPRESS_DEPTH {
        tracing::warn!(
            "Danmaku packet decompression exceeded max depth {}",
            MAX_DECOMPRESS_DEPTH
        );
        return;
    }
    let mut offset = 0;
    while offset + 16 <= data.len() {
        let packet_len = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        let header_len = u16::from_be_bytes([data[offset + 4], data[offset + 5]]) as usize;

        if packet_len < header_len || offset + packet_len > data.len() {
            tracing::warn!("Invalid packet length: {} at offset {}", packet_len, offset);
            break;
        }

        let proto_ver = u16::from_be_bytes([data[offset + 6], data[offset + 7]]);
        let op = u32::from_be_bytes([
            data[offset + 8],
            data[offset + 9],
            data[offset + 10],
            data[offset + 11],
        ]);
        let body = &data[offset + header_len..offset + packet_len];

        match proto_ver {
            2 => {
                if let Ok(decompressed) = decompress_zlib(body) {
                    process_packet_inner(&decompressed, app_handle, depth + 1, self_uid);
                }
            }
            3 => {
                if let Ok(decompressed) = decompress_brotli(body) {
                    process_packet_inner(&decompressed, app_handle, depth + 1, self_uid);
                }
            }
            _ => {
                if op == 5 {
                    if let Ok(s) = std::str::from_utf8(body) {
                        if let Ok(json) = serde_json::from_str::<Value>(s) {
                            handle_command(json, app_handle, self_uid);
                        }
                    }
                } else if op == 3 {
                    if body.len() >= 4 {
                        let pop = u32::from_be_bytes([body[0], body[1], body[2], body[3]]);
                        tracing::debug!("Popularity: {}", pop);
                    }
                } else if op == 8 {
                    if let Ok(s) = std::str::from_utf8(body) {
                        if let Ok(json) = serde_json::from_str::<Value>(s) {
                            let code = json["code"].as_i64().unwrap_or(-1);
                            if code == 0 {
                                tracing::info!("Danmaku authentication successful");
                            } else {
                                tracing::error!("Danmaku authentication failed: {:?}", json);
                            }
                        }
                    }
                }
            }
        }

        offset += packet_len;
    }
}

fn decompress_zlib(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    use std::io::Read;
    let mut decoder = flate2::read::ZlibDecoder::new(data);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result)?;
    Ok(result)
}

fn decompress_brotli(data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut result = Vec::new();
    let mut reader = brotli::Decompressor::new(data, 4096);
    use std::io::Read;
    reader.read_to_end(&mut result)?;
    Ok(result)
}

fn handle_command(
    cmd: Value,
    app_handle: &AppHandle,
    self_uid: &Arc<std::sync::Mutex<Option<u64>>>,
) {
    let self_uid_val = self_uid.lock().ok().and_then(|g| *g);
    let is_self = |uid: u64| self_uid_val.map_or(false, |s| s == uid);

    let cmd_str = cmd["cmd"].as_str().unwrap_or("");
    tracing::info!("Danmaku command received: {}", cmd_str);
    if cmd_str.starts_with("DANMU_MSG") {
        match cmd.get("info").and_then(|v| v.as_array()) {
            Some(info) if info.len() > 2 => {
                let uid = info[2][0].as_u64().unwrap_or(0);
                let uname = info[2][1].as_str().unwrap_or("").to_string();
                let msg = info[1].as_str().unwrap_or("").to_string();
                let face = extract_face(info);
                let msg_payload = DanmakuMessage::Danmaku {
                    uid,
                    uname: uname.clone(),
                    face,
                    msg: msg.clone(),
                    is_self: is_self(uid),
                };
                if let Err(e) = app_handle.emit("danmu-message", &msg_payload) {
                    tracing::error!("Failed to emit danmu-message: {}", e);
                } else {
                    tracing::info!("Emitted danmu: {}: {}", uname, msg);
                }
            }
            Some(info) => {
                tracing::warn!("DANMU_MSG info too short: len={}", info.len());
            }
            None => {
                tracing::warn!("DANMU_MSG missing info field");
            }
        }
    } else if cmd_str == "INTERACT_WORD" {
        if let Some(data) = cmd["data"].as_object() {
            let uname = data["uname"].as_str().unwrap_or("").to_string();
            let msg_type = data["msg_type"].as_i64().unwrap_or(0);
            let uid = data["uid"].as_u64().unwrap_or(0);
            let msg = match msg_type {
                1 => format!("{} 进入了直播间", uname),
                2 => format!("{} 关注了直播间", uname),
                3 => format!("{} 分享了直播间", uname),
                _ => return,
            };
            if let Err(e) = app_handle.emit(
                "danmu-message",
                DanmakuMessage::Interact {
                    uid,
                    uname: uname.clone(),
                    msg: msg.clone(),
                    is_self: is_self(uid),
                },
            ) {
                tracing::error!("Failed to emit INTERACT_WORD: {}", e);
            } else {
                tracing::info!("Emitted INTERACT_WORD: {} {}", uname, msg_type);
            }
        }
    } else if cmd_str.starts_with("ENTRY_EFFECT") {
        if let Some(data) = cmd["data"].as_object() {
            if let Some(copy_writing) = data["copy_writing"].as_str() {
                let msg = copy_writing.replace("<%", "").replace("%>", "");
                let uid = data["uid"].as_u64().unwrap_or(0);
                if let Err(e) = app_handle.emit(
                    "danmu-message",
                    DanmakuMessage::Interact {
                        uid,
                        uname: String::new(),
                        msg: msg.clone(),
                        is_self: is_self(uid),
                    },
                ) {
                    tracing::error!("Failed to emit ENTRY_EFFECT: {}", e);
                } else {
                    tracing::info!("Emitted ENTRY_EFFECT: {}", msg);
                }
            }
        }
    } else if cmd_str.starts_with("INTERACT_WORD_V2") {
        if let Some(data) = cmd["data"].as_object() {
            if let Some(pb_b64) = data["pb"].as_str() {
                match base64::engine::general_purpose::STANDARD.decode(pb_b64) {
                    Ok(pb_bytes) => {
                        match prost::Message::decode(&*pb_bytes) as Result<InteractWordV2, _> {
                            Ok(v2) => {
                                let msg = match v2.msg_type {
                                    1 => format!("{} 进入了直播间", v2.uname),
                                    2 => format!("{} 关注了直播间", v2.uname),
                                    3 => format!("{} 分享了直播间", v2.uname),
                                    _ => return,
                                };
                                if let Err(e) = app_handle.emit(
                                    "danmu-message",
                                    DanmakuMessage::Interact {
                                        uid: v2.uid,
                                        uname: v2.uname.clone(),
                                        msg,
                                        is_self: is_self(v2.uid),
                                    },
                                ) {
                                    tracing::error!("Failed to emit INTERACT_WORD_V2: {}", e);
                                } else {
                                    tracing::info!("Emitted INTERACT_WORD_V2: {}", v2.uname);
                                }
                            }
                            Err(e) => {
                                tracing::warn!("INTERACT_WORD_V2 protobuf decode failed: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("INTERACT_WORD_V2 base64 decode failed: {}", e);
                    }
                }
            } else {
                tracing::warn!("INTERACT_WORD_V2 missing pb field");
            }
        }
    } else if cmd_str.starts_with("SEND_GIFT") {
        if let Some(data) = cmd["data"].as_object() {
            let uname = data["uname"].as_str().unwrap_or("").to_string();
            let gift_name = data["giftName"].as_str().unwrap_or("").to_string();
            let num = data["num"].as_u64().unwrap_or(0) as u32;
            let action = data["action"].as_str().unwrap_or("赠送").to_string();
            let face = data["face"].as_str().unwrap_or("").to_string();
            let uid = data["uid"].as_u64().unwrap_or(0);
            if let Err(e) = app_handle.emit(
                "danmu-message",
                DanmakuMessage::Gift {
                    uid,
                    uname: uname.clone(),
                    face,
                    gift_name: gift_name.clone(),
                    num,
                    action: action.clone(),
                    is_self: is_self(uid),
                },
            ) {
                tracing::error!("Failed to emit SEND_GIFT: {}", e);
            } else {
                tracing::info!("Emitted SEND_GIFT: {} {}", uname, gift_name);
            }
        }
    }
}

// Bilibili DANMU_MSG format: info[0][15]["user"]["base"]["face"]
// This depends on Bilibili's internal protobuf-to-JSON mapping and may break if the server changes field ordering.
fn extract_face(info: &[Value]) -> String {
    if let Some(extra) = info.get(0).and_then(|v| v.as_array()) {
        if let Some(user_data) = extra
            .get(15)
            .and_then(|v| v.get("user"))
            .and_then(|v| v.get("base"))
        {
            return user_data["face"].as_str().unwrap_or("").to_string();
        }
    }
    String::new()
}
