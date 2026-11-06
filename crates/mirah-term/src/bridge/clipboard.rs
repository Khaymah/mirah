use mirah_bidi::{Cell, logical_row};
use unicode_bidi::{BidiClass, bidi_class};

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const PAD: u8 = b'=';

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Clipboard {
    pub(crate) allow: bool,
    pub(crate) visual_text: bool,
}

impl Default for Clipboard {
    fn default() -> Self {
        Self {
            allow: true,
            visual_text: false,
        }
    }
}

fn sextet(b: u8) -> Option<u32> {
    ALPHABET
        .iter()
        .position(|&a| a == b)
        .and_then(|p| u32::try_from(p).ok())
}

pub(crate) fn decode(text: &[u8]) -> Option<Vec<u8>> {
    let clean: Vec<u8> = text
        .iter()
        .copied()
        .filter(|b| !b.is_ascii_whitespace())
        .collect();
    let mut out = Vec::with_capacity(clean.len() / 4 * 3);
    for chunk in clean.chunks(4) {
        let pads = chunk.iter().filter(|&&b| b == PAD).count();
        let mut acc = 0u32;
        for (i, &b) in chunk.iter().enumerate() {
            let value = if b == PAD { 0 } else { sextet(b)? };
            acc |= value << (18 - 6 * i);
        }
        let bytes = acc.to_be_bytes();
        let keep = chunk.len().saturating_sub(1).saturating_sub(pads);
        out.extend_from_slice(&bytes[1..=keep.min(3)]);
    }
    Some(out)
}

pub(crate) fn encode(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let mut acc = 0u32;
        for (i, &b) in chunk.iter().enumerate() {
            acc |= u32::from(b) << (16 - 8 * i);
        }
        for i in 0..4 {
            if i <= chunk.len() {
                let index = usize::try_from((acc >> (18 - 6 * i)) & 0x3f).unwrap_or(0);
                out.push(ALPHABET[index]);
            } else {
                out.push(PAD);
            }
        }
    }
    out
}

fn logical_line(line: &str) -> String {
    let mut cells: Vec<Cell> = Vec::new();
    for c in line.chars() {
        match cells.last_mut() {
            Some(last) if bidi_class(c) == BidiClass::NSM => last.text.push(c),
            _ => cells.push(Cell::plain(c.to_string())),
        }
    }
    match logical_row(&cells, None) {
        Some(row) => row.cells.iter().map(|c| c.text.as_str()).collect(),
        None => line.to_string(),
    }
}

pub(crate) fn to_logical(data: &[u8]) -> Option<Vec<u8>> {
    let decoded = decode(data)?;
    let text = std::str::from_utf8(&decoded).ok()?;
    let fixed: Vec<String> = text.split('\n').map(logical_line).collect();
    Some(encode(fixed.join("\n").as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_round_trips() {
        for text in ["", "a", "ab", "abc", "abcd", "مرحبا\nworld"] {
            let encoded = encode(text.as_bytes());
            assert_eq!(decode(&encoded).as_deref(), Some(text.as_bytes()));
        }
        assert_eq!(encode(b"hello"), b"aGVsbG8=");
        assert_eq!(decode(b"aGVs\nbG8="), Some(b"hello".to_vec()));
        assert_eq!(decode(b"a$=="), None);
    }

    #[test]
    fn marks_stay_on_their_letter() {
        let copied = encode("اًلعف .7".as_bytes());
        let fixed = to_logical(&copied)
            .and_then(|b| decode(&b))
            .unwrap_or_default();
        assert_eq!(String::from_utf8_lossy(&fixed), "7. فعلاً");
    }

    #[test]
    fn visual_arabic_comes_out_in_reading_order() {
        let copied = encode("ءادألا .5\nRust ـلا".as_bytes());
        let fixed = to_logical(&copied)
            .and_then(|b| decode(&b))
            .unwrap_or_default();
        assert_eq!(String::from_utf8_lossy(&fixed), "5. الأداء\nالـ Rust");
    }
}
