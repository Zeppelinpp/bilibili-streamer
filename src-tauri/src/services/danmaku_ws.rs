use crate::models::danmaku::DanmakuMessage;
use crate::services::bili_api::BiliApi;
use futures::{SinkExt, StreamExt};
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
    SendDanmaku { msg: String },
}

pub struct DanmakuService {
    tx: mpsc::Sender<DanmakuCommand>,
    running: Arc<Mutex<bool>>,
}

impl DanmakuService {
    pub fn new(api: Arc<tokio::sync::Mutex<BiliApi>>, app_handle: AppHandle) -> Self {
        let (tx, mut rx) = mpsc::channel::<DanmakuCommand>(32);
        let running = Arc::new(Mutex::new(false));
        let running_clone = running.clone();

        tokio::spawn(async move {
            let mut ws_task: Option<tokio::task::JoinHandle<()>> = None;
            let mut room_id: Option<u64> = None;

            while let Some(cmd) = rx.recv().await {
                match cmd {
                    DanmakuCommand::Connect { room_id: rid } => {
                        if let Some(handle) = ws_task.take() {
                            handle.abort();
                        }
                        room_id = Some(rid);
                        *running_clone.lock().await = true;
                        let api_clone = api.clone();
                        let running_inner = running_clone.clone();
                        let app_handle_inner = app_handle.clone();
                        ws_task = Some(tokio::spawn(async move {
                            if let Err(e) = connect_and_run(api_clone, rid, running_inner, app_handle_inner).await {
                                tracing::error!("Danmaku error: {}", e);
                            }
                        }));
                    }
                    DanmakuCommand::Disconnect => {
                        *running_clone.lock().await = false;
                        if let Some(handle) = ws_task.take() {
                            handle.abort();
                        }
                        room_id = None;
                    }
                    DanmakuCommand::SendDanmaku { msg } => {
                        let api_guard = api.lock().await;
                        if let Some(rid) = room_id {
                            if let Some(csrf) = api_guard.get_csrf() {
                                let _ = api_guard.send_danmaku(rid, &msg, &csrf).await;
                            }
                        }
                    }
                }
            }
        });

        Self { tx, running }
    }

    pub async fn connect(&self, room_id: u64) {
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
) -> anyhow::Result<()> {
    let api_guard = api.lock().await;
    let danmaku_info = api_guard.get_danmaku_info(room_id).await?;
    drop(api_guard);

    let token = danmaku_info["data"]["token"].as_str().ok_or_else(|| anyhow::anyhow!("no token"))?;
    let host_list = danmaku_info["data"]["host_list"].as_array().ok_or_else(|| anyhow::anyhow!("no host list"))?;
    let host = host_list[0]["host"].as_str().ok_or_else(|| anyhow::anyhow!("no host"))?;
    let wss_port = host_list[0]["wss_port"].as_u64().unwrap_or(443) as u16;

    let ws_url = format!("wss://{}:{}/sub", host, wss_port);
    let (ws_stream, _) = connect_async(&ws_url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Send auth packet
    let auth = serde_json::json!({
        "uid": 0,
        "roomid": room_id,
        "protover": 3,
        "platform": "web",
        "type": 2,
        "key": token,
    });
    let auth_body = auth.to_string();
    let auth_packet = build_packet(7, &auth_body);
    write.send(WsMessage::Binary(auth_packet)).await?;

    // Heartbeat task
    let mut heartbeat = interval(Duration::from_secs(30));
    let write = Arc::new(Mutex::new(write));
    let write_clone = write.clone();

    tokio::spawn(async move {
        loop {
            heartbeat.tick().await;
            let packet = build_packet(2, "");
            if let Err(e) = write_clone.lock().await.send(WsMessage::Binary(packet)).await {
                tracing::error!("Heartbeat failed: {}", e);
                break;
            }
        }
    });

    // Read loop
    while *running.lock().await {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(Ok(WsMessage::Binary(data))) => {
                        process_packet(&data, &app_handle);
                    }
                    Some(Ok(WsMessage::Close(_))) | None => {
                        tracing::info!("WebSocket closed");
                        break;
                    }
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
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

fn process_packet(data: &[u8], app_handle: &AppHandle) {
    if data.len() < 16 {
        return;
    }
    let packet_len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let header_len = u16::from_be_bytes([data[4], data[5]]) as usize;
    let proto_ver = u16::from_be_bytes([data[6], data[7]]);
    let op = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

    let body = &data[header_len..packet_len];

    match proto_ver {
        2 => {
            if let Ok(decompressed) = decompress_zlib(body) {
                process_packet(&decompressed, app_handle);
            }
        }
        3 => {
            if let Ok(decompressed) = decompress_brotli(body) {
                process_packet(&decompressed, app_handle);
            }
        }
        _ => {
            if op == 5 {
                if let Ok(s) = std::str::from_utf8(body) {
                    if let Ok(json) = serde_json::from_str::<Value>(s) {
                        handle_command(json, app_handle);
                    }
                }
            } else if op == 3 {
                if body.len() >= 4 {
                    let pop = u32::from_be_bytes([body[0], body[1], body[2], body[3]]);
                    tracing::debug!("Popularity: {}", pop);
                }
            }
        }
    }

    if data.len() > packet_len {
        process_packet(&data[packet_len..], app_handle);
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

fn handle_command(cmd: Value, app_handle: &AppHandle) {
    let cmd_str = cmd["cmd"].as_str().unwrap_or("");
    if cmd_str.starts_with("DANMU_MSG") {
        if let Some(info) = cmd["info"].as_array() {
            if info.len() > 2 {
                let uid = info[2][0].as_u64().unwrap_or(0);
                let uname = info[2][1].as_str().unwrap_or("").to_string();
                let msg = info[1].as_str().unwrap_or("").to_string();
                let face = extract_face(info);
                let _ = app_handle.emit("danmu-message", DanmakuMessage::Danmaku { uid, uname, face, msg });
            }
        }
    } else if cmd_str == "INTERACT_WORD" {
        if let Some(data) = cmd["data"].as_object() {
            let uname = data["uname"].as_str().unwrap_or("").to_string();
            let msg_type = data["msg_type"].as_i64().unwrap_or(0);
            let msg = match msg_type {
                1 => format!("{} 进入了直播间", uname),
                2 => format!("{} 关注了直播间", uname),
                3 => format!("{} 分享了直播间", uname),
                _ => return,
            };
            let _ = app_handle.emit("danmu-message", DanmakuMessage::Interact { uid: data["uid"].as_u64().unwrap_or(0), uname, msg });
        }
    } else if cmd_str.starts_with("SEND_GIFT") {
        if let Some(data) = cmd["data"].as_object() {
            let uname = data["uname"].as_str().unwrap_or("").to_string();
            let gift_name = data["giftName"].as_str().unwrap_or("").to_string();
            let num = data["num"].as_u64().unwrap_or(0) as u32;
            let action = data["action"].as_str().unwrap_or("赠送").to_string();
            let face = data["face"].as_str().unwrap_or("").to_string();
            let uid = data["uid"].as_u64().unwrap_or(0);
            let _ = app_handle.emit("danmu-message", DanmakuMessage::Gift { uid, uname, face, gift_name, num, action });
        }
    }
}

fn extract_face(info: &[Value]) -> String {
    if let Some(extra) = info.get(0).and_then(|v| v.as_array()) {
        if let Some(user_data) = extra.get(15).and_then(|v| v.get("user")).and_then(|v| v.get("base")) {
            return user_data["face"].as_str().unwrap_or("").to_string();
        }
    }
    String::new()
}
