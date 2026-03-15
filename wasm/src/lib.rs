use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use base64::{Engine, engine::general_purpose::STANDARD as B64};

// === Tape / Kansas City Standard FSK ===
// 1200 Hz = 0, 2400 Hz = 1. 300 baud. 8N1 framing.

const SAMPLE_RATE: u32 = 44100;
const BAUD: u32 = 300;
const FREQ_ZERO: f32 = 1200.0;
const FREQ_ONE: f32 = 2400.0;
const SAMPLES_PER_BIT: u32 = SAMPLE_RATE / BAUD;

fn fsk_bit(bit: bool, out: &mut Vec<i16>) {
    let f = if bit { FREQ_ONE } else { FREQ_ZERO };
    for i in 0..SAMPLES_PER_BIT {
        let t = i as f32 / SAMPLE_RATE as f32;
        out.push(((2.0 * core::f32::consts::PI * f * t).sin() * 24000.0) as i16);
    }
}

fn fsk_byte(byte: u8, out: &mut Vec<i16>) {
    fsk_bit(false, out); // start
    for i in 0..8 { fsk_bit((byte >> i) & 1 == 1, out); }
    fsk_bit(true, out); // stop
}

fn tape_encode(data: &[u8]) -> Vec<i16> {
    let mut s = Vec::new();
    for _ in 0..(SAMPLE_RATE / SAMPLES_PER_BIT) { fsk_bit(true, &mut s); } // 1s leader
    for &b in &(data.len() as u32).to_be_bytes() { fsk_byte(b, &mut s); }
    for &b in data { fsk_byte(b, &mut s); }
    for _ in 0..(SAMPLE_RATE / SAMPLES_PER_BIT / 2) { fsk_bit(true, &mut s); } // trailer
    s
}

fn tape_decode(samples: &[i16]) -> Option<Vec<u8>> {
    let w = SAMPLES_PER_BIT as usize;
    let mut bits = Vec::new();
    let mut pos = 0;
    while pos + w <= samples.len() {
        let crossings = (1..w).filter(|&i|
            (samples[pos + i] >= 0) != (samples[pos + i - 1] >= 0)
        ).count();
        bits.push(crossings > 6);
        pos += w;
    }
    let start = bits.iter().position(|b| !b)?;
    let mut bytes = Vec::new();
    let mut i = start;
    while i + 10 <= bits.len() {
        if bits[i] { i += 1; continue; }
        let mut byte = 0u8;
        for b in 0..8 { if bits[i + 1 + b] { byte |= 1 << b; } }
        bytes.push(byte);
        i += 10;
    }
    if bytes.len() < 4 { return None; }
    let len = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    if bytes.len() < 4 + len { return None; }
    Some(bytes[4..4 + len].to_vec())
}

fn samples_to_wav(samples: &[i16]) -> Vec<u8> {
    let dlen = (samples.len() * 2) as u32;
    let mut w = Vec::with_capacity(44 + dlen as usize);
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + dlen).to_le_bytes());
    w.extend_from_slice(b"WAVEfmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes()); // PCM
    w.extend_from_slice(&1u16.to_le_bytes()); // mono
    w.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    w.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    w.extend_from_slice(&2u16.to_le_bytes());
    w.extend_from_slice(&16u16.to_le_bytes());
    w.extend_from_slice(b"data");
    w.extend_from_slice(&dlen.to_le_bytes());
    for &s in samples { w.extend_from_slice(&s.to_le_bytes()); }
    w
}

fn wav_to_samples(wav: &[u8]) -> Option<Vec<i16>> {
    if wav.len() < 44 || &wav[0..4] != b"RIFF" { return None; }
    let mut pos = 12;
    while pos + 8 < wav.len() {
        let size = u32::from_le_bytes([wav[pos+4], wav[pos+5], wav[pos+6], wav[pos+7]]) as usize;
        if &wav[pos..pos+4] == b"data" {
            return Some(wav[pos+8..pos+8+size.min(wav.len()-pos-8)]
                .chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect());
        }
        pos += 8 + size;
    }
    None
}

// === Numbers Station ===
// Encode data as hex digit groups, generate distinct tone per digit (0-F).
// Each digit = unique frequency, spoken in groups of 5 with pauses.
// Decimal encoding: each byte → 3 decimal digits (000-255), only 0-9 used.

