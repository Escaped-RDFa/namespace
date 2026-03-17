// === Stego: DCT + QIM + Hamming(7,4) + CRC-16 + PRNG keying + Luma ===

// --- Hamming(7,4) systematic: [d0,d1,d2,d3,p0,p1,p2] ---
// p0=d0^d1^d3, p1=d0^d2^d3, p2=d1^d2^d3
// Syndrome→position: [none,4,5,0,6,1,2,3]

pub(crate) fn hamming_encode(data: &[u8]) -> Vec<u8> {
    let mut bits = Vec::new();
    let data_bits: Vec<u8> = data.iter()
        .flat_map(|b| (0..8).rev().map(move |i| (b >> i) & 1))
        .collect();
    for chunk in data_bits.chunks(4) {
        let d = [
            chunk.first().copied().unwrap_or(0),
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
            chunk.get(3).copied().unwrap_or(0),
        ];
        bits.extend_from_slice(&[
            d[0], d[1], d[2], d[3],
            d[0] ^ d[1] ^ d[3],
            d[0] ^ d[2] ^ d[3],
            d[1] ^ d[2] ^ d[3],
        ]);
    }
    bits
}

const SYNDROME_POS: [usize; 8] = [usize::MAX, 4, 5, 0, 6, 1, 2, 3];

pub(crate) fn hamming_decode(bits: &[u8], num_bytes: usize) -> Vec<u8> {
    let mut out = vec![0u8; num_bytes];
    let mut bit_out = 0usize;
    for chunk in bits.chunks(7) {
        if chunk.len() < 7 || bit_out / 8 >= num_bytes { break; }
        let mut r = [chunk[0], chunk[1], chunk[2], chunk[3], chunk[4], chunk[5], chunk[6]];
        let s0 = r[0] ^ r[1] ^ r[3] ^ r[4];
        let s1 = r[0] ^ r[2] ^ r[3] ^ r[5];
        let s2 = r[1] ^ r[2] ^ r[3] ^ r[6];
        let syn = (s0 as usize) | ((s1 as usize) << 1) | ((s2 as usize) << 2);
        if syn != 0 && SYNDROME_POS[syn] < 7 {
            r[SYNDROME_POS[syn]] ^= 1;
        }
        for i in 0..4 {
            if bit_out / 8 >= num_bytes { break; }
            if r[i] == 1 { out[bit_out / 8] |= 1 << (7 - (bit_out % 8)); }
            bit_out += 1;
        }
    }
    out
}

// --- CRC-16-CCITT ---
pub(crate) fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in data {
        crc ^= (b as u16) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 { (crc << 1) ^ 0x1021 } else { crc << 1 };
        }
    }
    crc
}

