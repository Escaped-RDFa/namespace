use crate::codec::tape::{SAMPLE_RATE, samples_to_wav, wav_to_samples};
// === Numbers Station ===
// Encode data as hex digit groups, generate distinct tone per digit (0-F).
// Each digit = unique frequency, spoken in groups of 5 with pauses.
// Decimal encoding: each byte → 3 decimal digits (000-255), only 0-9 used.

pub(crate) const NUM_FREQS: [f32; 10] = [
    330.0, 370.0, 415.0, 466.0, 523.0, 587.0, 659.0, 740.0, 831.0, 932.0,
];
pub(crate) const DIGIT_SAMPLES: u32 = SAMPLE_RATE / 3; // 333ms per digit
pub(crate) const PAUSE_SAMPLES: u32 = SAMPLE_RATE / 6; // 166ms pause
pub(crate) const GROUP_PAUSE: u32 = SAMPLE_RATE / 2;   // 500ms between groups

/// Encode bytes as decimal digit string: each byte → 3 digits (000-255)
pub(crate) fn bytes_to_decimal(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:03}", b)).collect()
}

/// Decode decimal digit string back to bytes
pub(crate) fn decimal_to_bytes(digits: &str) -> Option<Vec<u8>> {
    if digits.len() % 3 != 0 { return None; }
    digits.as_bytes().chunks(3).map(|c| {
        std::str::from_utf8(c).ok()?.parse::<u16>().ok().filter(|&v| v <= 255).map(|v| v as u8)
    }).collect()
}

pub(crate) fn numbers_station_encode(data: &[u8]) -> Vec<i16> {
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

pub(crate) fn numbers_station_decode(samples: &[i16]) -> Option<Vec<u8>> {
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

pub(crate) fn pcm_to_i16(raw: &[u8]) -> Vec<i16> {
    raw.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect()
}

pub(crate) fn numbers_speech_synth(data: &[u8]) -> Vec<i16> {
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


pub(crate) fn load_digit(digit: u8, raw: &[u8]) {
    if digit > 9 { return; }
    VOICE_SAMPLES.with(|ds| ds.borrow_mut()[digit as usize] = pcm_to_i16(raw));
}
