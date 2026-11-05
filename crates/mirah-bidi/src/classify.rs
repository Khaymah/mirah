use std::ops::RangeInclusive;
use unicode_bidi::{BidiClass, bidi_class};

pub(crate) const MIN_GAP_COLUMNS: usize = 2;
pub(crate) const FRAME_CHARS: RangeInclusive<char> = '\u{2500}'..='\u{259F}';

pub(crate) fn is_rtl(c: char) -> bool {
    matches!(bidi_class(c), BidiClass::R | BidiClass::AL)
}

pub(crate) fn is_strong(c: char) -> bool {
    matches!(bidi_class(c), BidiClass::L | BidiClass::R | BidiClass::AL)
}

pub(crate) fn is_frame(c: char) -> bool {
    FRAME_CHARS.contains(&c)
}

pub(crate) fn has_rtl(text: &str) -> bool {
    text.chars().any(is_rtl)
}
