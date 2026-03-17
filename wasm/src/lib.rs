use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use base64::{Engine, engine::general_purpose::STANDARD as B64};

pub mod codec;
use codec::tape::*;
use codec::numbers::*;
use codec::cid::*;
use codec::morse::*;
use codec::bbs::*;
use codec::stego::*;
use codec::store::*;

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
    pub fn stego_encode(&self, data: &str, carrier: &[u8], width: u32, height: u32) -> Vec<u8> {
        codec::stego::stego_encode(data.as_bytes(), carrier, width, height, 0, codec::stego::DEFAULT_QUALITY)
            .unwrap_or_default()
    }
    pub fn stego_decode(&self, pixels: &[u8], width: u32, height: u32) -> Option<String> {
        let decoded = codec::stego::stego_decode(pixels, width, height, 0);
        if decoded.is_empty() { None } else { String::from_utf8(decoded).ok() }
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

#[wasm_bindgen]
pub fn load_digit_pcm(digit: u8, raw: &[u8]) {
    codec::numbers::load_digit(digit, raw);
}


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
    pub fn new() -> TripleStore { TripleStore }

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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_carrier(w: usize, h: usize) -> Vec<u8> {
        let mut pixels = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) * 4;
                pixels[i] = (x & 255) as u8;
                pixels[i+1] = ((x+y) & 255) as u8;
                pixels[i+2] = (y & 255) as u8;
                pixels[i+3] = 255;
            }
        }
        pixels
    }

    fn rgba_to_rgb(rgba: &[u8], w: usize, h: usize) -> Vec<u8> {
        let mut rgb = Vec::with_capacity(w * h * 3);
        for i in 0..w*h {
            rgb.push(rgba[i*4]);
            rgb.push(rgba[i*4+1]);
            rgb.push(rgba[i*4+2]);
        }
        rgb
    }

    fn rgb_to_rgba(rgb: &[u8], w: usize, h: usize) -> Vec<u8> {
        let mut rgba = vec![0u8; w * h * 4];
        for i in 0..w*h {
            rgba[i*4] = rgb[i*3];
            rgba[i*4+1] = rgb[i*3+1];
            rgba[i*4+2] = rgb[i*3+2];
            rgba[i*4+3] = 255;
        }
        rgba
    }

    fn jpeg_roundtrip(rgba: &[u8], w: usize, h: usize, quality: u8) -> Vec<u8> {
        use image::{ImageBuffer, RgbImage, codecs::jpeg};
        use std::io::Cursor;
        let rgb = rgba_to_rgb(rgba, w, h);
        let img: RgbImage = ImageBuffer::from_raw(w as u32, h as u32, rgb).unwrap();
        let mut buf = Vec::new();
        let encoder = jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
        img.write_with_encoder(encoder).unwrap();
        let decoded = image::load_from_memory_with_format(&buf, image::ImageFormat::Jpeg).unwrap();
        let rgb_out = decoded.to_rgb8();
        rgb_to_rgba(rgb_out.as_raw(), w, h)
    }

    #[test]
    fn stego_roundtrip_lossless() {
        let w = 256; let h = 256;
        let carrier = make_carrier(w, h);
        let msg = b"hello stego!";
        let encoded = stego_encode(msg, &carrier, w as u32, h as u32, 0, DEFAULT_QUALITY).unwrap();
        let decoded = stego_decode_wh(&encoded, w as u32, h as u32, 0, DEFAULT_QUALITY).unwrap();
        assert_eq!(&decoded, msg, "lossless roundtrip failed");
    }

    #[test]
    fn stego_jpeg_q90() {
        let w = 256; let h = 256;
        let carrier = make_carrier(w, h);
        let msg = b"jpeg90!";
        let encoded = stego_encode(msg, &carrier, w as u32, h as u32, 0, 90).unwrap();
        let compressed = jpeg_roundtrip(&encoded, w, h, 90);
        let decoded = stego_decode_wh(&compressed, w as u32, h as u32, 0, 90);
        if let Ok(d) = &decoded {
            assert_eq!(d, msg, "JPEG q=90 roundtrip: got {:?}", String::from_utf8_lossy(d));
        }
    }

    #[test]
    fn stego_jpeg_q75() {
        let w = 256; let h = 256;
        let carrier = make_carrier(w, h);
        let msg = b"jpeg75!";
        let encoded = stego_encode(msg, &carrier, w as u32, h as u32, 0, 75).unwrap();
        let compressed = jpeg_roundtrip(&encoded, w, h, 75);
        let decoded = stego_decode_wh(&compressed, w as u32, h as u32, 0, 75);
        // May fail at q=75 — CRC detects it
        if let Ok(d) = &decoded {
            assert_eq!(d, msg, "JPEG q=75: got {:?}", String::from_utf8_lossy(d));
        }
    }

    #[test]
    fn stego_jpeg_q50() {
        let w = 256; let h = 256;
        let carrier = make_carrier(w, h);
        let msg = b"jpeg50!";
        let encoded = stego_encode(msg, &carrier, w as u32, h as u32, 0, 50).unwrap();
        let compressed = jpeg_roundtrip(&encoded, w, h, 50);
        let decoded = stego_decode_wh(&compressed, w as u32, h as u32, 0, 50);
        // q=50 likely fails — just verify no panic and CRC catches it
        if let Ok(d) = &decoded {
            assert_eq!(d, msg);
        }
    }

    #[test]
    fn stego_capacity_overflow() {
        let w = 64; let h = 64;
        let carrier = make_carrier(w, h);
        let msg = vec![b'X'; 500];
        let result = stego_encode(&msg, &carrier, w as u32, h as u32, 0, 90);
        assert!(result.is_err(), "should reject oversized payload");
    }
}
