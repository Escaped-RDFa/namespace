// === Stego: DCT + QIM + Hamming(7,4) + CRC-16 + PRNG keying + Luma ===
//
// Design:
// - PNG-out, pixel-domain 8x8 DCT embedding
// - QIM on mid-frequency coefficients
// - Hamming(7,4) ECC
// - CRC-16 integrity
// - keyed PRNG block permutation
// - explicit width/height required for reliable decode
//
// Channel limits:
// - survives mild JPEG recompression better than plain LSB
// - crop / resize / heavy filtering can break alignment

// ---------------------------
// Configuration
// ---------------------------

pub const EMBED_POS: [(usize, usize); 4] = [(1, 2), (2, 1), (2, 2), (3, 1)];
pub const MAGIC: [u8; 2] = *b"SG";
pub const VERSION: u8 = 2;
pub const DEFAULT_QUALITY: u8 = 90;

// Header: [magic:2][version:1][crc16:2][len:2] = 7 bytes
const HEADER_LEN: usize = 7;
// 1 byte -> 2 Hamming codewords -> 14 bits
const CODE_BITS_PER_BYTE: usize = 14;

// ---------------------------
// Errors
// ---------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StegoError {
    InvalidDimensions,
    CarrierTooSmall,
    PayloadTooLarge,
    DecodeFailed,
    IntegrityFailed,
    BadHeader,
}

// ---------------------------
// CRC16-CCITT
// ---------------------------

pub fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in data {
        crc ^= (b as u16) << 8;
        for _ in 0..8 {
            crc = if (crc & 0x8000) != 0 { (crc << 1) ^ 0x1021 } else { crc << 1 };
        }
    }
    crc
}

// ---------------------------
// Hamming(7,4)
// systematic: [d0,d1,d2,d3,p0,p1,p2]
// p0=d0^d1^d3, p1=d0^d2^d3, p2=d1^d2^d3
// ---------------------------

fn hamming74_encode_nibble(n: u8) -> [u8; 7] {
    let d0 = (n >> 3) & 1;
    let d1 = (n >> 2) & 1;
    let d2 = (n >> 1) & 1;
    let d3 = n & 1;
    [d0, d1, d2, d3, d0 ^ d1 ^ d3, d0 ^ d2 ^ d3, d1 ^ d2 ^ d3]
}

fn hamming74_decode_codeword(mut c: [u8; 7]) -> u8 {
    let s0 = c[4] ^ c[0] ^ c[1] ^ c[3];
    let s1 = c[5] ^ c[0] ^ c[2] ^ c[3];
    let s2 = c[6] ^ c[1] ^ c[2] ^ c[3];
    let syndrome = (s0 << 2) | (s1 << 1) | s2;
    let flip = match syndrome {
        0b110 => Some(0), 0b101 => Some(1), 0b011 => Some(2), 0b111 => Some(3),
        0b100 => Some(4), 0b010 => Some(5), 0b001 => Some(6), _ => None,
    };
    if let Some(i) = flip { c[i] ^= 1; }
    (c[0] << 3) | (c[1] << 2) | (c[2] << 1) | c[3]
}

pub fn hamming_encode(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() * CODE_BITS_PER_BYTE);
    for &b in data {
        out.extend_from_slice(&hamming74_encode_nibble(b >> 4));
        out.extend_from_slice(&hamming74_encode_nibble(b & 0x0F));
    }
    out
}

pub fn hamming_decode(bits: &[u8], num_bytes: usize) -> Vec<u8> {
    let needed = num_bytes * CODE_BITS_PER_BYTE;
    if bits.len() < needed { return Vec::new(); }
    let mut out = Vec::with_capacity(num_bytes);
    for i in 0..num_bytes {
        let base = i * CODE_BITS_PER_BYTE;
        let hi = hamming74_decode_codeword([
            bits[base], bits[base+1], bits[base+2], bits[base+3],
            bits[base+4], bits[base+5], bits[base+6],
        ]);
        let lo = hamming74_decode_codeword([
            bits[base+7], bits[base+8], bits[base+9], bits[base+10],
            bits[base+11], bits[base+12], bits[base+13],
        ]);
        out.push((hi << 4) | lo);
    }
    out
}

// ---------------------------
// XorShift64 PRNG
// ---------------------------

