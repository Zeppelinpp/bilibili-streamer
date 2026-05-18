use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserInfo {
    pub mid: u64,
    pub uname: String,
    pub face: String,
    pub level_info: LevelInfo,
    pub money: f64,
    pub wallet: Wallet,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LevelInfo {
    #[serde(rename = "current_level")]
    pub current_level: u32,
    #[serde(rename = "current_exp")]
    pub current_exp: u64,
    #[serde(rename = "next_exp")]
    pub next_exp: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Wallet {
    #[serde(rename = "bcoin_balance")]
    pub bcoin_balance: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserStat {
    pub following: u32,
    pub follower: u32,
    pub dynamic_count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QrCodeData {
    pub url: String,
    pub qrcode_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoginResult {
    pub code: i32,
    pub uid: Option<u64>,
    pub user: Option<crate::models::config::UserConfig>,
}
