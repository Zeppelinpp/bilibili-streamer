use md5::{Digest, Md5};
use std::collections::HashMap;

const MIXIN_KEY_ENC_TAB: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29,
    28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25,
    54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
];

fn get_mixin_key(orig: &str) -> String {
    MIXIN_KEY_ENC_TAB
        .iter()
        .filter_map(|&i| orig.chars().nth(i))
        .take(32)
        .collect()
}

/// Sign params with WBI keys. Returns signed params with wts and w_rid added.
pub fn wbi_sign(params: &mut HashMap<String, String>, img_key: &str, sub_key: &str) {
    let mixin_key = get_mixin_key(&(img_key.to_string() + sub_key));
    let wts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string();
    params.insert("wts".to_string(), wts);

    // Sort by key
    let mut keys: Vec<_> = params.keys().cloned().collect();
    keys.sort();

    // Build query string, filtering "!'()*" from values
    let query: Vec<String> = keys
        .iter()
        .map(|k| {
            let filtered: String = params[k]
                .chars()
                .filter(|c| !matches!(c, '\'' | '!' | '(' | ')' | '*'))
                .collect();
            format!(
                "{}={}",
                urlencoding::encode(k),
                urlencoding::encode(&filtered)
            )
        })
        .collect();
    let query_str = query.join("&");

    let sign_str = format!("{}{}", query_str, mixin_key);
    let hash = Md5::digest(sign_str.as_bytes());
    let w_rid = hex::encode(hash);
    params.insert("w_rid".to_string(), w_rid);
}

/// Extract WBI keys from nav response data.
pub fn extract_wbi_keys(data: &serde_json::Value) -> Option<(String, String)> {
    let img_url = data["wbi_img"]["img_url"].as_str()?;
    let sub_url = data["wbi_img"]["sub_url"].as_str()?;
    let img_key = img_url.rsplit('/').next()?.split('.').next()?;
    let sub_key = sub_url.rsplit('/').next()?.split('.').next()?;
    Some((img_key.to_string(), sub_key.to_string()))
}