#[derive(Clone)]
struct XorShift64 { state: u64 }

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self { state: if seed == 0 { 0x9E3779B97F4A7C15 } else { seed } }
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13; x ^= x >> 7; x ^= x << 17;
        self.state = x; x
    }
    fn gen_range(&mut self, upper: usize) -> usize {
        if upper <= 1 { 0 } else { (self.next_u64() as usize) % upper }
    }
}

fn shuffled_block_order(num_blocks: usize, key: u64) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..num_blocks).collect();
    let mut rng = XorShift64::new(key);
    for i in (1..idx.len()).rev() {
        let j = rng.gen_range(i + 1);
        idx.swap(i, j);
    }
    idx
}

// ---------------------------
// DCT-II / IDCT (8-point)
// ---------------------------

pub fn dct8(x: &[f32; 8]) -> [f32; 8] {
    let mut out = [0.0f32; 8];
    for k in 0..8 {
        let s = if k == 0 { 0.5_f32.sqrt() } else { 1.0 };
        let mut sum = 0.0f32;
        for n in 0..8 {
            sum += x[n] * (core::f32::consts::PI * (2 * n + 1) as f32 * k as f32 / 16.0).cos();
        }
        out[k] = sum * s * 0.5;
    }
    out
}

pub fn idct8(x: &[f32; 8]) -> [f32; 8] {
    let mut out = [0.0f32; 8];
    for n in 0..8 {
        let mut sum = 0.0f32;
        for k in 0..8 {
            let s = if k == 0 { 0.5_f32.sqrt() } else { 1.0 };
            sum += s * x[k] * (core::f32::consts::PI * (2 * n + 1) as f32 * k as f32 / 16.0).cos();
        }
        out[n] = sum * 0.5;
    }
    out
}

pub fn dct2d(block: &[[f32; 8]; 8]) -> [[f32; 8]; 8] {
    let mut tmp = [[0.0f32; 8]; 8];
    for i in 0..8 { tmp[i] = dct8(&block[i]); }
    let mut out = [[0.0f32; 8]; 8];
    for j in 0..8 {
        let col: [f32; 8] = core::array::from_fn(|i| tmp[i][j]);
        let dc = dct8(&col);
        for i in 0..8 { out[i][j] = dc[i]; }
    }
    out
}

pub fn idct2d(block: &[[f32; 8]; 8]) -> [[f32; 8]; 8] {
    let mut tmp = [[0.0f32; 8]; 8];
    for i in 0..8 { tmp[i] = idct8(&block[i]); }
    let mut out = [[0.0f32; 8]; 8];
    for j in 0..8 {
        let col: [f32; 8] = core::array::from_fn(|i| tmp[i][j]);
        let dc = idct8(&col);
        for i in 0..8 { out[i][j] = dc[i]; }
    }
    out
}

// ---------------------------
// Luma helpers
// ---------------------------

#[inline]
fn rgb_to_luma(r: u8, g: u8, b: u8) -> f32 {
    0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32
}

fn extract_luma(carrier: &[u8], w: usize, h: usize) -> Vec<f32> {
    let mut y = vec![0.0f32; w * h];
    for i in 0..w * h {
        let off = i * 4;
        y[i] = rgb_to_luma(
            carrier.get(off).copied().unwrap_or(128),
            carrier.get(off + 1).copied().unwrap_or(128),
            carrier.get(off + 2).copied().unwrap_or(128),
        );
    }
    y
}

fn apply_luma_back(carrier: &[u8], luma: &[f32], w: usize, h: usize) -> Vec<u8> {
    let mut pixels = carrier.to_vec();
    if pixels.len() < w * h * 4 { pixels.resize(w * h * 4, 255); }
    for i in 0..w * h {
        let off = i * 4;
        let old_y = rgb_to_luma(pixels[off], pixels[off + 1], pixels[off + 2]);
        let delta = luma[i] - old_y;
        let g_raw = pixels[off + 1] as f32 + delta / 0.587;
        let g_new = g_raw.round().clamp(0.0, 255.0);
        let g_remainder_y = (g_raw - g_new) * 0.587;
        pixels[off + 1] = g_new as u8;
        if g_remainder_y.abs() > 0.25 {
            let denom = 0.299 + 0.114;
            pixels[off] = (pixels[off] as f32 + g_remainder_y * (0.299 / denom) / 0.299)
                .round().clamp(0.0, 255.0) as u8;
            pixels[off + 2] = (pixels[off + 2] as f32 + g_remainder_y * (0.114 / denom) / 0.114)
                .round().clamp(0.0, 255.0) as u8;
        }
    }
    pixels
}

