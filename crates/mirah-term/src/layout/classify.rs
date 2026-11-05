use super::{Align, Cursor};
use mirah_bidi::{Cell, Color};
use unicode_bidi::{BidiClass, bidi_class};

const TOOL_OUTPUT: char = '⎿';
const CODE_INDENT: usize = 4;
const CODE_CHAR_RATIO: f32 = 0.12;
const CODE_CHARS: &str = "{}[]();=<>&|$`\\_#";
const COLORED_RATIO: f32 = 0.5;
const MIN_LETTERS: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Prose,
    Framed,
    Fixed,
}

pub(super) fn base(cell: &Cell) -> char {
    cell.text.chars().next().unwrap_or(' ')
}

pub(super) fn is_blank(cell: &Cell) -> bool {
    base(cell) == ' '
}

pub(super) fn is_rtl(c: char) -> bool {
    matches!(bidi_class(c), BidiClass::R | BidiClass::AL)
}

pub(super) fn is_frame(c: char) -> bool {
    ('\u{2500}'..='\u{259F}').contains(&c)
}

fn has_rtl(row: &[Cell]) -> bool {
    row.iter().any(|c| c.text.chars().any(is_rtl))
}

fn is_lone_inverse(row: &[Cell], i: usize) -> bool {
    let inverse = |j: usize| row.get(j).is_some_and(|c| c.style.inverse);
    inverse(i) && !(i > 0 && inverse(i - 1)) && !inverse(i + 1)
}

fn input_row(logical: &[Vec<Cell>], cursor: &Cursor) -> Option<usize> {
    let fake = logical
        .iter()
        .rposition(|row| (0..row.len()).any(|i| is_lone_inverse(row, i)));
    let real = cursor.visible.then_some(cursor.row);
    fake.max(real)
}

fn looks_like_code(row: &[Cell]) -> bool {
    let content: Vec<&Cell> = row.iter().filter(|c| !is_blank(c)).collect();
    if content.is_empty() {
        return false;
    }
    let total = content.len() as f32;
    let colored = content
        .iter()
        .filter(|c| c.style.fg != Color::Default)
        .count() as f32;
    let highlighted = content.iter().any(|c| c.style.bg != Color::Default);
    let code_chars = content
        .iter()
        .filter(|c| CODE_CHARS.contains(base(c)))
        .count() as f32;
    let indent = row.iter().take_while(|c| is_blank(c)).count();
    let first = content.first().map(|c| base(c)).unwrap_or(' ');
    highlighted
        || colored / total > COLORED_RATIO
        || code_chars / total > CODE_CHAR_RATIO
        || first == TOOL_OUTPUT
        || (indent >= CODE_INDENT && !has_rtl(row) && !super::align::has_marker(row, indent))
}

fn has_gap(row: &[Cell]) -> bool {
    let first = row.iter().position(|c| !is_blank(c));
    let last = row.iter().rposition(|c| !is_blank(c));
    let (Some(first), Some(last)) = (first, last) else {
        return false;
    };
    row[first..=last]
        .windows(2)
        .any(|w| is_blank(&w[0]) && is_blank(&w[1]))
}

fn is_prose(row: &[Cell]) -> bool {
    let letters = row.iter().filter(|c| base(c).is_alphabetic()).count();
    letters >= MIN_LETTERS && !looks_like_code(row) && !has_gap(row)
}

pub(super) fn classify(logical: &[Vec<Cell>], cursor: &Cursor, align: Align) -> Vec<Kind> {
    let input = input_row(logical, cursor);
    let prose: Vec<bool> = logical
        .iter()
        .enumerate()
        .map(|(i, row)| {
            !input.is_some_and(|r| i >= r)
                && !row.iter().any(|c| is_frame(base(c)))
                && is_prose(row)
        })
        .collect();
    let arabic_on_screen = logical
        .iter()
        .zip(&prose)
        .any(|(row, p)| *p && has_rtl(row));
    logical
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let framed = row.iter().any(|c| is_frame(base(c)));
            let below_input = input.is_some_and(|r| i >= r);
            if framed && !below_input && has_rtl(row) {
                return Kind::Framed;
            }
            let wanted = !cursor.editor
                && match align {
                    Align::Prose => arabic_on_screen,
                    Align::Arabic => has_rtl(row),
                    Align::Off => false,
                };
            if prose[i] && wanted {
                Kind::Prose
            } else {
                Kind::Fixed
            }
        })
        .collect()
}
