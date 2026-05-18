use crate::models::config::UserConfig;
use crate::models::user::{LoginResult, QrCodeData};
use crate::services::bili_api::BiliApi;
use anyhow::Result;

pub struct AuthService;

impl AuthService {
    pub async fn get_login_qrcode(api: &BiliApi) -> Result<QrCodeData> {
        api.get_passport_qrcode().await
    }

    pub async fn poll_login_status(api: &BiliApi, key: &str) -> Result<LoginResult> {
        let (code, _message, cookies) = api.poll_passport_qrcode(key).await?;
        if code == 0 {
            let csrf = cookies
                .get("bili_jct")
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("登录成功但缺少 bili_jct cookie"))?;
            let uid = cookies
                .get("DedeUserID")
                .and_then(|s| s.parse::<u64>().ok())
                .ok_or_else(|| anyhow::anyhow!("登录成功但缺少 DedeUserID cookie"))?;
            let cookie_str = cookies
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("; ");
            let user = UserConfig {
                uid,
                uname: String::new(),
                face: String::new(),
                cookie: cookie_str,
                room_id: String::new(),
                csrf,
                last_title: String::new(),
                last_area_id: 0,
                last_area_name: vec![],
                level: 0,
                follower: 0,
                following: 0,
                dynamic_count: 0,
            };
            Ok(LoginResult {
                code,
                uid: Some(uid),
                user: Some(user),
            })
        } else {
            Ok(LoginResult {
                code,
                uid: None,
                user: None,
            })
        }
    }
}
