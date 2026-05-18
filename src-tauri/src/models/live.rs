use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StreamCodeData {
    pub rtmp1: StreamProtocol,
    pub rtmp2: StreamProtocol,
    pub srt: StreamProtocol,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct StreamProtocol {
    pub addr: String,
    pub code: String,
}

pub type PartitionMap = HashMap<String, HashMap<String, u64>>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StartLiveResponse {
    pub code: i32,
    pub data: Option<StreamCodeData>,
    pub qr: Option<String>,
    pub msg: Option<String>,
}
