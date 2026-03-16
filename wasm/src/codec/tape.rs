// === Tape / Kansas City Standard FSK ===
// 1200 Hz = 0, 2400 Hz = 1. 300 baud. 8N1 framing.

pub(crate) const SAMPLE_RATE: u32 = 44100;
pub(crate) const BAUD: u32 = 300;
pub(crate) const FREQ_ZERO: f32 = 1200.0;
pub(crate) const FREQ_ONE: f32 = 2400.0;
pub(crate) const SAMPLES_PER_BIT: u32 = SAMPLE_RATE / BAUD;

pub(crate) fn fsk_bit(bit: bool, out: &mut Vec<i16>) {
    let f = if bit { FREQ_ONE } else { FREQ_ZERO };
    for i in 0..SAMPLES_PER_BIT {
        let t = i as f32 / SAMPLE_RATE as f32;
        out.push(((2.0 * core::f32::consts::PI * f * t).sin() * 24000.0) as i16);
    }
}

pub(crate) fn fsk_byte(byte: u8, out: &mut Vec<i16>) {
    fsk_bit(false, out); // start
    for i in 0..8 { fsk_bit((byte >> i) & 1 == 1, out); }
    fsk_bit(true, out); // stop
}

pub(crate) fn tape_encode(data: &[u8]) -> Vec<i16> {
    let mut s = Vec::new();
    for _ in 0..(SAMPLE_RATE / SAMPLES_PER_BIT) { fsk_bit(true, &mut s); } // 1s leader
    for &b in &(data.len() as u32).to_be_bytes() { fsk_byte(b, &mut s); }
    for &b in data { fsk_byte(b, &mut s); }
    for _ in 0..(SAMPLE_RATE / SAMPLES_PER_BIT / 2) { fsk_bit(true, &mut s); } // trailer
    s
}

pub(crate) fn tape_decode(samples: &[i16]) -> Option<Vec<u8>> {
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

pub(crate) fn samples_to_wav(samples: &[i16]) -> Vec<u8> {
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

pub(crate) fn wav_to_samples(wav: &[u8]) -> Option<Vec<i16>> {
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

