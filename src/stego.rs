//! Hostile Media Embedding (HME) Steganographic System
//! 
//! Embed structured data in hostile environments that strip metadata

/// Hostility level of environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HostilityLevel {
    Friendly = 0,        // No sanitization
    Cautious = 1,        // Basic sanitization
    Restrictive = 2,     // Whitelist-based
    Aggressive = 3,      // Blacklist + Whitelist
    Paranoid = 4,        // Complete rewrite
    MaximumHostile = 5,  // Text-only
}

/// Steganographic encoding strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StegoStrategy {
    HtmlEscape,      // Standard HTML entity escaping
    CommentEmbed,    // Hide in HTML comments
    HiddenDiv,       // Display:none div
    DataAttribute,   // data-* attributes
    Whitespace,      // Extra whitespace encoding
    ZeroWidth,       // Zero-width characters
    Unicode,         // Homoglyph substitution
    MultiLayer,      // Multiple encoding layers
    Position,        // Element positioning (x,y coordinates)
    Ordering,        // DOM element order
    Color,           // RGB/HSL color values
    CssProperty,     // CSS property values
    FontSize,        // Font size variations
    Bitmap,          // Image pixel data
    QrCode,          // QR code encoding
    VisualNoise,     // Imperceptible visual variations
}

/// Steganographic encoder trait
pub trait StegoEncoder {
    fn encode(&self, data: &str, strategy: StegoStrategy) -> String;
    fn decode(&self, encoded: &str, strategy: StegoStrategy) -> Option<String>;
    fn max_hostility(&self, strategy: StegoStrategy) -> HostilityLevel;
}

/// eRDFa steganographic system
pub struct ERdfaStego;

