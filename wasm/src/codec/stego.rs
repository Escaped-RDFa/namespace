// === Stego (LSB in RGBA pixels) ===

pub(crate) const STEGO_REDUNDANCY: usize = 7;
pub(crate) const QIM_STEP: f32 = 30.0; // default step; users can tune via ALife search

pub(crate) fn ecc_encode(data: &[u8]) -> Vec<u8> {
    let mut bits = Vec::with_capacity(data.len() * 8 * STEGO_REDUNDANCY);
    for byte in data {
        for b in (0..8).rev() {
            let bit = (byte >> b) & 1;
            for _ in 0..STEGO_REDUNDANCY { bits.push(bit); }
        }
    }
    bits
}

pub(crate) fn ecc_decode(bits: &[u8], num_bytes: usize) -> Vec<u8> {
    let mut out = vec![0u8; num_bytes];
    for i in 0..num_bytes * 8 {
        let start = i * STEGO_REDUNDANCY;
        if start + STEGO_REDUNDANCY > bits.len() { break; }
        let ones: usize = bits[start..start + STEGO_REDUNDANCY].iter().map(|&b| b as usize).sum();
        if ones > STEGO_REDUNDANCY / 2 {
            out[i / 8] |= 1 << (7 - (i % 8));
        }
    }
    out
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

// Mid-frequency zigzag positions for embedding (survive JPEG quantization)
pub(crate) const EMBED_POS: [(usize, usize); 4] = [(1,2), (2,1), (2,2), (3,1)];

pub(crate) fn qim_embed(coeff: f32, bit: u8) -> f32 { qim_embed_s(coeff, bit, QIM_STEP) }
pub(crate) fn qim_extract(coeff: f32) -> u8 { qim_extract_s(coeff, QIM_STEP) }
pub(crate) fn qim_embed_s(coeff: f32, bit: u8, step: f32) -> f32 {
    let q = (coeff / step).round() as i32;
    let target = if (q & 1) as u8 == bit { q } else if coeff >= q as f32 * step { q + 1 } else { q - 1 };
    target as f32 * step
}
pub(crate) fn qim_extract_s(coeff: f32, step: f32) -> u8 {
    ((coeff / step).round() as i32 & 1) as u8
}

pub(crate) fn stego_encode(data: &[u8], carrier: &[u8], width: u32, height: u32) -> Vec<u8> {
    let w = width as usize;
    let h = height as usize;
    let bx = w / 8;
    let by = h / 8;
    let bits_per_block = EMBED_POS.len();
    let capacity = (bx * by * bits_per_block) / STEGO_REDUNDANCY / 8;

    let mut payload = Vec::new();
    let len = data.len().min(capacity.saturating_sub(4)) as u32;
    payload.extend_from_slice(&len.to_be_bytes());
    payload.extend_from_slice(&data[..len as usize]);
    let ecc_bits = ecc_encode(&payload);

    // Work on green channel
    let mut green = vec![0.0f32; w * h];
    for i in 0..w * h {
        green[i] = if i * 4 + 1 < carrier.len() { carrier[i * 4 + 1] as f32 } else { 128.0 };
    }

    let mut bit_idx = 0;
    for by_i in 0..by {
        for bx_i in 0..bx {
            let mut block = [[0.0f32; 8]; 8];
            for r in 0..8 {
                for c in 0..8 {
                    block[r][c] = green[(by_i * 8 + r) * w + bx_i * 8 + c];
                }
            }
            let mut dct = dct2d(&block);
            for &(er, ec) in &EMBED_POS {
                if bit_idx < ecc_bits.len() {
                    dct[er][ec] = qim_embed(dct[er][ec], ecc_bits[bit_idx]);
                    bit_idx += 1;
                }
            }
            let out = idct2d(&dct);
            for r in 0..8 {
                for c in 0..8 {
                    green[(by_i * 8 + r) * w + bx_i * 8 + c] = out[r][c];
                }
            }
        }
    }

    // Write back green channel, clamp
    let mut pixels = carrier.to_vec();
    if pixels.len() < w * h * 4 { pixels.resize(w * h * 4, 255); }
    for i in 0..w * h {
        pixels[i * 4 + 1] = green[i].round().clamp(0.0, 255.0) as u8;
    }
    pixels
}

pub(crate) fn stego_decode(pixels: &[u8]) -> Vec<u8> {
    let num_pixels = pixels.len() / 4;
    // Guess square-ish dimensions
    let w = (num_pixels as f64).sqrt() as usize;
    let h = num_pixels / w;
    stego_decode_wh(pixels, w, h)
}

pub(crate) fn stego_decode_wh(pixels: &[u8], w: usize, h: usize) -> Vec<u8> {
    let bx = w / 8;
    let by = h / 8;

    let mut green = vec![0.0f32; w * h];
    for i in 0..w * h {
        green[i] = pixels[i * 4 + 1] as f32;
    }

    let mut bits = Vec::new();
    for by_i in 0..by {
        for bx_i in 0..bx {
            let mut block = [[0.0f32; 8]; 8];
            for r in 0..8 {
                for c in 0..8 {
                    block[r][c] = green[(by_i * 8 + r) * w + bx_i * 8 + c];
                }
            }
            let dct = dct2d(&block);
            for &(er, ec) in &EMBED_POS {
                bits.push(qim_extract(dct[er][ec]));
            }
        }
    }

    if bits.len() < 32 * STEGO_REDUNDANCY { return Vec::new(); }
    let len_bytes = ecc_decode(&bits, 4);
    let len = u32::from_be_bytes([len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3]]) as usize;
    if len > 100_000 || bits.len() < (4 + len) * 8 * STEGO_REDUNDANCY { return Vec::new(); }
    ecc_decode(&bits[32 * STEGO_REDUNDANCY..], len)
}

