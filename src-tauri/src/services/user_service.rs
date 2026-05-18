use crate::models::config::UserConfig;
use crate::services::bili_api::BiliApi;
use crate::services::config_store::ConfigStore;
use anyhow::Result;
use serde_json::Value;

pub struct UserService;

impl UserService {
    pub fn init_current_user(
        config: &ConfigStore,
        session: &mut crate::state::SessionState,
        api: &mut BiliApi,
    ) {
        if let Some(uid) = config.data().current_uid {
            let uid_str = uid.to_string();
            if let Some(user) = config.data().users.get(&uid_str) {
                session.uid = Some(user.uid);
                session.room_id = Some(user.room_id.clone());
                session.csrf = Some(user.csrf.clone());
                session.current_area_id = if user.last_area_id > 0 {
                    Some(user.last_area_id)
                } else {
                    None
                };
                session.current_area_names = user.last_area_name.clone();
                let cookies = parse_cookie_str(&user.cookie);
                api.update_cookies(cookies);
            }
        }
    }

    pub async fn refresh_current_user(
        api: &mut BiliApi,
        config: &mut ConfigStore,
        session: &mut crate::state::SessionState,
    ) -> Result<UserConfig> {
        let uid = session.uid.ok_or_else(|| anyhow::anyhow!("未登录"))?;
        let nav = api.get_user_info().await?;
        if nav["code"].as_i64().unwrap_or(-1) != 0 {
            return Err(anyhow::anyhow!("获取用户信息失败"));
        }
        let stat = api.get_user_stat().await?;
        let stat_data = if stat["code"].as_i64().unwrap_or(-1) == 0 {
            stat["data"].clone()
        } else {
            Value::Null
        };

        let uid_str = uid.to_string();
        let mut room_id = session.room_id.clone().unwrap_or_default();
        if room_id.is_empty() {
            let room_res = api.get_room_id_by_uid(uid).await?;
            if room_res["code"].as_i64().unwrap_or(-1) == 0 {
                room_id = room_res["data"]["room_id"]
                    .as_u64()
                    .unwrap_or(0)
                    .to_string();
                session.room_id = Some(room_id.clone());
            }
        }
        let csrf = session.csrf.clone().unwrap_or_default();
        let cookie_str = api.cookie_str();

        let user = build_user_config(uid, &nav["data"], &stat_data, &cookie_str, &room_id, &csrf);
        config
            .data_mut()
            .users
            .insert(uid_str.clone(), user.clone());
        config.data_mut().current_uid = Some(uid);
        config.save()?;
        Ok(user)
    }

    pub fn get_account_list(config: &ConfigStore) -> Vec<UserConfig> {
        config.data().users.values().cloned().collect()
    }

    pub fn switch_account(
        config: &mut ConfigStore,
        session: &mut crate::state::SessionState,
        api: &mut BiliApi,
        uid: u64,
    ) -> Result<UserConfig> {
        let uid_str = uid.to_string();
        let user = config
            .data()
            .users
            .get(&uid_str)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("账户不存在"))?;
        config.data_mut().current_uid = Some(uid);
        config.save()?;
        session.uid = Some(user.uid);
        session.room_id = Some(user.room_id.clone());
        session.csrf = Some(user.csrf.clone());
        session.current_area_id = if user.last_area_id > 0 {
            Some(user.last_area_id)
        } else {
            None
        };
        let cookies = parse_cookie_str(&user.cookie);
        api.update_cookies(cookies);
        Ok(user)
    }

    pub fn logout(
        config: &mut ConfigStore,
        session: &mut crate::state::SessionState,
        api: &mut BiliApi,
        uid: u64,
    ) -> Result<()> {
        let uid_str = uid.to_string();
        config.data_mut().users.remove(&uid_str);
        if config.data().current_uid == Some(uid) {
            config.data_mut().current_uid = None;
            session.uid = None;
            session.room_id = None;
            session.csrf = None;
            api.update_cookies(std::collections::HashMap::new());
        }
        config.save()?;
        Ok(())
    }
}

fn parse_cookie_str(s: &str) -> std::collections::HashMap<String, String> {
    s.split(';')
        .filter_map(|part| {
            let mut kv = part.trim().splitn(2, '=');
            let k = kv.next()?;
            let v = kv.next()?;
            Some((k.to_string(), v.to_string()))
        })
        .collect()
}

fn build_user_config(
    uid: u64,
    nav: &Value,
    stat: &Value,
    cookie: &str,
    room_id: &str,
    csrf: &str,
) -> UserConfig {
    let face = nav["face"]
        .as_str()
        .filter(|s| !s.is_empty())
        .or_else(|| nav["face_nft"].as_str().filter(|s| !s.is_empty()))
        .unwrap_or("")
        .to_string();
    if face.is_empty() {
        tracing::warn!(
            "Bilibili API returned empty face for uid={}, nav keys: {:?}",
            uid,
            nav.as_object().map(|m| m.keys().collect::<Vec<_>>())
        );
    }
    UserConfig {
        uid,
        uname: nav["uname"].as_str().unwrap_or("").to_string(),
        face,
        cookie: cookie.to_string(),
        room_id: room_id.to_string(),
        csrf: csrf.to_string(),
        last_title: String::new(),
        last_area_id: 0,
        last_area_name: vec![],
        level: nav["level_info"]["current_level"].as_u64().unwrap_or(0) as u32,
        follower: stat["follower"].as_u64().unwrap_or(0) as u32,
        following: stat["following"].as_u64().unwrap_or(0) as u32,
        dynamic_count: stat["dynamic_count"].as_u64().unwrap_or(0) as u32,
    }
}
