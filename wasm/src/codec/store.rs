use web_sys;
use base64::{Engine, engine::general_purpose::STANDARD as B64};

// === URL encoding ===

pub(crate) fn encode_data_url(data: &[u8], mime: &str) -> String {
    format!("data:{};base64,{}", mime, B64.encode(data))
}

pub(crate) fn decode_data_url(url: &str) -> Option<Vec<u8>> {
    let b64 = url.split(",").nth(1)?;
    B64.decode(b64).ok()
}

// === QR (numeric encoding as text — actual QR rendering done in JS/canvas) ===

pub(crate) fn encode_qr_payload(data: &[u8]) -> String {
    // Encode as base64 for QR text mode
    B64.encode(data)
}

pub(crate) fn decode_qr_payload(payload: &str) -> Option<Vec<u8>> {
    B64.decode(payload).ok()
}

// === localStorage paste store ===

pub(crate) fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

pub(crate) fn store_put(cid: &str, data: &str) {
    if let Some(s) = storage() {
        let _ = s.set_item(&format!("erdfa:{}", cid), data);
    }
}

pub(crate) fn store_get(cid: &str) -> Option<String> {
    storage()?.get_item(&format!("erdfa:{}", cid)).ok()?
}

pub(crate) fn store_list() -> Vec<String> {
    let Some(s) = storage() else { return Vec::new() };
    let len = s.length().unwrap_or(0);
    let mut keys = Vec::new();
    for i in 0..len {
        if let Ok(Some(k)) = s.key(i) {
            if let Some(cid) = k.strip_prefix("erdfa:") {
                keys.push(cid.to_string());
            }
        }
    }
    keys
}

pub(crate) fn store_remove(cid: &str) {
    if let Some(s) = storage() {
        let _ = s.remove_item(&format!("erdfa:{}", cid));
    }
}

