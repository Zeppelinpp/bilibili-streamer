use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum DanmakuMessage {
    #[serde(rename = "danmaku")]
    Danmaku {
        uid: u64,
        uname: String,
        face: String,
        msg: String,
        is_self: bool,
    },
    #[serde(rename = "interact")]
    Interact {
        uid: u64,
        uname: String,
        msg: String,
        is_self: bool,
    },
    #[serde(rename = "gift")]
    Gift {
        uid: u64,
        uname: String,
        face: String,
        gift_name: String,
        num: u32,
        action: String,
        is_self: bool,
    },
    #[serde(rename = "system")]
    System { msg: String },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SendDanmakuResult {
    pub code: i32,
    pub msg: String,
}

// Protobuf definition for INTERACT_WORD_V2
#[derive(Clone, PartialEq, prost::Message)]
pub struct InteractWordV2 {
    #[prost(uint64, tag = "1")]
    pub uid: u64,
    #[prost(string, tag = "2")]
    pub uname: String,
    #[prost(enumeration = "InteractWordV2MsgType", tag = "5")]
    pub msg_type: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, prost::Enumeration)]
#[repr(i32)]
pub enum InteractWordV2MsgType {
    Unknown = 0,
    EnterRoom = 1,
    Follow = 2,
    ShareRoom = 3,
}