impl StegoEncoder for ERdfaStego {
    fn encode(&self, data: &str, strategy: StegoStrategy) -> String {
        match strategy {
            StegoStrategy::HtmlEscape => escape_html(data),
            StegoStrategy::CommentEmbed => format!("<!-- {} -->", escape_html(data)),
            StegoStrategy::HiddenDiv => format!(r#"<div style="display:none">{}</div>"#, escape_html(data)),
            StegoStrategy::DataAttribute => format!(r#"<div data-erdfa="{}">"#, escape_html(data)),
            StegoStrategy::Whitespace => encode_whitespace(data),
            StegoStrategy::ZeroWidth => encode_zero_width(data),
            StegoStrategy::Unicode => encode_unicode(data),
            StegoStrategy::MultiLayer => encode_multi_layer(data),
        }
    }
    
    fn decode(&self, encoded: &str, strategy: StegoStrategy) -> Option<String> {
        match strategy {
            StegoStrategy::HtmlEscape => Some(unescape_html(encoded)),
            StegoStrategy::CommentEmbed => extract_from_comment(encoded),
            StegoStrategy::HiddenDiv => extract_from_hidden_div(encoded),
            StegoStrategy::DataAttribute => extract_from_data_attr(encoded),
            StegoStrategy::Whitespace => decode_whitespace(encoded),
            StegoStrategy::ZeroWidth => decode_zero_width(encoded),
            StegoStrategy::Unicode => decode_unicode(encoded),
            StegoStrategy::MultiLayer => decode_multi_layer(encoded),
        }
    }
    
    fn max_hostility(&self, strategy: StegoStrategy) -> HostilityLevel {
        match strategy {
            StegoStrategy::HtmlEscape => HostilityLevel::Aggressive,
            StegoStrategy::CommentEmbed => HostilityLevel::Aggressive,
            StegoStrategy::HiddenDiv => HostilityLevel::Restrictive,
            StegoStrategy::DataAttribute => HostilityLevel::Restrictive,
            StegoStrategy::Whitespace => HostilityLevel::Paranoid,
            StegoStrategy::ZeroWidth => HostilityLevel::MaximumHostile,
            StegoStrategy::Unicode => HostilityLevel::MaximumHostile,
            StegoStrategy::MultiLayer => HostilityLevel::MaximumHostile,
        }
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}

fn unescape_html(s: &str) -> String {
    s.replace("&lt;", "<")
     .replace("&gt;", ">")
     .replace("&quot;", "\"")
     .replace("&amp;", "&")
}

fn encode_whitespace(data: &str) -> String {
    data.bytes()
        .map(|b| if b & 1 == 1 { "  " } else { " " })
        .collect()
}

fn decode_whitespace(encoded: &str) -> Option<String> {
    let bytes: Vec<u8> = encoded
        .split(' ')
        .filter(|s| !s.is_empty())
        .enumerate()
        .map(|(i, s)| if s.len() > 1 { 1u8 << (i % 8) } else { 0 })
        .collect();
    String::from_utf8(bytes).ok()
}

fn encode_zero_width(data: &str) -> String {
    data.bytes()
        .flat_map(|b| {
            (0..8).map(move |i| {
                if b & (1 << i) != 0 {
                    '\u{200B}' // ZERO WIDTH SPACE
                } else {
                    '\u{200C}' // ZERO WIDTH NON-JOINER
                }
            })
        })
        .collect()
}

fn decode_zero_width(encoded: &str) -> Option<String> {
    let bytes: Vec<u8> = encoded
        .chars()
        .collect::<Vec<_>>()
        .chunks(8)
        .map(|chunk| {
            chunk.iter().enumerate().fold(0u8, |acc, (i, &c)| {
                if c == '\u{200B}' {
                    acc | (1 << i)
                } else {
                    acc
                }
            })
        })
        .collect();
    String::from_utf8(bytes).ok()
}

fn encode_unicode(data: &str) -> String {
    data.chars()
        .map(|c| match c {
            'a' => 'а', // Cyrillic a (U+0430)
            'e' => 'е', // Cyrillic e (U+0435)
            'o' => 'о', // Cyrillic o (U+043E)
            'p' => 'р', // Cyrillic r (U+0440)
            'c' => 'с', // Cyrillic s (U+0441)
            'x' => 'х', // Cyrillic h (U+0445)
            _ => c,
        })
        .collect()
}

fn decode_unicode(encoded: &str) -> Option<String> {
    Some(encoded.chars()
        .map(|c| match c {
            'а' => 'a',
            'е' => 'e',
            'о' => 'o',
            'р' => 'p',
            'с' => 'c',
            'х' => 'x',
            _ => c,
        })
        .collect())
}

fn encode_multi_layer(data: &str) -> String {
    let layer1 = escape_html(data);
    let layer2 = escape_html(&layer1);
    format!("<!-- {} -->", layer2)
}

fn decode_multi_layer(encoded: &str) -> Option<String> {
    extract_from_comment(encoded)
        .and_then(|s| Some(unescape_html(&s)))
        .and_then(|s| Some(unescape_html(&s)))
}

fn extract_from_comment(html: &str) -> Option<String> {
    html.strip_prefix("<!-- ")
        .and_then(|s| s.strip_suffix(" -->"))
        .map(|s| s.to_string())
}

fn extract_from_hidden_div(html: &str) -> Option<String> {
    html.strip_prefix(r#"<div style="display:none">"#)
        .and_then(|s| s.strip_suffix("</div>"))
        .map(|s| unescape_html(s))
}

fn extract_from_data_attr(html: &str) -> Option<String> {
    html.strip_prefix(r#"<div data-erdfa=""#)
        .and_then(|s| s.strip_suffix("\">"))
        .map(|s| unescape_html(s))
}

/// Select best strategy for hostility level
pub fn select_strategy(hostility: HostilityLevel) -> StegoStrategy {
    match hostility {
        HostilityLevel::Friendly => StegoStrategy::HtmlEscape,
        HostilityLevel::Cautious => StegoStrategy::HtmlEscape,
        HostilityLevel::Restrictive => StegoStrategy::DataAttribute,
        HostilityLevel::Aggressive => StegoStrategy::CommentEmbed,
        HostilityLevel::Paranoid => StegoStrategy::Whitespace,
        HostilityLevel::MaximumHostile => StegoStrategy::ZeroWidth,
    }
}

/// Visual steganography module
pub mod visual {
    use super::*;
    
    /// Encode data in element positions
    pub fn encode_position(data: &[u8]) -> Vec<(i32, i32)> {
        data.iter()
            .map(|&b| ((b as i32) % 100, (b as i32) / 100))
            .collect()
    }
    
    /// Decode data from element positions
    pub fn decode_position(positions: &[(i32, i32)]) -> Vec<u8> {
        positions.iter()
            .map(|&(x, y)| ((x % 100) + (y * 100)) as u8)
            .collect()
    }
    
    /// Encode data in element ordering
    pub fn encode_ordering(data: &[u8]) -> Vec<usize> {
        data.iter()
            .enumerate()
            .map(|(i, &b)| (i + b as usize) % 256)
            .collect()
    }
    
    /// Encode data in RGB colors
    pub fn encode_color(data: &[u8]) -> Vec<(u8, u8, u8)> {
        data.chunks(3)
            .map(|chunk| {
                let r = chunk.get(0).copied().unwrap_or(0);
                let g = chunk.get(1).copied().unwrap_or(0);
                let b = chunk.get(2).copied().unwrap_or(0);
                (r, g, b)
            })
            .collect()
    }
    
    /// Decode data from RGB colors
    pub fn decode_color(colors: &[(u8, u8, u8)]) -> Vec<u8> {
        colors.iter()
            .flat_map(|&(r, g, b)| vec![r, g, b])
            .collect()
    }
    
    /// Encode data in CSS property values
    pub fn encode_css(data: &[u8]) -> Vec<String> {
        data.iter()
            .map(|&b| format!("{}px", b))
            .collect()
    }
    
    /// Encode data in font sizes
    pub fn encode_font_size(data: &[u8]) -> Vec<String> {
        data.iter()
            .map(|&b| format!("{}pt", 8 + (b % 16)))
            .collect()
    }
    
    /// Encode data in bitmap LSB
    pub fn encode_bitmap_lsb(data: &[u8], carrier: &mut [u8]) {
        for (i, &byte) in data.iter().enumerate() {
            for bit in 0..8 {
                let idx = i * 8 + bit;
                if idx < carrier.len() {
                    carrier[idx] = (carrier[idx] & 0xFE) | ((byte >> bit) & 1);
                }
            }
        }
    }
    
    /// Decode data from bitmap LSB
    pub fn decode_bitmap_lsb(carrier: &[u8], length: usize) -> Vec<u8> {
        (0..length)
            .map(|i| {
                (0..8)
                    .map(|bit| (carrier[i * 8 + bit] & 1) << bit)
                    .sum()
            })
            .collect()
    }
    
    /// Generate QR code data URL
    pub fn encode_qr_code(data: &str) -> String {
        format!("data:image/svg+xml,<svg><!-- {} --></svg>", data)
    }
    
    /// Encode in imperceptible visual noise
    pub fn encode_visual_noise(data: &[u8]) -> Vec<f32> {
        data.iter()
            .map(|&b| (b as f32) / 255.0 * 0.01) // 1% variation
            .collect()
    }
}

/// Generate HTML with visual steganography
pub fn generate_visual_stego(data: &str) -> String {
    let bytes = data.as_bytes();
    let positions = visual::encode_position(bytes);
    let colors = visual::encode_color(bytes);
    let font_sizes = visual::encode_font_size(bytes);
    
    let mut html = String::from("<div>\n");
    
    for (i, ((x, y), (r, g, b))) in positions.iter().zip(colors.iter()).enumerate() {
        let font_size = font_sizes.get(i).map(|s| s.as_str()).unwrap_or("12pt");
        html.push_str(&format!(
            r#"  <span style="position:absolute;left:{}px;top:{}px;color:rgb({},{},{});font-size:{}">·</span>"#,
            x, y, r, g, b, font_size
        ));
        html.push('\n');
    }
    
    html.push_str("</div>");
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_html_escape() {
        let stego = ERdfaStego;
        let data = r#"<div property="name">Test</div>"#;
        let encoded = stego.encode(data, StegoStrategy::HtmlEscape);
        let decoded = stego.decode(&encoded, StegoStrategy::HtmlEscape).unwrap();
        assert_eq!(data, decoded);
    }
    
    #[test]
    fn test_comment_embed() {
        let stego = ERdfaStego;
        let data = r#"<div>Secret</div>"#;
        let encoded = stego.encode(data, StegoStrategy::CommentEmbed);
        assert!(encoded.starts_with("<!--"));
        let decoded = stego.decode(&encoded, StegoStrategy::CommentEmbed).unwrap();
        assert_eq!(data, decoded);
    }
    
    #[test]
    fn test_zero_width() {
        let stego = ERdfaStego;
        let data = "Hi";
        let encoded = stego.encode(data, StegoStrategy::ZeroWidth);
        assert!(encoded.contains('\u{200B}') || encoded.contains('\u{200C}'));
        let decoded = stego.decode(&encoded, StegoStrategy::ZeroWidth).unwrap();
        assert_eq!(data, decoded);
    }
    
    #[test]
    fn test_hostility_levels() {
        let stego = ERdfaStego;
        assert_eq!(stego.max_hostility(StegoStrategy::HtmlEscape), HostilityLevel::Aggressive);
        assert_eq!(stego.max_hostility(StegoStrategy::ZeroWidth), HostilityLevel::MaximumHostile);
    }
    
    #[test]
    fn test_strategy_selection() {
        assert_eq!(select_strategy(HostilityLevel::Friendly), StegoStrategy::HtmlEscape);
        assert_eq!(select_strategy(HostilityLevel::MaximumHostile), StegoStrategy::ZeroWidth);
    }
    
    #[test]
    fn test_position_encoding() {
        let data = b"Hi";
        let positions = visual::encode_position(data);
        let decoded = visual::decode_position(&positions);
        assert_eq!(data[0], decoded[0]);
    }
    
    #[test]
    fn test_color_encoding() {
        let data = b"RGB";
        let colors = visual::encode_color(data);
        let decoded = visual::decode_color(&colors);
        assert_eq!(data.to_vec(), decoded);
    }
    
    #[test]
    fn test_bitmap_lsb() {
        let data = b"Secret";
        let mut carrier = vec![0u8; 64];
        visual::encode_bitmap_lsb(data, &mut carrier);
        let decoded = visual::decode_bitmap_lsb(&carrier, data.len());
        assert_eq!(data.to_vec(), decoded);
    }
    
    #[test]
    fn test_visual_stego_generation() {
        let html = generate_visual_stego("Test");
        assert!(html.contains("position:absolute"));
        assert!(html.contains("color:rgb"));
        assert!(html.contains("font-size"));
    }
}
