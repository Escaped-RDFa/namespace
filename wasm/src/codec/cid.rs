use sha2::{Sha256, Digest};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use serde_json;
use js_sys;

// === CID ===

pub(crate) fn sha256_hex(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

pub(crate) fn content_cid(data: &[u8]) -> String {
    format!("bafk{}", &sha256_hex(data)[..32])
}

/// DASL 0xDA51 address from content hash (matches server-side dasl_cid)
pub(crate) fn dasl_address(data: &[u8]) -> String {
    let h = Sha256::digest(data);
    let hi = u64::from_be_bytes(h[0..8].try_into().unwrap());
    format!("0xda51{:012x}", hi & 0xFFFF_FFFF_FFFF)
}

/// Orbifold coordinates mod Monster primes (47, 59, 71)
pub(crate) fn orbifold_coords(data: &[u8]) -> (u8, u8, u8) {
    let h = Sha256::digest(data);
    (h[0] % 71, h[1] % 59, h[2] % 47)
}

/// Build DASL envelope JSON for a block
pub(crate) fn dasl_envelope(data: &[u8], mime: &str, encoding: &str, source: &str) -> String {
    let cid = content_cid(data);
    let dasl = dasl_address(data);
    let (l, m, n) = orbifold_coords(data);
    let bott = Sha256::digest(data)[2] % 8;
    serde_json::json!({
        "prefix": "0xDA51",
        "dasl_type": 3,
        "cid": cid,
        "dasl": dasl,
        "orbifold": [l, m, n],
        "bott": bott,
        "mime": mime,
        "encoding": encoding,
        "size": data.len(),
        "source": source,
        "created": js_sys::Date::new_0().to_iso_string().as_string().unwrap_or_default()
    }).to_string()
}

