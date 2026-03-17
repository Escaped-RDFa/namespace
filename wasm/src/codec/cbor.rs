// Minimal CBOR encoder/decoder — subset for RDFa triples
// Supports: unsigned int, negative int, byte string, text string, array, map

pub fn encode_uint(major: u8, val: u64, out: &mut Vec<u8>) {
    let mt = major << 5;
    if val < 24 { out.push(mt | val as u8); }
    else if val <= 0xFF { out.push(mt | 24); out.push(val as u8); }
    else if val <= 0xFFFF { out.push(mt | 25); out.extend_from_slice(&(val as u16).to_be_bytes()); }
    else if val <= 0xFFFFFFFF { out.push(mt | 26); out.extend_from_slice(&(val as u32).to_be_bytes()); }
    else { out.push(mt | 27); out.extend_from_slice(&val.to_be_bytes()); }
}

pub fn encode_int(v: i64, out: &mut Vec<u8>) {
    if v >= 0 { encode_uint(0, v as u64, out); }
    else { encode_uint(1, (-1 - v) as u64, out); }
}

pub fn encode_bytes(b: &[u8], out: &mut Vec<u8>) {
    encode_uint(2, b.len() as u64, out);
    out.extend_from_slice(b);
}

pub fn encode_str(s: &str, out: &mut Vec<u8>) {
    encode_uint(3, s.len() as u64, out);
    out.extend_from_slice(s.as_bytes());
}

pub fn encode_array_header(len: usize, out: &mut Vec<u8>) {
    encode_uint(4, len as u64, out);
}

pub fn encode_map_header(len: usize, out: &mut Vec<u8>) {
    encode_uint(5, len as u64, out);
}

// --- Decoder ---

#[derive(Debug, Clone, PartialEq)]
pub enum CborValue {
    Uint(u64),
    Nint(i64),
    Bytes(Vec<u8>),
    Text(String),
    Array(Vec<CborValue>),
    Map(Vec<(CborValue, CborValue)>),
    Null,
}

impl CborValue {
    pub fn as_str(&self) -> Option<&str> {
        if let CborValue::Text(s) = self { Some(s) } else { None }
    }
    pub fn as_u64(&self) -> Option<u64> {
        match self { CborValue::Uint(n) => Some(*n), _ => None }
    }
}

fn decode_head(data: &[u8], pos: &mut usize) -> Option<(u8, u64)> {
    if *pos >= data.len() { return None; }
    let b = data[*pos]; *pos += 1;
    let major = b >> 5;
    let info = b & 0x1F;
    let val = match info {
        0..=23 => info as u64,
        24 => { if *pos >= data.len() { return None; } let v = data[*pos] as u64; *pos += 1; v }
        25 => { if *pos + 2 > data.len() { return None; } let v = u16::from_be_bytes([data[*pos], data[*pos+1]]) as u64; *pos += 2; v }
        26 => { if *pos + 4 > data.len() { return None; } let v = u32::from_be_bytes([data[*pos], data[*pos+1], data[*pos+2], data[*pos+3]]) as u64; *pos += 4; v }
        27 => { if *pos + 8 > data.len() { return None; } let v = u64::from_be_bytes([data[*pos], data[*pos+1], data[*pos+2], data[*pos+3], data[*pos+4], data[*pos+5], data[*pos+6], data[*pos+7]]); *pos += 8; v }
        _ => return None,
    };
    Some((major, val))
}

pub fn decode_value(data: &[u8], pos: &mut usize) -> Option<CborValue> {
    let (major, val) = decode_head(data, pos)?;
    match major {
        0 => Some(CborValue::Uint(val)),
        1 => Some(CborValue::Nint(-(val as i64) - 1)),
        2 => {
            let end = *pos + val as usize;
            if end > data.len() { return None; }
            let b = data[*pos..end].to_vec(); *pos = end;
            Some(CborValue::Bytes(b))
        }
        3 => {
            let end = *pos + val as usize;
            if end > data.len() { return None; }
            let s = core::str::from_utf8(&data[*pos..end]).ok()?.to_string(); *pos = end;
            Some(CborValue::Text(s))
        }
        4 => {
            let mut arr = Vec::with_capacity(val as usize);
            for _ in 0..val { arr.push(decode_value(data, pos)?); }
            Some(CborValue::Array(arr))
        }
        5 => {
            let mut map = Vec::with_capacity(val as usize);
            for _ in 0..val {
                let k = decode_value(data, pos)?;
                let v = decode_value(data, pos)?;
                map.push((k, v));
            }
            Some(CborValue::Map(map))
        }
        7 if val == 22 => Some(CborValue::Null),
        _ => None,
    }
}

pub fn decode(data: &[u8]) -> Option<CborValue> {
    let mut pos = 0;
    decode_value(data, &mut pos)
}

// --- Triple encoding ---

/// Encode triples as CBOR: array of [s, p, o] arrays
pub fn encode_triples(triples: &[(String, String, String)]) -> Vec<u8> {
    let mut out = Vec::new();
    encode_array_header(triples.len(), &mut out);
    for (s, p, o) in triples {
        encode_array_header(3, &mut out);
        encode_str(s, &mut out);
        encode_str(p, &mut out);
        encode_str(o, &mut out);
    }
    out
}

/// Decode CBOR to triples
pub fn decode_triples(data: &[u8]) -> Vec<(String, String, String)> {
    let val = match decode(data) { Some(v) => v, None => return Vec::new() };
    let arr = match val { CborValue::Array(a) => a, _ => return Vec::new() };
    let mut out = Vec::new();
    for item in arr {
        if let CborValue::Array(triple) = item {
            if triple.len() >= 3 {
                if let (Some(s), Some(p), Some(o)) = (
                    triple[0].as_str(), triple[1].as_str(), triple[2].as_str()
                ) {
                    out.push((s.to_string(), p.to_string(), o.to_string()));
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cbor_roundtrip_triples() {
        let triples = vec![
            ("_:a".into(), "rdf:type".into(), "fractran:Program".into()),
            ("_:a".into(), "fractran:fraction".into(), "3/2".into()),
        ];
        let encoded = encode_triples(&triples);
        let decoded = decode_triples(&encoded);
        assert_eq!(decoded, triples);
    }

    #[test]
    fn cbor_int_roundtrip() {
        for v in [0i64, 1, 23, 24, 255, 256, 65535, 65536, -1, -100] {
            let mut buf = Vec::new();
            encode_int(v, &mut buf);
            let decoded = decode(&buf).unwrap();
            match decoded {
                CborValue::Uint(n) => assert_eq!(n as i64, v),
                CborValue::Nint(n) => assert_eq!(n, v),
                _ => panic!("unexpected type for {}", v),
            }
        }
    }

    #[test]
    fn cbor_string_roundtrip() {
        let mut buf = Vec::new();
        encode_str("hello CBOR", &mut buf);
        assert_eq!(decode(&buf).unwrap().as_str().unwrap(), "hello CBOR");
    }

    #[test]
    fn cbor_empty_triples() {
        let encoded = encode_triples(&[]);
        let decoded = decode_triples(&encoded);
        assert!(decoded.is_empty());
    }
}
