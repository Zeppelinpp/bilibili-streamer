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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Area {
    pub id: u64,
    pub name: String,
    pub list: Vec<SubArea>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubArea {
    pub id: u64,
    pub name: String,
}

pub type PartitionMap = HashMap<String, HashMap<String, u64>>;
