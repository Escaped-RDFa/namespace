// === BBS / FSK audio tones ===
// Encode bytes as hex, each hex digit maps to a DTMF-like frequency pair.
// Low group: 697, 770, 852, 941 Hz  High group: 1209, 1336, 1477, 1633 Hz

pub(crate) const DTMF_LOW: [f32; 4] = [697.0, 770.0, 852.0, 941.0];
pub(crate) const DTMF_HIGH: [f32; 4] = [1209.0, 1336.0, 1477.0, 1633.0];

pub(crate) fn hex_digit_freqs(d: u8) -> (f32, f32) {
    let row = (d >> 2) & 0x3;
    let col = d & 0x3;
    (DTMF_LOW[row as usize], DTMF_HIGH[col as usize])
}

