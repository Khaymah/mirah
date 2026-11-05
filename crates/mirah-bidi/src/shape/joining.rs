use super::forms::{LETTERS, forms};
use std::ops::RangeInclusive;
use unicode_bidi::{BidiClass, bidi_class};

const TATWEEL: char = '\u{0640}';
const PRESENTATION_FORMS: RangeInclusive<char> = '\u{FB50}'..='\u{FEFC}';

#[derive(Clone, Copy, PartialEq, Eq)]
enum Joining {
    Dual,
    Right,
    Causing,
}

fn joining(c: char) -> Option<Joining> {
    if c == TATWEEL {
        return Some(Joining::Causing);
    }
    forms(c).map(|f| {
        if f.initial.is_some() {
            Joining::Dual
        } else {
            Joining::Right
        }
    })
}

pub(crate) fn joins_next(c: Option<char>) -> bool {
    matches!(c.and_then(joining), Some(Joining::Dual | Joining::Causing))
}

pub(crate) fn joins_prev(c: Option<char>) -> bool {
    c.and_then(joining).is_some()
}

pub(crate) fn is_transparent(c: char) -> bool {
    bidi_class(c) == BidiClass::NSM
}

pub(crate) fn unshape(c: char) -> char {
    if !PRESENTATION_FORMS.contains(&c) {
        return c;
    }
    LETTERS
        .iter()
        .copied()
        .find(|&letter| forms(letter).is_some_and(|f| f.contains(c)))
        .unwrap_or(c)
}
