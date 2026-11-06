use super::{Align, Cursor};
use mirah_bidi::{Cell, Color};
use unicode_bidi::{BidiClass, bidi_class};

const TOOL_OUTPUT: char = '⎿';
const CODE_INDENT: usize = 4;
const CODE_CHAR_RATIO: f32 = 0.12;
const CODE_CHARS: &str = "{}[]();=<>&|$`\\_#";
const COLORED_RATIO: f32 = 0.5;
const MIN_LETTERS: usize = 2;
const WHITE: u8 = 7;
const BRIGHT_WHITE: u8 = 15;

fn is_syntax_color(color: Color, text: Color) -> bool {
    color != text
        && !matches!(
            color,
            Color::Default | Color::Indexed(WHITE) | Color::Indexed(BRIGHT_WHITE)
        )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Palette {
    fg: Color,
    bg: Color,
}

fn dominant(colors: impl Iterator<Item = Color>) -> Color {
    let mut counts: Vec<(Color, usize)> = Vec::new();
    for color in colors {
        match counts.iter_mut().find(|(c, _)| *c == color) {
            Some((_, n)) => *n += 1,
            None => counts.push((color, 1)),
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map_or(Color::Default, |(c, _)| c)
}

fn palette(logical: &[Vec<Cell>]) -> Palette {
    let letters = || {
        logical
            .iter()
            .flatten()
            .filter(|cell| base(cell).is_alphabetic())
    };
    let bg = dominant(letters().map(|c| c.style.bg));
    let fg = dominant(letters().filter(|c| c.style.bg == bg).map(|c| c.style.fg));
    Palette { fg, bg }
}

fn is_diff_line(content: &[&Cell]) -> bool {
    let digits = content
        .iter()
        .take_while(|c| base(c).is_ascii_digit())
        .count();
    digits > 0
        && content.get(digits).is_some_and(|c| is_blank(c))
        && content
            .iter()
            .skip(digits)
            .find(|c| !is_blank(c))
            .is_some_and(|c| matches!(base(c), '+' | '-'))
}

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
    !cell.continuation && base(cell) == ' '
}

pub(super) fn blank_like(cell: Option<&Cell>) -> Cell {
    let mut blank = Cell::plain(" ");
    if let Some(cell) = cell {
        blank.style.bg = cell.style.bg;
        blank.style.fg = cell.style.fg;
    }
    blank
}

pub(super) fn is_rtl(c: char) -> bool {
    matches!(bidi_class(c), BidiClass::R | BidiClass::AL)
}

pub(super) fn is_frame(c: char) -> bool {
    ('\u{2500}'..='\u{259F}').contains(&c)
}

pub(super) fn has_rtl(row: &[Cell]) -> bool {
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

fn looks_like_code(row: &[Cell], text: Palette) -> bool {
    let content: Vec<&Cell> = row.iter().filter(|c| !is_blank(c)).collect();
    if content.is_empty() {
        return false;
    }
    let total = content.len() as f32;
    let plain_bg = content.iter().filter(|c| c.style.bg == text.bg).count() as f32;
    let colored = content
        .iter()
        .filter(|c| c.style.bg == text.bg && is_syntax_color(c.style.fg, text.fg))
        .count() as f32;
    let code_chars = content
        .iter()
        .filter(|c| CODE_CHARS.contains(base(c)))
        .count() as f32;
    let indent = row.iter().take_while(|c| is_blank(c)).count();
    let first = content.first().map(|c| base(c)).unwrap_or(' ');
    let letters: Vec<&&Cell> = content.iter().filter(|c| base(c).is_alphabetic()).collect();
    let heading = has_rtl(row)
        && letters.iter().all(|c| c.style.bold)
        && letters.windows(2).all(|w| w[0].style.fg == w[1].style.fg);
    is_diff_line(&content)
        || (!heading && plain_bg * 2.0 >= total && colored / plain_bg > COLORED_RATIO)
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

fn is_prose(row: &[Cell], text: Palette) -> bool {
    let letters = row.iter().filter(|c| base(c).is_alphabetic()).count();
    letters >= MIN_LETTERS && !looks_like_code(row, text) && !has_gap(row)
}

fn is_empty_row(row: &[Cell]) -> bool {
    row.iter().all(is_blank)
}

fn rows_in_code_blocks(logical: &[Vec<Cell>], text: Palette) -> Vec<bool> {
    let mut in_code = vec![false; logical.len()];
    let mut start = 0;
    while start < logical.len() {
        if is_empty_row(&logical[start]) {
            start += 1;
            continue;
        }
        let end = (start..logical.len())
            .find(|&i| is_empty_row(&logical[i]))
            .unwrap_or(logical.len());
        if logical[start..end]
            .iter()
            .any(|row| looks_like_code(row, text))
        {
            for (i, row) in logical.iter().enumerate().take(end).skip(start) {
                in_code[i] = !has_rtl(row);
            }
        }
        start = end;
    }
    in_code
}

pub(super) fn classify(logical: &[Vec<Cell>], cursor: &Cursor, align: Align) -> Vec<Kind> {
    let input = input_row(logical, cursor);
    let text = palette(logical);
    let in_code = rows_in_code_blocks(logical, text);
    let prose: Vec<bool> = logical
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let fixed_input = input.is_some_and(|r| i > r || (i == r && !has_rtl(row)));
            !fixed_input
                && !in_code[i]
                && !row.iter().any(|c| is_frame(base(c)))
                && is_prose(row, text)
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