// ---------------------------
// QIM
// ---------------------------

fn jpeg_q_scale(quality: u8) -> f32 {
    match quality {
        0 => 5000.0,
        1..=49 => 5000.0 / quality as f32,
        50..=99 => (200 - 2 * quality as u16) as f32,
        _ => 1.0,
    }
}

fn adaptive_step(pos: (usize, usize), quality: u8) -> f32 {
    let base = match pos {
        (1, 2) | (2, 1) => 18.0,
        (2, 2) => 22.0,
        (3, 1) => 24.0,
        _ => 20.0,
    };
    let q = jpeg_q_scale(quality);
    (base * (q / 20.0).max(1.0)).clamp(12.0, 48.0)
}

pub fn qim_embed_s(coeff: f32, bit: u8, step: f32) -> f32 {
    let q = (coeff / step).round() as i32;
    if ((q & 1) as u8) == bit { return q as f32 * step; }
    let up = q + 1;
    let dn = q - 1;
    let up_ok = ((up & 1) as u8) == bit;
    let dn_ok = ((dn & 1) as u8) == bit;
    match (dn_ok, up_ok) {
        (true, true) => {
            if (coeff - dn as f32 * step).abs() <= (coeff - up as f32 * step).abs()
                { dn as f32 * step } else { up as f32 * step }
        }
        (true, false) => dn as f32 * step,
        (false, true) => up as f32 * step,
        _ => q as f32 * step,
    }
}

pub fn qim_extract_s(coeff: f32, step: f32) -> u8 {
    ((coeff / step).round() as i32 & 1) as u8
}

// ---------------------------
// Framing
// ---------------------------

fn build_framed_payload(data: &[u8]) -> Result<Vec<u8>, StegoError> {
    if data.len() > u16::MAX as usize { return Err(StegoError::PayloadTooLarge); }
    let crc = crc16(data);
    let len = data.len() as u16;
    let mut out = Vec::with_capacity(HEADER_LEN + data.len());
    out.extend_from_slice(&MAGIC);
    out.push(VERSION);
    out.extend_from_slice(&crc.to_be_bytes());
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(data);
    Ok(out)
}

fn parse_framed_payload(full: &[u8]) -> Result<Vec<u8>, StegoError> {
    if full.len() < HEADER_LEN { return Err(StegoError::BadHeader); }
    if full[0..2] != MAGIC || full[2] != VERSION { return Err(StegoError::BadHeader); }
    let crc_stored = u16::from_be_bytes([full[3], full[4]]);
    let len = u16::from_be_bytes([full[5], full[6]]) as usize;
    if full.len() < HEADER_LEN + len { return Err(StegoError::DecodeFailed); }
    let payload = &full[HEADER_LEN..HEADER_LEN + len];
    if crc_stored != crc16(payload) { return Err(StegoError::IntegrityFailed); }
    Ok(payload.to_vec())
}

// ---------------------------
// Capacity
// ---------------------------

pub fn stego_capacity_bytes(width: u32, height: u32) -> usize {
    let bx = width as usize / 8;
    let by = height as usize / 8;
    let raw_bits = bx * by * EMBED_POS.len();
    (raw_bits / CODE_BITS_PER_BYTE).saturating_sub(HEADER_LEN)
}

fn checked_dims(width: u32, height: u32, carrier_len: usize) -> Result<(usize, usize), StegoError> {
    let w = width as usize;
    let h = height as usize;
    if w < 8 || h < 8 { return Err(StegoError::InvalidDimensions); }
    if carrier_len < w * h * 4 { return Err(StegoError::CarrierTooSmall); }
    Ok((w, h))
}

// ---------------------------
// Encode
// ---------------------------