// --- 8-point DCT-II / IDCT ---
pub(crate) fn dct8(x: &[f32; 8]) -> [f32; 8] {
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

pub(crate) fn idct8(x: &[f32; 8]) -> [f32; 8] {
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

pub(crate) fn dct2d(block: &[[f32; 8]; 8]) -> [[f32; 8]; 8] {
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

pub(crate) fn idct2d(block: &[[f32; 8]; 8]) -> [[f32; 8]; 8] {
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

// --- JPEG luminance quantization table ---
#[rustfmt::skip]
const JPEG_LUMA_Q: [[u16; 8]; 8] = [
    [16,11,10,16,24,40,51,61],
    [12,12,14,19,26,58,60,55],
    [14,13,16,24,40,57,69,56],
    [14,17,22,29,51,87,80,62],
    [18,22,37,56,68,109,103,77],
    [24,35,55,64,81,104,113,92],
    [49,64,78,87,103,121,120,101],
    [72,92,95,98,112,100,103,99],
];

const MIN_STEP: f32 = 8.0;
const QIM_MARGIN: f32 = 2.0; // robustness margin over quantization step

pub(crate) const EMBED_POS: [(usize, usize); 4] = [(1,2), (2,1), (2,2), (3,1)];

fn jpeg_q_scale(quality: u8) -> f32 {
    if quality < 50 { 5000.0 / quality as f32 } else { (200 - 2 * quality as u16) as f32 }
}

fn adaptive_step(pos: (usize, usize), quality: u8) -> f32 {
    let base = JPEG_LUMA_Q[pos.0][pos.1] as f32;
    let scaled = (base * jpeg_q_scale(quality) / 100.0).max(1.0);
    (scaled * QIM_MARGIN).max(MIN_STEP)
}

// --- QIM with adaptive step ---
pub(crate) fn qim_embed_s(coeff: f32, bit: u8, step: f32) -> f32 {
    let q = (coeff / step).round() as i32;
    let target = if (q & 1) as u8 == bit { q } else if coeff >= q as f32 * step { q + 1 } else { q - 1 };
    target as f32 * step
}

pub(crate) fn qim_extract_s(coeff: f32, step: f32) -> u8 {
    ((coeff / step).round() as i32 & 1) as u8
}

// --- PRNG (xorshift64) ---
fn xorshift(state: &mut u64) -> u64 {
    let mut x = *state;
    if x == 0 { x = 1; }
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

fn shuffled_indices(count: usize, key: u64) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..count).collect();
    if key == 0 { return indices; } // key=0 → sequential (backward compat for testing)
    let mut rng = key;
    for i in (1..count).rev() {
        let j = (xorshift(&mut rng) as usize) % (i + 1);
        indices.swap(i, j);
    }
    indices
}

// --- RGB ↔ Luma ---
fn rgb_to_luma(r: u8, g: u8, b: u8) -> f32 {
    0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32
}

// --- Encode ---
pub(crate) fn stego_encode(data: &[u8], carrier: &[u8], width: u32, height: u32) -> Vec<u8> {
    stego_encode_keyed(data, carrier, width, height, 0, 75)
}

pub(crate) fn stego_encode_keyed(data: &[u8], carrier: &[u8], width: u32, height: u32, key: u64, jpeg_q: u8) -> Vec<u8> {
    let w = width as usize;
    let h = height as usize;
    let bx = w / 8;
    let by = h / 8;
    let num_blocks = bx * by;
    let total_code_bits = num_blocks * EMBED_POS.len();
    // Hamming(7,4): 7 code bits per 4 data bits
    let capacity = (total_code_bits / 7) * 4 / 8;

    // Payload: [CRC-16 (2 bytes)] [len (2 bytes)] [data]
    let len = data.len().min(capacity.saturating_sub(4)) as u16;
    let mut payload = Vec::with_capacity(4 + len as usize);
    let len_bytes = len.to_be_bytes();
    payload.extend_from_slice(&len_bytes);
    payload.extend_from_slice(&data[..len as usize]);
    let crc = crc16(&payload);
    let mut full = Vec::with_capacity(2 + payload.len());
    full.extend_from_slice(&crc.to_be_bytes());
    full.extend_from_slice(&payload);
    let ecc_bits = hamming_encode(&full);

    // Extract luma channel
    let mut luma = vec![0.0f32; w * h];
    for i in 0..w * h {
        let off = i * 4;
        if off + 2 < carrier.len() {
            luma[i] = rgb_to_luma(carrier[off], carrier[off + 1], carrier[off + 2]);
        } else {
            luma[i] = 128.0;
        }
    }

    // PRNG-keyed block order
    let block_order = shuffled_indices(num_blocks, key);
    // Bit scrambling order
    let bit_order = shuffled_indices(ecc_bits.len(), key.wrapping_add(1));

    let mut bit_idx = 0;
    for &blk in &block_order {
        let bx_i = blk % bx;
        let by_i = blk / bx;
        let mut block = [[0.0f32; 8]; 8];
        for r in 0..8 {
            for c in 0..8 {
                block[r][c] = luma[(by_i * 8 + r) * w + bx_i * 8 + c];
            }
        }
        let mut dct = dct2d(&block);
        for &(er, ec) in &EMBED_POS {
            if bit_idx < ecc_bits.len() {
                let step = adaptive_step((er, ec), jpeg_q);
                let scrambled = if key != 0 { bit_order[bit_idx] } else { bit_idx };
                let bit = if scrambled < ecc_bits.len() { ecc_bits[scrambled] } else { 0 };
                dct[er][ec] = qim_embed_s(dct[er][ec], bit, step);
                bit_idx += 1;
            }
        }
        let out = idct2d(&dct);
        for r in 0..8 {
            for c in 0..8 {
                luma[(by_i * 8 + r) * w + bx_i * 8 + c] = out[r][c];
            }
        }
    }

    // Apply luma delta back to RGB (distribute through green channel)
    let mut pixels = carrier.to_vec();
    if pixels.len() < w * h * 4 { pixels.resize(w * h * 4, 255); }
    for i in 0..w * h {
        let off = i * 4;
        let old_y = rgb_to_luma(pixels[off], pixels[off + 1], pixels[off + 2]);
        let delta = luma[i] - old_y;
        // Apply delta primarily to green (weight 0.587)
        let new_g = (pixels[off + 1] as f32 + delta / 0.587).round().clamp(0.0, 255.0);
        pixels[off + 1] = new_g as u8;
    }
    pixels
}

// --- Decode ---
pub(crate) fn stego_decode(pixels: &[u8]) -> Vec<u8> {
    let num_pixels = pixels.len() / 4;
    let w = (num_pixels as f64).sqrt() as usize;
    let h = num_pixels / w;
    stego_decode_wh(pixels, w, h)
}

pub(crate) fn stego_decode_wh(pixels: &[u8], w: usize, h: usize) -> Vec<u8> {
    stego_decode_keyed(pixels, w, h, 0, 75)
}

pub(crate) fn stego_decode_keyed(pixels: &[u8], w: usize, h: usize, key: u64, jpeg_q: u8) -> Vec<u8> {
    let bx = w / 8;
    let by = h / 8;
    let num_blocks = bx * by;

    // Extract luma
    let mut luma = vec![0.0f32; w * h];
    for i in 0..w * h {
        let off = i * 4;
        if off + 2 < pixels.len() {
            luma[i] = rgb_to_luma(pixels[off], pixels[off + 1], pixels[off + 2]);
        }
    }

    // PRNG-keyed block order (same as encode)
    let block_order = shuffled_indices(num_blocks, key);

    let mut raw_bits = Vec::new();
    for &blk in &block_order {
        let bx_i = blk % bx;
        let by_i = blk / bx;
        let mut block = [[0.0f32; 8]; 8];
        for r in 0..8 {
            for c in 0..8 {
                block[r][c] = luma[(by_i * 8 + r) * w + bx_i * 8 + c];
            }
        }
        let dct = dct2d(&block);
        for &(er, ec) in &EMBED_POS {
            let step = adaptive_step((er, ec), jpeg_q);
            raw_bits.push(qim_extract_s(dct[er][ec], step));
        }
    }

    // Unscramble bits
    let mut bits = vec![0u8; raw_bits.len()];
    if key != 0 {
        let bit_order = shuffled_indices(raw_bits.len(), key.wrapping_add(1));
        for (i, &scrambled) in bit_order.iter().enumerate() {
            if scrambled < bits.len() && i < raw_bits.len() {
                bits[scrambled] = raw_bits[i];
            }
        }
    } else {
        bits = raw_bits;
    }

    // Decode: [CRC-16 (2)] [len (2)] [data]
    let header_bits = 4 * 7 * 8 / 4; // 4 bytes header → 32 data bits → 56 code bits
    if bits.len() < 56 { return Vec::new(); }
    let header = hamming_decode(&bits, 4);
    let crc_got = u16::from_be_bytes([header[0], header[1]]);
    let len = u16::from_be_bytes([header[2], header[3]]) as usize;
    if len > 100_000 || bits.len() < (4 + len) * 2 * 7 { return Vec::new(); }

    let payload_with_len = hamming_decode(&bits[56..], len + 2);
    // Verify: payload = [len (2)] [data (len)]
    let mut check = Vec::with_capacity(2 + len);
    check.extend_from_slice(&header[2..4]); // len bytes
    check.extend_from_slice(&payload_with_len[2..2 + len.min(payload_with_len.len().saturating_sub(2))]);

    // Actually, re-decode the full thing at once for CRC check
    let full_len = 4 + len;
    let full = hamming_decode(&bits, full_len);
    let crc_stored = u16::from_be_bytes([full[0], full[1]]);
    let crc_check = crc16(&full[2..]);
    if crc_stored != crc_check { return Vec::new(); }

    full[4..4 + len].to_vec()
}

// Legacy compat constants (used by lib.rs status messages)
pub(crate) const STEGO_REDUNDANCY: usize = 7; // Hamming(7,4) codeword length
