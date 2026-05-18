use serde::{Deserialize, Serialize};

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