pub fn stego_encode(
    data: &[u8], carrier: &[u8], width: u32, height: u32, key: u64, jpeg_quality_hint: u8,
) -> Result<Vec<u8>, StegoError> {
    let (w, h) = checked_dims(width, height, carrier.len())?;
    let framed = build_framed_payload(data)?;
    let ecc_bits = hamming_encode(&framed);
    let bx = w / 8;
    let by = h / 8;
    let num_blocks = bx * by;
    if ecc_bits.len() > num_blocks * EMBED_POS.len() { return Err(StegoError::PayloadTooLarge); }

    let order = shuffled_block_order(num_blocks, key);
    let mut luma = extract_luma(carrier, w, h);
    let mut bit_idx = 0usize;

    for &blk in &order {
        if bit_idx >= ecc_bits.len() { break; }
        let by_i = blk / bx;
        let bx_i = blk % bx;
        let mut block = [[0.0f32; 8]; 8];
        for r in 0..8 { for c in 0..8 {
            block[r][c] = luma[(by_i * 8 + r) * w + bx_i * 8 + c] - 128.0;
        }}
        let mut dct = dct2d(&block);
        for &pos in &EMBED_POS {
            if bit_idx >= ecc_bits.len() { break; }
            let step = adaptive_step(pos, jpeg_quality_hint);
            dct[pos.0][pos.1] = qim_embed_s(dct[pos.0][pos.1], ecc_bits[bit_idx], step);
            bit_idx += 1;
        }
        let out = idct2d(&dct);
        for r in 0..8 { for c in 0..8 {
            luma[(by_i * 8 + r) * w + bx_i * 8 + c] = (out[r][c] + 128.0).clamp(0.0, 255.0);
        }}
    }

    Ok(apply_luma_back(carrier, &luma, w, h))
}

// ---------------------------
// Decode
// ---------------------------

pub fn stego_decode_wh(
    pixels: &[u8], width: u32, height: u32, key: u64, jpeg_quality_hint: u8,
) -> Result<Vec<u8>, StegoError> {
    let (w, h) = checked_dims(width, height, pixels.len())?;
    let bx = w / 8;
    let by = h / 8;
    let num_blocks = bx * by;
    let order = shuffled_block_order(num_blocks, key);
    let luma = extract_luma(pixels, w, h);
    let mut bits = Vec::with_capacity(num_blocks * EMBED_POS.len());

    for &blk in &order {
        let by_i = blk / bx;
        let bx_i = blk % bx;
        let mut block = [[0.0f32; 8]; 8];
        for r in 0..8 { for c in 0..8 {
            block[r][c] = luma[(by_i * 8 + r) * w + bx_i * 8 + c] - 128.0;
        }}
        let dct = dct2d(&block);
        for &pos in &EMBED_POS {
            let step = adaptive_step(pos, jpeg_quality_hint);
            bits.push(qim_extract_s(dct[pos.0][pos.1], step));
        }
    }

    // Decode header first to get payload length
    let hdr_bits = HEADER_LEN * CODE_BITS_PER_BYTE;
    if bits.len() < hdr_bits { return Err(StegoError::DecodeFailed); }
    let header = hamming_decode(&bits[..hdr_bits], HEADER_LEN);
    if header.len() != HEADER_LEN { return Err(StegoError::DecodeFailed); }
    if header[0..2] != MAGIC || header[2] != VERSION { return Err(StegoError::BadHeader); }

    let payload_len = u16::from_be_bytes([header[5], header[6]]) as usize;
    let full_len = HEADER_LEN + payload_len;
    let full_bits = full_len * CODE_BITS_PER_BYTE;
    if bits.len() < full_bits { return Err(StegoError::DecodeFailed); }

    let full = hamming_decode(&bits[..full_bits], full_len);
    if full.len() != full_len { return Err(StegoError::DecodeFailed); }
    parse_framed_payload(&full)
}

/// Legacy wrapper — returns empty Vec on any error
pub fn stego_decode(pixels: &[u8], width: u32, height: u32, key: u64) -> Vec<u8> {
    stego_decode_wh(pixels, width, height, key, DEFAULT_QUALITY).unwrap_or_default()
}

// Legacy compat
pub const STEGO_REDUNDANCY: usize = 7;