const NUM_FREQS: [f32; 10] = [
    330.0, 370.0, 415.0, 466.0, 523.0, 587.0, 659.0, 740.0, 831.0, 932.0,
];
const DIGIT_SAMPLES: u32 = SAMPLE_RATE / 3; // 333ms per digit
const PAUSE_SAMPLES: u32 = SAMPLE_RATE / 6; // 166ms pause
const GROUP_PAUSE: u32 = SAMPLE_RATE / 2;   // 500ms between groups

/// Encode bytes as decimal digit string: each byte → 3 digits (000-255)
fn bytes_to_decimal(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:03}", b)).collect()
}

/// Decode decimal digit string back to bytes
fn decimal_to_bytes(digits: &str) -> Option<Vec<u8>> {
    if digits.len() % 3 != 0 { return None; }
    digits.as_bytes().chunks(3).map(|c| {
        std::str::from_utf8(c).ok()?.parse::<u16>().ok().filter(|&v| v <= 255).map(|v| v as u8)
    }).collect()
}

fn numbers_station_encode(data: &[u8]) -> Vec<i16> {
    let dec = bytes_to_decimal(data);
    let mut out = Vec::new();
    // Preamble: 3 beeps at 1000Hz
    for _ in 0..3 {
        for i in 0..(SAMPLE_RATE / 4) {
            let t = i as f32 / SAMPLE_RATE as f32;
            out.push(((2.0 * core::f32::consts::PI * 1000.0 * t).sin() * 20000.0) as i16);
        }
        out.extend(vec![0i16; PAUSE_SAMPLES as usize]);
    }
    // Digits in groups of 5
    for (i, c) in dec.chars().enumerate() {
        if i > 0 && i % 5 == 0 {
            out.extend(vec![0i16; GROUP_PAUSE as usize]);
        }
        let d = c.to_digit(10).unwrap_or(0) as usize;
        let freq = NUM_FREQS[d];
        for j in 0..DIGIT_SAMPLES {
            let t = j as f32 / SAMPLE_RATE as f32;
            out.push(((2.0 * core::f32::consts::PI * freq * t).sin() * 20000.0) as i16);
        }
        out.extend(vec![0i16; PAUSE_SAMPLES as usize]);
    }
    out
}

fn numbers_station_decode(samples: &[i16]) -> Option<Vec<u8>> {
    let chunk = DIGIT_SAMPLES as usize;
    let pause = PAUSE_SAMPLES as usize;
    let mut digits = Vec::new();
    let mut pos = 0;
    while pos + chunk < samples.len() {
        let energy: f64 = samples[pos..pos+chunk].iter().map(|&s| (s as f64).powi(2)).sum();
        if energy > 1e8 {
            let crossings = (1..chunk).filter(|&i|
                (samples[pos+i] >= 0) != (samples[pos+i-1] >= 0)
            ).count();
            let freq_est = crossings as f32 * SAMPLE_RATE as f32 / (2.0 * chunk as f32);
            // Skip preamble tones (1000Hz)
            if (freq_est - 1000.0).abs() > 50.0 {
                let digit = NUM_FREQS.iter().enumerate()
                    .min_by_key(|(_, &f)| ((f - freq_est).abs() * 100.0) as u32)
                    .map(|(i, _)| i as u8)
                    .unwrap_or(0);
                digits.push(digit);
            }
        }
        pos += chunk + pause;
    }
    let dec_str: String = digits.iter().map(|d| format!("{}", d)).collect();
    decimal_to_bytes(&dec_str)
}

/// Digit speech samples loaded on demand from server (44100Hz 16-bit mono PCM)
use std::cell::RefCell;
thread_local! {
    static VOICE_SAMPLES: RefCell<Vec<Vec<i16>>> = RefCell::new(vec![Vec::new(); 10]);
}

fn pcm_to_i16(raw: &[u8]) -> Vec<i16> {
    raw.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect()
}

/// Load a digit sample from JS (call once per digit 0-9)
#[wasm_bindgen]
pub fn load_digit_pcm(digit: u8, raw: &[u8]) {
    if digit > 9 { return; }
    VOICE_SAMPLES.with(|ds| ds.borrow_mut()[digit as usize] = pcm_to_i16(raw));
}

