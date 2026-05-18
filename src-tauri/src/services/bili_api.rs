use crate::models::user::QrCodeData;
use crate::utils::crypto::app_sign;
use crate::utils::wbi::{extract_wbi_keys, wbi_sign};
use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde_json::Value;
use std::collections::HashMap;

const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36";

pub struct BiliApi {
    client: reqwest::Client,
    cookies: HashMap<String, String>,
    wbi_keys: Option<(String, String)>,
}

impl BiliApi {
    pub fn new() -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static(UA));
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        Ok(Self {
            client,
            cookies: HashMap::new(),
            wbi_keys: None,
        })
    }

    pub fn update_cookies(&mut self, cookies: HashMap<String, String>) {
        self.cookies = cookies;
    }

    pub fn cookie_str(&self) -> String {
        self.cookies
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("; ")
    }

    pub fn get_csrf(&self) -> Option<String> {
        self.cookies.get("bili_jct").cloned()
    }

    fn headers(&self) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert("User-Agent", HeaderValue::from_static(UA));
        h.insert(
            "Referer",
            HeaderValue::from_static("https://live.bilibili.com"),
        );
        if !self.cookies.is_empty() {
            if let Ok(v) = HeaderValue::from_str(&self.cookie_str()) {
                h.insert("Cookie", v);
            }
        }
        h
    }

    pub async fn request(
        &self,
        method: &str,
        url: &str,
        params: Option<HashMap<String, String>>,
        data: Option<HashMap<String, String>>,
    ) -> Result<Value> {
        let mut req = if method == "GET" {
            let mut r = self.client.get(url);
            if let Some(p) = params {
                r = r.query(&p);
            }
            r
        } else {
            let mut r = self.client.post(url);
            if let Some(p) = params {
                r = r.query(&p);
            }
            if let Some(d) = data {
                r = r.form(&d);
            }
            r
        };
        req = req.headers(self.headers());
        let resp = req.send().await?.error_for_status()?;
        let json: Value = resp.json().await?;
        Ok(json)
    }

    // --- 扫码登录 ---
    pub async fn get_passport_qrcode(&self) -> Result<QrCodeData> {
        let res = self
            .request(
                "GET",
                "https://passport.bilibili.com/x/passport-login/web/qrcode/generate",
                None,
                None,
            )
            .await?;
        let data = res["data"].clone();
        let qr: QrCodeData = serde_json::from_value(data)?;
        Ok(qr)
    }

    pub async fn poll_passport_qrcode(
        &self,
        key: &str,
    ) -> Result<(i32, String, HashMap<String, String>)> {
        let url = "https://passport.bilibili.com/x/passport-login/web/qrcode/poll";
        let resp = self
            .client
            .get(url)
            .query(&[("qrcode_key", key)])
            .headers(self.headers())
            .send()
            .await?;

        let mut cookies = HashMap::new();
        for cookie_hdr in resp.headers().get_all("set-cookie") {
            if let Ok(s) = cookie_hdr.to_str() {
                if let Some((name_value, _)) = s.split_once(';') {
                    if let Some((name, value)) = name_value.split_once('=') {
                        cookies.insert(name.trim().to_string(), value.trim().to_string());
                    }
                }
            }
        }

        let json: Value = resp.json().await?;
        let code = json["data"]["code"].as_i64().unwrap_or(-1) as i32;
        let message = json["data"]["message"].as_str().unwrap_or("").to_string();
        Ok((code, message, cookies))
    }

    // --- 用户信息 ---
    pub async fn get_user_info(&mut self) -> Result<Value> {
        let res = self
            .request(
                "GET",
                "https://api.bilibili.com/x/web-interface/nav",
                None,
                None,
            )
            .await?;
        if let Some((img, sub)) = extract_wbi_keys(&res["data"]) {
            self.wbi_keys = Some((img, sub));
        }
        Ok(res)
    }

    pub async fn get_user_stat(&self) -> Result<Value> {
        self.request(
            "GET",
            "https://api.bilibili.com/x/web-interface/nav/stat",
            None,
            None,
        )
        .await
    }

    pub async fn get_room_id_by_uid(&self, uid: u64) -> Result<Value> {
        self.request(
            "GET",
            "https://api.live.bilibili.com/room/v2/Room/room_id_by_uid",
            Some(HashMap::from([("uid".to_string(), uid.to_string())])),
            None,
        )
        .await
    }

    // --- 直播控制 ---
    pub async fn get_area_list(&self) -> Result<Value> {
        self.request(
            "GET",
            "https://api.live.bilibili.com/room/v1/Area/getList",
            Some(HashMap::from([(
                "show_pinyin".to_string(),
                "1".to_string(),
            )])),
            None,
        )
        .await
    }

    pub async fn update_title(&self, room_id: u64, title: &str, csrf: &str) -> Result<Value> {
        let mut data = HashMap::new();
        data.insert("room_id".to_string(), room_id.to_string());
        data.insert("platform".to_string(), "pc_link".to_string());
        data.insert("title".to_string(), title.to_string());
        data.insert("csrf_token".to_string(), csrf.to_string());
        data.insert("csrf".to_string(), csrf.to_string());
        self.request(
            "POST",
            "https://api.live.bilibili.com/room/v1/Room/update",
            None,
            Some(data),
        )
        .await
    }

    pub async fn update_area(&self, room_id: u64, area_id: u64, csrf: &str) -> Result<Value> {
        let mut data = HashMap::new();
        data.insert("room_id".to_string(), room_id.to_string());
        data.insert("area_id".to_string(), area_id.to_string());
        data.insert("platform".to_string(), "pc_link".to_string());
        data.insert("csrf_token".to_string(), csrf.to_string());
        data.insert("csrf".to_string(), csrf.to_string());
        self.request(
            "POST",
            "https://api.live.bilibili.com/room/v1/Room/update",
            None,
            Some(data),
        )
        .await
    }

    pub async fn start_live(&self, room_id: u64, area_id: u64, csrf: &str) -> Result<Value> {
        // 1. 获取时间戳
        let now_res = self
            .request(
                "GET",
                "https://api.bilibili.com/x/report/click/now",
                None,
                None,
            )
            .await?;
        let ts = now_res["data"]["now"].as_i64().unwrap_or(0).to_string();

        // 2. 获取版本
        let mut v_params = HashMap::new();
        v_params.insert("system_version".to_string(), "2".to_string());
        v_params.insert("ts".to_string(), ts.clone());
        let v_signed = app_sign(&mut v_params);
        let v_res = self.request("GET", "https://api.live.bilibili.com/xlive/app-blink/v1/liveVersionInfo/getHomePageLiveVersion", Some(v_signed), None).await?;
        let build = v_res["data"]["build"].as_i64().unwrap_or(0).to_string();
        let version = v_res["data"]["curr_version"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // 3. 开播
        let mut data = HashMap::new();
        data.insert("room_id".to_string(), room_id.to_string());
        data.insert("platform".to_string(), "pc_link".to_string());
        data.insert("area_v2".to_string(), area_id.to_string());
        data.insert("backup_stream".to_string(), "0".to_string());
        data.insert("csrf_token".to_string(), csrf.to_string());
        data.insert("csrf".to_string(), csrf.to_string());
        data.insert("build".to_string(), build);
        data.insert("version".to_string(), version);
        data.insert("ts".to_string(), ts);
        let signed = app_sign(&mut data);
        self.request(
            "POST",
            "https://api.live.bilibili.com/room/v1/Room/startLive",
            Some(signed),
            None,
        )
        .await
    }

    pub async fn stop_live(&self, room_id: u64, csrf: &str) -> Result<Value> {
        let mut data = HashMap::new();
        data.insert("room_id".to_string(), room_id.to_string());
        data.insert("platform".to_string(), "pc_link".to_string());
        data.insert("csrf_token".to_string(), csrf.to_string());
        data.insert("csrf".to_string(), csrf.to_string());
        self.request(
            "POST",
            "https://api.live.bilibili.com/room/v1/Room/stopLive",
            None,
            Some(data),
        )
        .await
    }

    // --- 弹幕 ---
    pub async fn get_danmaku_info(&mut self, room_id: u64) -> Result<Value> {
        let mut params = HashMap::from([
            ("id".to_string(), room_id.to_string()),
            ("type".to_string(), "0".to_string()),
        ]);
        if let Some((ref img, ref sub)) = self.wbi_keys {
            wbi_sign(&mut params, img, sub);
        } else {
            // Fallback: fetch WBI keys from nav API
            if let Ok(nav) = self
                .request(
                    "GET",
                    "https://api.bilibili.com/x/web-interface/nav",
                    None,
                    None,
                )
                .await
            {
                if let Some((img, sub)) = extract_wbi_keys(&nav["data"]) {
                    wbi_sign(&mut params, &img, &sub);
                    self.wbi_keys = Some((img, sub));
                }
            }
        }
        self.request(
            "GET",
            "https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo",
            Some(params),
            None,
        )
        .await
    }

    pub async fn send_danmaku(&self, room_id: u64, msg: &str, csrf: &str) -> Result<Value> {
        let mut data = HashMap::new();
        data.insert("msg".to_string(), msg.to_string());
        data.insert("roomid".to_string(), room_id.to_string());
        data.insert("csrf_token".to_string(), csrf.to_string());
        data.insert("csrf".to_string(), csrf.to_string());
        data.insert(
            "rnd".to_string(),
            chrono::Utc::now().timestamp().to_string(),
        );
        data.insert("fontsize".to_string(), "25".to_string());
        data.insert("mode".to_string(), "1".to_string());
        data.insert("pool".to_string(), "0".to_string());
        data.insert("color".to_string(), "16777215".to_string());
        self.request(
            "POST",
            "https://api.live.bilibili.com/msg/send",
            None,
            Some(data),
        )
        .await
    }
}
