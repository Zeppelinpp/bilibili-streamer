use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DanmakuServerInfo {
    pub token: String,
    pub host_list: Vec<DanmakuHost>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DanmakuHost {
    pub host: String,
    pub wss_port: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum DanmakuMessage {
    #[serde(rename = "danmaku")]
    Danmaku { uid: u64, uname: String, face: String, msg: String },
    #[serde(rename = "interact")]
    Interact { uid: u64, uname: String, msg: String },
    #[serde(rename = "gift")]
    Gift { uid: u64, uname: String, face: String, gift_name: String, num: u32, action: String },
    #[serde(rename = "system")]
    System { msg: String },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SendDanmakuResult {
    pub code: i32,
    pub msg: String,
}
