use md5::{Digest, Md5};
use std::collections::HashMap;

const APP_KEY: &str = "aae92bc66f3edfab";
const APP_SEC: &str = "af125a0d5279fd576c1b4418a3e8276d";

pub fn app_sign(params: &mut HashMap<String, String>) -> HashMap<String, String> {
    params.insert("appkey".to_string(), APP_KEY.to_string());
    let mut keys: Vec<_> = params.keys().cloned().collect();
    keys.sort();
    let query: Vec<String> = keys
        .iter()
        .map(|k| {
            format!(
                "{}={}",
                urlencoding::encode(k),
                urlencoding::encode(&params[k])
            )
        })
        .collect();
    let query_str = query.join("&");
    let sign_str = format!("{}{}", query_str, APP_SEC);
    let hash = Md5::digest(sign_str.as_bytes());
    let sign = hex::encode(hash);
    params.insert("sign".to_string(), sign);
    params.clone()
}
