use std::collections::HashMap;

/// 将Header分为需要参与签名的Header和剩余Header
pub fn partition_header(
    header_map: HashMap<String, String>,
) -> (HashMap<String, String>, HashMap<String, String>) {
    let mut sign_map = HashMap::new();
    let mut remaining_map = HashMap::new();
    for (k, v) in header_map {
        let k = k.to_lowercase();
        if k == "content-type" || k == "content-md5" || k.starts_with("x-oss-") {
            sign_map.insert(k, v);
        } else {
            remaining_map.insert(k, v);
        }
    }
    (sign_map, remaining_map)
}
