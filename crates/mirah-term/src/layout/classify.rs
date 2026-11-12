use super::{Align, Cursor};
use mirah_bidi::{Cell, Color};
use unicode_bidi::{BidiClass, bidi_class};

const TOOL_OUTPUT: char = '⎿';
const CODE_INDENT: usize = 4;
const CODE_CHAR_RATIO: f32 = 0.12;
const ARABIC_CODE_CHAR_RATIO: f32 = 0.25;
const GRAY_SPREAD: u8 = 24;
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
pub(super) struct Palette {
    pub(super) fg: Color,
    pub(super) bg: Color,
}

fn dominant(colors: impl Iterator<Item = Color>) -> Color {
    let mut counts: std::collections::HashMap<Color, (usize, usize)> =
        std::collections::HashMap::new();
    for (seen, color) in colors.enumerate() {
        counts.entry(color).or_insert((0, seen)).0 += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(_, (n, first))| (*n, *first))
        .map_or(Color::Default, |(c, _)| c)
}

pub(super) fn palette(logical: &[Vec<Cell>]) -> Palette {
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

fn is_diff_line(row: &[Cell]) -> bool {
    let start = row.iter().take_while(|c| is_blank(c)).count();
    let line = &row[start..];
    let digits = line.iter().take_while(|c| base(c).is_ascii_digit()).count();
    digits > 0
        && line.get(digits).is_some_and(is_blank)
        && line
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
    Code,
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

fn is_gray(color: Color) -> bool {
    match color {
        Color::Rgb(r, g, b) => {
            let (high, low) = (r.max(g).max(b), r.min(g).min(b));
            high - low <= GRAY_SPREAD
        }
        Color::Indexed(n) => matches!(n, 8 | 232..=255),
        Color::Default => true,
    }
}

fn rtl_in_text_color(content: &[&Cell], text: Palette) -> bool {
    let rtl: Vec<&&Cell> = content
        .iter()
        .filter(|c| c.text.chars().any(is_rtl))
        .collect();
    let plain = rtl
        .iter()
        .filter(|c| !is_syntax_color(c.style.fg, text.fg) || is_gray(c.style.fg))
        .count();
    plain * 2 > rtl.len()
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
    let rtl = has_rtl(row);
    let arabic_prose = rtl && rtl_in_text_color(&content, text);
    let char_ratio = if rtl {
        ARABIC_CODE_CHAR_RATIO
    } else {
        CODE_CHAR_RATIO
    };
    is_diff_line(row)
        || (!heading
            && !arabic_prose
            && plain_bg * 2.0 >= total
            && colored / plain_bg > COLORED_RATIO)
        || code_chars / total > char_ratio
        || first == TOOL_OUTPUT
        || (indent >= CODE_INDENT && !has_rtl(row) && !super::align::has_marker(row, indent))
}

fn mostly_latin_code(row: &[Cell], text: Palette) -> bool {
    let letters: Vec<char> = row.iter().map(base).filter(|c| c.is_alphabetic()).collect();
    let rtl = letters.iter().filter(|c| is_rtl(**c)).count();
    rtl > 0 && rtl * 2 < letters.len() && looks_like_code(row, text)
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
            } else if !framed && !below_input && mostly_latin_code(row, text) {
                Kind::Code
            } else {
                Kind::Fixed
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CODE: Color = Color::Rgb(230, 160, 90);

    fn row(parts: &[(&str, Color)]) -> Vec<Cell> {
        let mut cells: Vec<Cell> = Vec::new();
        for (text, fg) in parts {
            for c in text.chars() {
                let mut cell = Cell::plain(c.to_string());
                cell.style.fg = *fg;
                cells.push(cell);
            }
        }
        cells.resize(60, Cell::plain(" "));
        cells
    }

    fn kinds(rows: &[Vec<Cell>]) -> Vec<Kind> {
        let cursor = Cursor {
            row: 0,
            col: 0,
            visible: false,
            editor: false,
        };
        classify(rows, &cursor, Align::Prose)
    }

    fn screen(line: Vec<Cell>) -> Vec<Vec<Cell>> {
        let prose = row(&[("هذا سطر عادي من الكلام العربي الطويل نسبيًا", Color::Default)]);
        vec![prose.clone(), prose, line]
    }

    #[test]
    fn a_numbered_diff_line_keeps_its_place_even_with_arabic() {
        let line = row(&[
            ("  12 + ", Color::Default),
            ("// تعليق عن الدالة هنا", Color::Default),
        ]);
        assert_ne!(kinds(&screen(line))[2], Kind::Prose);
        let list = row(&[
            ("1. ", Color::Default),
            ("هذا بند في قائمة عادية", Color::Default),
        ]);
        assert_eq!(kinds(&screen(list))[2], Kind::Prose);
    }

    #[test]
    fn arabic_with_coloured_inline_code_is_prose() {
        let line = row(&[
            ("- ", Color::Default),
            ("Rust:", CODE),
            (" بيستخدم ", Color::Default),
            ("Result<T, E>", CODE),
        ]);
        assert_eq!(kinds(&screen(line))[2], Kind::Prose);
    }

    #[test]
    fn arabic_with_a_few_code_characters_is_prose() {
        let line = row(&[("claude ready args=[] مرحبا", Color::Default)]);
        assert_eq!(kinds(&screen(line))[2], Kind::Prose);
    }

    #[test]
    fn grey_arabic_like_a_thought_is_prose() {
        let grey = Color::Rgb(128, 128, 128);
        let line = row(&[("الاتنين لغات حديثة و", grey)]);
        assert_eq!(kinds(&screen(line))[2], Kind::Prose);
    }

    #[test]
    fn arabic_prose_that_looks_like_code_keeps_its_direction() {
        let line = row(&[
            ("- ", Color::Default),
            ("Go:", CODE),
            (" اتعملت عشان البساطة", CODE),
        ]);
        assert_ne!(kinds(&screen(line))[2], Kind::Code);
    }

    #[test]
    fn coloured_code_stays_fixed_even_with_an_arabic_string() {
        let line = row(&[
            ("println!", CODE),
            ("(\"", Color::Default),
            ("مرحبا", Color::Rgb(120, 200, 120)),
            ("\");", Color::Default),
        ]);
        assert_eq!(kinds(&screen(line))[2], Kind::Code);
        let plain_code = row(&[("let x = foo(a, b);", CODE)]);
        assert_eq!(kinds(&screen(plain_code))[2], Kind::Fixed);
    }
}