// ---------------------------
// Tests
// ---------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dct_roundtrip_constant() {
        let block = [[32.0f32; 8]; 8];
        let r = idct2d(&dct2d(&block));
        for y in 0..8 { for x in 0..8 {
            assert!((r[y][x] - 32.0).abs() < 1e-3, "({},{}) = {}", y, x, r[y][x]);
        }}
    }

    #[test]
    fn dct_roundtrip_gradient() {
        let block: [[f32; 8]; 8] = core::array::from_fn(|i|
            core::array::from_fn(|j| (i * 8 + j) as f32));
        let r = idct2d(&dct2d(&block));
        for i in 0..8 { for j in 0..8 {
            assert!((block[i][j] - r[i][j]).abs() < 0.01, "({},{}) {} vs {}", i, j, block[i][j], r[i][j]);
        }}
    }

    #[test]
    fn hamming_roundtrip() {
        let data = b"hello world";
        assert_eq!(hamming_decode(&hamming_encode(data), data.len()), data);
    }

    #[test]
    fn hamming_corrects_single_bit() {
        let data = b"\xA5";
        let mut bits = hamming_encode(data);
        for i in (0..bits.len()).step_by(7) { bits[i] ^= 1; }
        assert_eq!(hamming_decode(&bits, 1), data);
    }

    #[test]
    fn framing_roundtrip() {
        let p = b"abc123";
        assert_eq!(parse_framed_payload(&build_framed_payload(p).unwrap()).unwrap(), p);
    }

    #[test]
    fn framing_detects_corruption() {
        let mut f = build_framed_payload(b"test").unwrap();
        f[HEADER_LEN] ^= 0xFF; // corrupt first data byte
        assert_eq!(parse_framed_payload(&f), Err(StegoError::IntegrityFailed));
    }

    #[test]
    fn qim_roundtrip() {
        for step in [12.0, 18.0, 30.0, 48.0] {
            for bit in [0u8, 1] {
                for coeff in [-100.0, -30.0, 0.0, 15.5, 30.0, 100.0, 255.0] {
                    let e = qim_embed_s(coeff, bit, step);
                    assert_eq!(qim_extract_s(e, step), bit,
                        "coeff={} bit={} step={} embedded={}", coeff, bit, step, e);
                }
            }
        }
    }

    fn make_carrier(w: usize, h: usize) -> Vec<u8> {
        let mut px = vec![0u8; w * h * 4];
        for y in 0..h { for x in 0..w {
            let i = (y * w + x) * 4;
            px[i] = (x & 255) as u8;
            px[i + 1] = (y & 255) as u8;
            px[i + 2] = 128;
            px[i + 3] = 255;
        }}
        px
    }

    #[test]
    fn stego_roundtrip_256() {
        let c = make_carrier(256, 256);
        let msg = b"Hello stego roundtrip!";
        let enc = stego_encode(msg, &c, 256, 256, 0x1234, 90).unwrap();
        let dec = stego_decode_wh(&enc, 256, 256, 0x1234, 90).unwrap();
        assert_eq!(dec, msg);
    }

    #[test]
    fn stego_roundtrip_nonsquare() {
        let c = make_carrier(256, 128);
        let msg = b"wide image";
        let enc = stego_encode(msg, &c, 256, 128, 42, 90).unwrap();
        let dec = stego_decode_wh(&enc, 256, 128, 42, 90).unwrap();
        assert_eq!(dec, msg);
    }

    #[test]
    fn stego_wrong_key_fails() {
        let c = make_carrier(256, 256);
        let enc = stego_encode(b"secret", &c, 256, 256, 42, 90).unwrap();
        let dec = stego_decode_wh(&enc, 256, 256, 99, 90);
        assert!(dec.is_err(), "Wrong key should fail");
    }

    #[test]
    fn stego_payload_too_large() {
        let c = make_carrier(64, 64);
        let big = vec![b'X'; 500];
        assert_eq!(stego_encode(&big, &c, 64, 64, 0, 90), Err(StegoError::PayloadTooLarge));
    }

    #[test]
    fn stego_capacity() {
        // 256×256: 32×32=1024 blocks × 4 bits = 4096 code bits / 14 = 292 bytes - 7 header = 285
        let cap = stego_capacity_bytes(256, 256);
        assert!(cap > 200, "capacity should be >200, got {}", cap);
    }

    #[test]
    fn stego_corruption_detected() {
        let c = make_carrier(256, 256);
        let mut enc = stego_encode(b"integrity", &c, 256, 256, 0, 90).unwrap();
        for i in 0..2000 { enc[i * 4 + 1] = 0; }
        let dec = stego_decode_wh(&enc, 256, 256, 0, 90);
        assert!(dec.is_err(), "Corruption should be detected");
    }
}