fn numbers_speech_synth(data: &[u8]) -> Vec<i16> {
    let dec = bytes_to_decimal(data);
    let gap = vec![0i16; SAMPLE_RATE as usize / 10];
    let group_gap = vec![0i16; SAMPLE_RATE as usize / 3];
    let mut out = Vec::new();
    VOICE_SAMPLES.with(|ds| {
        let ds = ds.borrow();
        for (i, ch) in dec.chars().enumerate() {
            if i > 0 && i % 2 == 0 { out.extend(&group_gap); }
            let d = ch.to_digit(10).unwrap_or(0) as usize;
            out.extend(&ds[d]);
            out.extend(&gap);
        }
    });
    out
}

// === CID ===

fn sha256_hex(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn content_cid(data: &[u8]) -> String {
    format!("bafk{}", &sha256_hex(data)[..32])
}

/// DASL 0xDA51 address from content hash (matches server-side dasl_cid)
fn dasl_address(data: &[u8]) -> String {
    let h = Sha256::digest(data);
    let hi = u64::from_be_bytes(h[0..8].try_into().unwrap());
    format!("0xda51{:012x}", hi & 0xFFFF_FFFF_FFFF)
}

/// Orbifold coordinates mod Monster primes (47, 59, 71)
fn orbifold_coords(data: &[u8]) -> (u8, u8, u8) {
    let h = Sha256::digest(data);
    (h[0] % 71, h[1] % 59, h[2] % 47)
}

/// Build DASL envelope JSON for a block
fn dasl_envelope(data: &[u8], mime: &str, encoding: &str, source: &str) -> String {
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

// === Morse ===

const MORSE: &[(char, &str)] = &[
    ('A',".-"),('B',"-..."),('C',"-.-."),('D',"-.."),('E',"."),
    ('F',"..-."),('G',"--."),('H',"...."),('I',".."),('J',".---"),
    ('K',"-.-"),('L',".-.."),('M',"--"),('N',"-."),('O',"---"),
    ('P',".--."),('Q',"--.-"),('R',".-."),('S',"..."),('T',"-"),
    ('U',"..-"),('V',"...-"),('W',".--"),('X',"-..-"),('Y',"-.--"),
    ('Z',"--.."),('0',"-----"),('1',".----"),('2',"..---"),
    ('3',"...--"),('4',"....-"),('5',"....."),('6',"-...."),
    ('7',"--..."),('8',"---.."),('9',"----."),(' ',"/"),
    ('.',".-.-."),(',',"--..-"),('?',"..--.."),('/',"-..-."),
    (':',"---..."),('=',"-..-"),
];

fn char_to_morse(c: char) -> &'static str {
    let u = c.to_ascii_uppercase();
    MORSE.iter().find(|(ch, _)| *ch == u).map(|(_, m)| *m).unwrap_or("?")
}

fn morse_to_char(m: &str) -> char {
    MORSE.iter().find(|(_, code)| *code == m).map(|(c, _)| *c).unwrap_or('?')
}

fn encode_morse(text: &str) -> String {
    text.chars().map(char_to_morse).collect::<Vec<_>>().join(" ")
}

fn decode_morse(morse: &str) -> String {
    morse.split(' ').map(|m| {
        if m == "/" { ' ' } else { morse_to_char(m) }
    }).collect()
}

// === BBS / FSK audio tones ===
// Encode bytes as hex, each hex digit maps to a DTMF-like frequency pair.
// Low group: 697, 770, 852, 941 Hz  High group: 1209, 1336, 1477, 1633 Hz

const DTMF_LOW: [f32; 4] = [697.0, 770.0, 852.0, 941.0];
const DTMF_HIGH: [f32; 4] = [1209.0, 1336.0, 1477.0, 1633.0];

fn hex_digit_freqs(d: u8) -> (f32, f32) {
    let row = (d >> 2) & 0x3;
    let col = d & 0x3;
    (DTMF_LOW[row as usize], DTMF_HIGH[col as usize])
}

// === Stego (LSB in RGBA pixels) ===

fn stego_encode(data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let capacity = (width * height * 3) as usize / 8; // 3 channels, 1 bit each
    let mut payload = Vec::new();
    // length prefix (4 bytes big-endian)
    let len = data.len().min(capacity.saturating_sub(4)) as u32;
    payload.extend_from_slice(&len.to_be_bytes());
    payload.extend_from_slice(&data[..len as usize]);

    let total_pixels = (width * height) as usize;
    let mut pixels = vec![255u8; total_pixels * 4]; // RGBA white

    let mut bit_idx = 0usize;
    let total_bits = payload.len() * 8;
    for i in 0..total_pixels {
        for ch in 0..3u8 { // R, G, B
            if bit_idx < total_bits {
                let byte = payload[bit_idx / 8];
                let bit = (byte >> (7 - (bit_idx % 8))) & 1;
                pixels[i * 4 + ch as usize] = 254 + bit; // 254 or 255
                bit_idx += 1;
            }
        }
    }
    pixels
}

fn stego_decode(pixels: &[u8]) -> Vec<u8> {
    let mut bits = Vec::new();
    let num_pixels = pixels.len() / 4;
    for i in 0..num_pixels {
        for ch in 0..3usize {
            bits.push(pixels[i * 4 + ch] & 1);
        }
    }
    if bits.len() < 32 { return Vec::new(); }
    // read length prefix
    let mut len_bytes = [0u8; 4];
    for i in 0..32 {
        len_bytes[i / 8] |= bits[i] << (7 - (i % 8));
    }
    let len = u32::from_be_bytes(len_bytes) as usize;
    let data_bits = &bits[32..];
    if data_bits.len() < len * 8 { return Vec::new(); }
    let mut out = vec![0u8; len];
    for i in 0..len * 8 {
        out[i / 8] |= data_bits[i] << (7 - (i % 8));
    }
    out
}

// === URL encoding ===

fn encode_data_url(data: &[u8], mime: &str) -> String {
    format!("data:{};base64,{}", mime, B64.encode(data))
}

fn decode_data_url(url: &str) -> Option<Vec<u8>> {
    let b64 = url.split(",").nth(1)?;
    B64.decode(b64).ok()
}

// === QR (numeric encoding as text — actual QR rendering done in JS/canvas) ===

fn encode_qr_payload(data: &[u8]) -> String {
    // Encode as base64 for QR text mode
    B64.encode(data)
}

fn decode_qr_payload(payload: &str) -> Option<Vec<u8>> {
    B64.decode(payload).ok()
}

// === localStorage paste store ===

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

fn store_put(cid: &str, data: &str) {
    if let Some(s) = storage() {
        let _ = s.set_item(&format!("erdfa:{}", cid), data);
    }
}

fn store_get(cid: &str) -> Option<String> {
    storage()?.get_item(&format!("erdfa:{}", cid)).ok()?
}

fn store_list() -> Vec<String> {
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

fn store_remove(cid: &str) {
    if let Some(s) = storage() {
        let _ = s.remove_item(&format!("erdfa:{}", cid));
    }
}

// === WASM exports ===

/// Minimal tar header writer (POSIX ustar)
fn tar_append(tar: &mut Vec<u8>, name: &str, data: &[u8]) {
    let mut header = [0u8; 512];
    // name (0..100)
    let name_bytes = name.as_bytes();
    header[..name_bytes.len().min(100)].copy_from_slice(&name_bytes[..name_bytes.len().min(100)]);
    // mode (100..108)
    header[100..107].copy_from_slice(b"0000644");
    // uid/gid (108..124) - zeros ok
    // size (124..136) - octal
    let size_str = format!("{:011o}", data.len());
    header[124..135].copy_from_slice(size_str.as_bytes());
    // mtime (136..148)
    header[136..147].copy_from_slice(b"00000000000");
    // typeflag (156) - '0' = regular file
    header[156] = b'0';
    // magic (257..263)
    header[257..263].copy_from_slice(b"ustar\0");
    // version (263..265)
    header[263..265].copy_from_slice(b"00");
    // checksum (148..156) - compute
    header[148..156].copy_from_slice(b"        "); // 8 spaces for checksum calc
    let cksum: u32 = header.iter().map(|&b| b as u32).sum();
    let cksum_str = format!("{:06o}\0 ", cksum);
    header[148..156].copy_from_slice(cksum_str.as_bytes());
    tar.extend_from_slice(&header);
    tar.extend_from_slice(data);
    // Pad to 512-byte boundary
    let pad = (512 - (data.len() % 512)) % 512;
    tar.extend(vec![0u8; pad]);
}

#[wasm_bindgen]
pub struct Pad;

#[wasm_bindgen]
impl Pad {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self { Pad }

    // -- CID --
    pub fn cid(&self, data: &str) -> String { content_cid(data.as_bytes()) }
    pub fn sha256(&self, data: &str) -> String { sha256_hex(data.as_bytes()) }

    // -- Morse --
    pub fn to_morse(&self, text: &str) -> String { encode_morse(text) }
    pub fn from_morse(&self, morse: &str) -> String { decode_morse(morse) }
    /// Morse as WAV bytes
    pub fn to_morse_wav(&self, text: &str, wpm: f64) -> Vec<u8> {
        let morse = encode_morse(text);
        let dot = (1.2 / wpm * SAMPLE_RATE as f64) as u32;
        let freq = 700.0f32;
        let mut samples = Vec::new();
        for c in morse.chars() {
            let dur = match c {
                '.' => dot,
                '-' => dot * 3,
                ' ' => { samples.extend(vec![0i16; dot as usize * 3]); continue; }
                '/' => { samples.extend(vec![0i16; dot as usize * 7]); continue; }
                _ => continue,
            };
            for i in 0..dur {
                let t = i as f32 / SAMPLE_RATE as f32;
                samples.push(((2.0 * core::f32::consts::PI * freq * t).sin() * 24000.0) as i16);
            }
            samples.extend(vec![0i16; dot as usize]); // inter-element gap
        }
        samples_to_wav(&samples)
    }

    // -- Base64 --
    pub fn to_base64(&self, data: &str) -> String { B64.encode(data.as_bytes()) }
    pub fn from_base64(&self, b64: &str) -> Option<String> {
        B64.decode(b64).ok().and_then(|b| String::from_utf8(b).ok())
    }

    // -- URL --
    pub fn to_data_url(&self, data: &str, mime: &str) -> String {
        encode_data_url(data.as_bytes(), mime)
    }
    pub fn from_data_url(&self, url: &str) -> Option<String> {
        decode_data_url(url).and_then(|b| String::from_utf8(b).ok())
    }

    // -- QR payload --
    pub fn to_qr(&self, data: &str) -> String { encode_qr_payload(data.as_bytes()) }
    pub fn from_qr(&self, payload: &str) -> Option<String> {
        decode_qr_payload(payload).and_then(|b| String::from_utf8(b).ok())
    }

    // -- Stego --
    pub fn stego_encode(&self, data: &str, width: u32, height: u32) -> Vec<u8> {
        stego_encode(data.as_bytes(), width, height)
    }
    pub fn stego_decode(&self, pixels: &[u8]) -> Option<String> {
        let decoded = stego_decode(pixels);
        String::from_utf8(decoded).ok()
    }

    // -- BBS/DTMF frequencies for a hex string --
    pub fn to_dtmf_freqs(&self, data: &str) -> String {
        let hex_str = hex::encode(data.as_bytes());
        let freqs: Vec<String> = hex_str.chars().filter_map(|c| {
            let d = u8::from_str_radix(&c.to_string(), 16).ok()?;
            let (lo, hi) = hex_digit_freqs(d);
            Some(format!("{},{}", lo, hi))
        }).collect();
        freqs.join(";")
    }

    /// BBS/DTMF tones as WAV bytes
    pub fn to_bbs_wav(&self, data: &str, tone_ms: f64) -> Vec<u8> {
        let hex_str = hex::encode(data.as_bytes());
        let tone_samples = (SAMPLE_RATE as f64 * tone_ms / 1000.0) as u32;
        let pause_samples = tone_samples / 2;
        let mut samples = Vec::new();
        for c in hex_str.chars() {
            if let Ok(d) = u8::from_str_radix(&c.to_string(), 16) {
                let (lo, hi) = hex_digit_freqs(d);
                for i in 0..tone_samples {
                    let t = i as f32 / SAMPLE_RATE as f32;
                    let s = (2.0 * core::f32::consts::PI * lo * t).sin()
                          + (2.0 * core::f32::consts::PI * hi * t).sin();
                    samples.push((s * 12000.0) as i16);
                }
                samples.extend(vec![0i16; pause_samples as usize]);
            }
        }
        samples_to_wav(&samples)
    }

    // -- Store --
    pub fn store_paste(&self, content: &str) -> String {
        let cid = content_cid(content.as_bytes());
        store_put(&cid, content);
        cid
    }
    pub fn load_paste(&self, cid: &str) -> Option<String> { store_get(cid) }
    pub fn list_pastes(&self) -> String {
        serde_json::to_string(&store_list()).unwrap_or_default()
    }
    pub fn delete_paste(&self, cid: &str) { store_remove(cid) }

    // -- Local IPFS block store --
    /// Store arbitrary bytes, return CID
    pub fn ipfs_put(&self, data: &[u8]) -> String {
        let cid = content_cid(data);
        if let Some(s) = storage() {
            let _ = s.set_item(&format!("ipfs:{}", cid), &B64.encode(data));
        }
        cid
    }
    /// Store with explicit mime type (stored as metadata)
    pub fn ipfs_put_typed(&self, data: &[u8], mime: &str) -> String {
        self.ipfs_put_dasl(data, mime, "raw", "")
    }
    /// Store with DASL envelope: mime + encoding type + source text
    /// Auto-creates RDFa triple: source_cid --erdfa:{encoding}--> block_cid
    pub fn ipfs_put_dasl(&self, data: &[u8], mime: &str, encoding: &str, source: &str) -> String {
        let cid = content_cid(data);
        if let Some(s) = storage() {
            let _ = s.set_item(&format!("ipfs:{}", cid), &B64.encode(data));
            let envelope = dasl_envelope(data, mime, encoding, source);
            let _ = s.set_item(&format!("ipfs-meta:{}", cid), &envelope);
        }
        // Auto-link source → encoded block as triple
        if !source.is_empty() {
            let src_cid = content_cid(source.as_bytes());
            let pred = format!("erdfa:{}", encoding);
            let ts = TripleStore;
            ts.add(&src_cid, &pred, &cid);
        }
        cid
    }
    /// Retrieve bytes by CID
    pub fn ipfs_get(&self, cid: &str) -> Option<Vec<u8>> {
        let s = storage()?;
        let b64 = s.get_item(&format!("ipfs:{}", cid)).ok()??;
        B64.decode(&b64).ok()
    }
    /// Get DASL envelope JSON for a CID
    pub fn ipfs_meta(&self, cid: &str) -> Option<String> {
        storage()?.get_item(&format!("ipfs-meta:{}", cid)).ok()?
    }
    /// List all IPFS CIDs
    pub fn ipfs_list(&self) -> String {
        let Some(s) = storage() else { return "[]".into() };
        let len = s.length().unwrap_or(0);
        let mut cids = Vec::new();
        for i in 0..len {
            if let Ok(Some(k)) = s.key(i) {
                if let Some(cid) = k.strip_prefix("ipfs:") {
                    if !cid.contains("meta:") { cids.push(cid.to_string()); }
                }
            }
        }
        serde_json::to_string(&cids).unwrap_or("[]".into())
    }
    /// Total size of local IPFS store
    pub fn ipfs_size(&self) -> u32 {
        let Some(s) = storage() else { return 0 };
        let len = s.length().unwrap_or(0);
        let mut total = 0u32;
        for i in 0..len {
            if let Ok(Some(k)) = s.key(i) {
                if k.starts_with("ipfs:") {
                    if let Ok(Some(v)) = s.get_item(&k) { total += v.len() as u32; }
                }
            }
        }
        total
    }

    // -- Export as tar archive --
    /// Export all IPFS blocks as a tar file (bytes)
    pub fn export_tar(&self) -> Vec<u8> {
        let Some(s) = storage() else { return Vec::new() };
        let len = s.length().unwrap_or(0);
        let mut tar = Vec::new();
        for i in 0..len {
            let Ok(Some(k)) = s.key(i) else { continue };
            let Some(cid) = k.strip_prefix("ipfs:") else { continue };
            if cid.starts_with("meta:") { continue; }
            let Ok(Some(b64)) = s.get_item(&k) else { continue };
            let Ok(data) = B64.decode(&b64) else { continue };
            let mime = s.get_item(&format!("ipfs-meta:{}", cid))
                .ok().flatten().unwrap_or_default();
            let ext = match mime.as_str() {
                "audio/wav" => ".wav",
                "image/png" => ".png",
                "text/plain" => ".txt",
                "text/html" => ".html",
                "text/x-morse" => ".morse",
                _ => "",
            };
            let name = format!("blocks/{}{}", cid, ext);
            tar_append(&mut tar, &name, &data);
        }
        // Also export triples
        if let Ok(Some(tj)) = s.get_item("erdfa:triples") {
            tar_append(&mut tar, "triples.json", tj.as_bytes());
        }
        // Tar end: two 512-byte zero blocks
        tar.extend(vec![0u8; 1024]);
        tar
    }

    // -- Tape (KCS FSK) --
    /// Encode data as KCS tape WAV (base64 data: URL)
    pub fn to_tape_url(&self, data: &str) -> String {
        let samples = tape_encode(data.as_bytes());
        let wav = samples_to_wav(&samples);
        format!("data:audio/wav;base64,{}", B64.encode(&wav))
    }
    /// Encode data as raw WAV bytes
    pub fn to_tape_wav(&self, data: &str) -> Vec<u8> {
        samples_to_wav(&tape_encode(data.as_bytes()))
    }
    /// Decode WAV bytes back to text
    pub fn from_tape_wav(&self, wav: &[u8]) -> Option<String> {
        let samples = wav_to_samples(wav)?;
        let data = tape_decode(&samples)?;
        String::from_utf8(data).ok()
    }

    // -- Numbers Station --
    /// Encode data as numbers station WAV (base64 data: URL)
    pub fn to_numbers_url(&self, data: &str) -> String {
        let samples = numbers_station_encode(data.as_bytes());
        let wav = samples_to_wav(&samples);
        format!("data:audio/wav;base64,{}", B64.encode(&wav))
    }
    /// Encode as WAV bytes (tones)
    pub fn to_numbers_wav(&self, data: &str) -> Vec<u8> {
        samples_to_wav(&numbers_station_encode(data.as_bytes()))
    }
    /// Encode as speech WAV (formant-synthesized digits)
    pub fn to_numbers_speech_wav(&self, data: &str) -> Vec<u8> {
        samples_to_wav(&numbers_speech_synth(data.as_bytes()))
    }
    /// Decode numbers station WAV back to text
    pub fn from_numbers_wav(&self, wav: &[u8]) -> Option<String> {
        let samples = wav_to_samples(wav)?;
        let data = numbers_station_decode(&samples)?;
        String::from_utf8(data).ok()
    }
    /// Get decimal digit groups for speech synthesis (pairs)
    pub fn to_number_groups(&self, data: &str) -> String {
        let dec = bytes_to_decimal(data.as_bytes());
        dec.as_bytes().chunks(2)
            .map(|g| std::str::from_utf8(g).unwrap_or(""))
            .collect::<Vec<_>>()
            .join(" ")
    }

    // -- Play BBS tones via Web Audio API --
    pub fn play_bbs(&self, data: &str, tone_ms: f64) -> Result<(), JsValue> {
        let ctx = web_sys::AudioContext::new()?;
        let hex_str = hex::encode(data.as_bytes());
        let mut time = ctx.current_time();
        let dur = tone_ms / 1000.0;

        for c in hex_str.chars() {
            let d = u8::from_str_radix(&c.to_string(), 16).unwrap_or(0);
            let (lo, hi) = hex_digit_freqs(d);

            let osc_lo = ctx.create_oscillator()?;
            osc_lo.set_type(web_sys::OscillatorType::Sine);
            osc_lo.frequency().set_value(lo);
            let gain_lo = ctx.create_gain()?;
            gain_lo.gain().set_value(0.3);
            osc_lo.connect_with_audio_node(&gain_lo)?;
            gain_lo.connect_with_audio_node(&ctx.destination())?;
            osc_lo.start_with_when(time)?;
            osc_lo.stop_with_when(time + dur)?;

            let osc_hi = ctx.create_oscillator()?;
            osc_hi.set_type(web_sys::OscillatorType::Sine);
            osc_hi.frequency().set_value(hi);
            let gain_hi = ctx.create_gain()?;
            gain_hi.gain().set_value(0.3);
            osc_hi.connect_with_audio_node(&gain_hi)?;
            gain_hi.connect_with_audio_node(&ctx.destination())?;
            osc_hi.start_with_when(time)?;
            osc_hi.stop_with_when(time + dur)?;

            time += dur + 0.02; // 20ms gap
        }
        Ok(())
    }

    // -- Play morse via Web Audio API --
    pub fn play_morse(&self, text: &str, wpm: f64) -> Result<(), JsValue> {
        let ctx = web_sys::AudioContext::new()?;
        let morse = encode_morse(text);
        let dot = 1.2 / wpm; // seconds per dot
        let mut time = ctx.current_time();
        let freq = 700.0;

        for c in morse.chars() {
            let dur = match c {
                '.' => dot,
                '-' => dot * 3.0,
                ' ' => { time += dot * 3.0; continue; }
                '/' => { time += dot * 7.0; continue; }
                _ => continue,
            };
            let osc = ctx.create_oscillator()?;
            osc.set_type(web_sys::OscillatorType::Sine);
            osc.frequency().set_value(freq as f32);
            let gain = ctx.create_gain()?;
            gain.gain().set_value(0.5);
            osc.connect_with_audio_node(&gain)?;
            gain.connect_with_audio_node(&ctx.destination())?;
            osc.start_with_when(time)?;
            osc.stop_with_when(time + dur)?;
            time += dur + dot; // inter-element gap
        }
        Ok(())
    }
}

#[wasm_bindgen(start)]
pub fn main() {}

// === RDFa Triple Store ===

#[derive(Serialize, Deserialize, Clone)]
pub struct Triple {
    pub s: String,
    pub p: String,
    pub o: String,
}

#[wasm_bindgen]
pub struct TripleStore;

#[wasm_bindgen]
impl TripleStore {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self { TripleStore }

    /// Add a triple, store under its CID
    pub fn add(&self, subject: &str, predicate: &str, object: &str) -> String {
        let t = Triple { s: subject.into(), p: predicate.into(), o: object.into() };
        let json = serde_json::to_string(&t).unwrap_or_default();
        let cid = content_cid(json.as_bytes());
        // Append to triples list in localStorage
        let mut all = self.load_all();
        all.push(t);
        if let Some(s) = storage() {
            let _ = s.set_item("erdfa:triples", &serde_json::to_string(&all).unwrap_or_default());
        }
        cid
    }

    /// Get all triples as JSON
    pub fn list(&self) -> String {
        serde_json::to_string(&self.load_all()).unwrap_or("[]".into())
    }

    /// Query triples by subject, predicate, or object (empty = wildcard)
    pub fn query(&self, subject: &str, predicate: &str, object: &str) -> String {
        let all = self.load_all();
        let results: Vec<&Triple> = all.iter().filter(|t| {
            (subject.is_empty() || t.s == subject) &&
            (predicate.is_empty() || t.p == predicate) &&
            (object.is_empty() || t.o == object)
        }).collect();
        serde_json::to_string(&results).unwrap_or("[]".into())
    }

    /// Export all triples as RDFa HTML
    pub fn to_rdfa(&self) -> String {
        let all = self.load_all();
        let mut html = String::from("<div vocab=\"https://dasl.dev/ns/\">\n");
        for t in &all {
            html.push_str(&format!(
                "  <div about=\"{}\" property=\"{}\">{}</div>\n",
                t.s, t.p, t.o
            ));
        }
        html.push_str("</div>");
        html
    }

    /// Export as N-Triples
    pub fn to_ntriples(&self) -> String {
        self.load_all().iter().map(|t|
            format!("<{}> <{}> \"{}\" .", t.s, t.p, t.o)
        ).collect::<Vec<_>>().join("\n")
    }

    /// Import from N-Triples
    pub fn from_ntriples(&self, nt: &str) -> u32 {
        let mut count = 0u32;
        for line in nt.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            // Simple parse: <s> <p> "o" .
            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() >= 3 {
                let s = parts[0].trim_matches(|c| c == '<' || c == '>');
                let p = parts[1].trim_matches(|c| c == '<' || c == '>');
                let rest = parts[2].trim_end_matches(" .");
                let o = rest.trim_matches('"');
                self.add(s, p, o);
                count += 1;
            }
        }
        count
    }

    /// Clear all triples
    pub fn clear(&self) {
        if let Some(s) = storage() { let _ = s.remove_item("erdfa:triples"); }
    }

    fn load_all(&self) -> Vec<Triple> {
        storage()
            .and_then(|s| s.get_item("erdfa:triples").ok()?)
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default()
    }
}
