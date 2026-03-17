//! erdfa-publish — publish RDFa triples as CID-addressed CBOR shards
//!
//! ```rust
//! use erdfa_publish::{Shard, ShardSet};
//!
//! let mut set = ShardSet::new("72-names", 72);
//! set.push_triples(&[("_:name0", "erdfa:hebrew", "והו")]);
//! set.push_triples(&[("_:name1", "erdfa:hebrew", "ילי")]);
//! // ... 70 more ...
//! let tar = set.to_tar();       // tar archive of all shards
//! let urls = set.to_urls("https://example.com/erdfa/");
//! let manifest = set.manifest(); // manifest shard listing all CIDs
//! ```

use sha2::{Sha256, Digest};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;

// --- Minimal CBOR encoder (same wire format as wasm/src/codec/cbor.rs) ---

fn cbor_uint(major: u8, val: u64, out: &mut Vec<u8>) {
    let mt = major << 5;
    if val < 24 { out.push(mt | val as u8); }
    else if val <= 0xFF { out.push(mt | 24); out.push(val as u8); }
    else if val <= 0xFFFF { out.push(mt | 25); out.extend(&(val as u16).to_be_bytes()); }
    else if val <= 0xFFFF_FFFF { out.push(mt | 26); out.extend(&(val as u32).to_be_bytes()); }
    else { out.push(mt | 27); out.extend(&val.to_be_bytes()); }
}

fn cbor_str(s: &str, out: &mut Vec<u8>) {
    cbor_uint(3, s.len() as u64, out);
    out.extend(s.as_bytes());
}

fn cbor_array(len: usize, out: &mut Vec<u8>) { cbor_uint(4, len as u64, out); }

/// Encode triples as CBOR: array of [s, p, o] arrays
pub fn encode_triples(triples: &[(&str, &str, &str)]) -> Vec<u8> {
    let mut out = Vec::new();
    cbor_array(triples.len(), &mut out);
    for (s, p, o) in triples {
        cbor_array(3, &mut out);
        cbor_str(s, &mut out);
        cbor_str(p, &mut out);
        cbor_str(o, &mut out);
    }
    out
}

/// SHA-256 → CID string (same format as wasm crate)
pub fn content_cid(data: &[u8]) -> String {
    let hash = Sha256::digest(data);
    format!("baf{}", hex::encode(&hash[..16]))
}

mod hex {
    pub fn encode(data: &[u8]) -> String {
        data.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

// --- Shard ---

#[derive(Clone, Debug)]
pub struct Shard {
    pub index: usize,
    pub cid: String,
    pub cbor: Vec<u8>,
    pub triples: Vec<(String, String, String)>,
}

impl Shard {
    pub fn b64(&self) -> String { B64.encode(&self.cbor) }

    /// Composable eRDFa URL fragment: `?op=decbor&text=<base64>`
    pub fn url_fragment(&self) -> String {
        format!("?op=decbor&text={}", self.b64())
    }

    /// Full URL given a base
    pub fn url(&self, base: &str) -> String {
        format!("{}{}", base.trim_end_matches('/'), self.url_fragment())
    }
}

// --- ShardSet ---

pub struct ShardSet {
    pub name: String,
    pub expected: usize,
    pub shards: Vec<Shard>,
}

impl ShardSet {
    pub fn new(name: &str, expected: usize) -> Self {
        ShardSet { name: name.into(), expected, shards: Vec::new() }
    }

    /// Add a shard from triples
    pub fn push_triples(&mut self, triples: &[(&str, &str, &str)]) {
        let cbor = encode_triples(triples);
        let cid = content_cid(&cbor);
        let index = self.shards.len();
        let owned: Vec<(String, String, String)> = triples.iter()
            .map(|(s, p, o)| (s.to_string(), p.to_string(), o.to_string()))
            .collect();
        self.shards.push(Shard { index, cid, cbor, triples: owned });
    }

    /// Add a shard from owned triples
    pub fn push_owned(&mut self, triples: Vec<(String, String, String)>) {
        let refs: Vec<(&str, &str, &str)> = triples.iter()
            .map(|(s, p, o)| (s.as_str(), p.as_str(), o.as_str()))
            .collect();
        let cbor = encode_triples(&refs);
        let cid = content_cid(&cbor);
        let index = self.shards.len();
        self.shards.push(Shard { index, cid, cbor, triples });
    }

    /// Generate manifest shard (lists all CIDs)
    pub fn manifest(&self) -> Shard {
        let triples: Vec<(&str, &str, &str)> = self.shards.iter()
            .map(|s| ("_:manifest" as &str, "erdfa:shard" as &str, s.cid.as_str()))
            .collect();
        let mut meta: Vec<(&str, &str, &str)> = vec![
            ("_:manifest", "erdfa:name", &self.name),
            ("_:manifest", "rdf:type", "erdfa:ShardSet"),
        ];
        let expected_str = self.expected.to_string();
        meta.push(("_:manifest", "erdfa:expected", &expected_str));
        let mut all = meta;
        all.extend(triples);
        let cbor = encode_triples(&all);
        let cid = content_cid(&cbor);
        let owned: Vec<(String, String, String)> = all.iter()
            .map(|(s, p, o)| (s.to_string(), p.to_string(), o.to_string()))
            .collect();
        Shard { index: self.shards.len(), cid, cbor, triples: owned }
    }

    /// All shard URLs given a base URL
    pub fn to_urls(&self, base: &str) -> Vec<String> {
        let mut urls: Vec<String> = self.shards.iter().map(|s| s.url(base)).collect();
        urls.push(self.manifest().url(base));
        urls
    }

    /// Tar archive: each shard as `shards/{cid}.cbor` + manifest.cbor
    pub fn to_tar(&self) -> Vec<u8> {
        let mut tar = Vec::new();
        for shard in &self.shards {
            tar_append(&mut tar, &format!("shards/{}.cbor", shard.cid), &shard.cbor);
        }
        let manifest = self.manifest();
        tar_append(&mut tar, "manifest.cbor", &manifest.cbor);
        // Also write manifest as JSON for easy inspection
        let json = self.manifest_json();
        tar_append(&mut tar, "manifest.json", json.as_bytes());
        // End-of-archive: two 512-byte zero blocks
        tar.extend(vec![0u8; 1024]);
        tar
    }

    /// Manifest as JSON string
    pub fn manifest_json(&self) -> String {
        let mut json = format!("{{\"name\":\"{}\",\"expected\":{},\"shards\":[", self.name, self.expected);
        for (i, shard) in self.shards.iter().enumerate() {
            if i > 0 { json.push(','); }
            json.push_str(&format!("{{\"index\":{},\"cid\":\"{}\",\"b64\":\"{}\",\"count\":{}}}",
                shard.index, shard.cid, shard.b64(), shard.triples.len()));
        }
        json.push_str("]}");
        json
    }

    pub fn is_complete(&self) -> bool { self.shards.len() >= self.expected }
}

// --- Tar helper (POSIX ustar, same as wasm/src/lib.rs) ---

fn tar_append(tar: &mut Vec<u8>, name: &str, data: &[u8]) {
    let mut header = [0u8; 512];
    let name_bytes = name.as_bytes();
    header[..name_bytes.len().min(100)].copy_from_slice(&name_bytes[..name_bytes.len().min(100)]);
    // Mode
    header[100..107].copy_from_slice(b"0000644");
    // UID/GID
    header[108..115].copy_from_slice(b"0001000");
    header[116..123].copy_from_slice(b"0001000");
    // Size (octal)
    let size_str = format!("{:011o}", data.len());
    header[124..135].copy_from_slice(size_str.as_bytes());
    // Mtime
    header[136..147].copy_from_slice(b"00000000000");
    // Type: regular file
    header[156] = b'0';
    // Magic
    header[257..263].copy_from_slice(b"ustar\0");
    header[263..265].copy_from_slice(b"00");
    // Checksum
    header[148..156].copy_from_slice(b"        ");
    let cksum: u32 = header.iter().map(|&b| b as u32).sum();
    let cksum_str = format!("{:06o}\0 ", cksum);
    header[148..156].copy_from_slice(cksum_str.as_bytes());

    tar.extend_from_slice(&header);
    tar.extend_from_slice(data);
    // Pad to 512-byte boundary
    let pad = (512 - data.len() % 512) % 512;
    tar.extend(vec![0u8; pad]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shard_roundtrip() {
        let mut set = ShardSet::new("test", 2);
        set.push_triples(&[("_:a", "rdf:type", "erdfa:Name"), ("_:a", "erdfa:hebrew", "והו")]);
        set.push_triples(&[("_:b", "rdf:type", "erdfa:Name"), ("_:b", "erdfa:hebrew", "ילי")]);
        assert_eq!(set.shards.len(), 2);
        assert!(set.is_complete());
        assert!(set.shards[0].cid.starts_with("baf"));
        assert!(set.shards[1].cid.starts_with("baf"));
        assert_ne!(set.shards[0].cid, set.shards[1].cid);
    }

    #[test]
    fn manifest_has_all_cids() {
        let mut set = ShardSet::new("72-names", 3);
        set.push_triples(&[("_:a", "erdfa:hebrew", "והו")]);
        set.push_triples(&[("_:b", "erdfa:hebrew", "ילי")]);
        set.push_triples(&[("_:c", "erdfa:hebrew", "סיט")]);
        let m = set.manifest();
        // Manifest triples should reference all 3 shard CIDs
        let shard_refs: Vec<_> = m.triples.iter()
            .filter(|(_, p, _)| p == "erdfa:shard")
            .collect();
        assert_eq!(shard_refs.len(), 3);
    }

    #[test]
    fn url_generation() {
        let mut set = ShardSet::new("test", 1);
        set.push_triples(&[("_:x", "erdfa:val", "hello")]);
        let urls = set.to_urls("https://example.com/erdfa");
        assert_eq!(urls.len(), 2); // 1 shard + manifest
        assert!(urls[0].starts_with("https://example.com/erdfa?op=decbor&text="));
    }

    #[test]
    fn tar_not_empty() {
        let mut set = ShardSet::new("test", 1);
        set.push_triples(&[("_:x", "erdfa:val", "hello")]);
        let tar = set.to_tar();
        assert!(tar.len() > 1024); // at least headers + data + end blocks
        // Check tar magic
        assert_eq!(&tar[257..263], b"ustar\0");
    }

    #[test]
    fn cbor_wire_compat() {
        // Verify our CBOR matches the wasm crate format
        let cbor = encode_triples(&[("s", "p", "o")]);
        // Should be: array(1) [ array(3) [ text("s"), text("p"), text("o") ] ]
        assert_eq!(cbor[0], 0x81); // array of 1
        assert_eq!(cbor[1], 0x83); // array of 3
        assert_eq!(cbor[2], 0x61); // text of length 1
        assert_eq!(cbor[3], b's');
    }
}
