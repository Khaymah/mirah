use crate::cell::Cell;
use crate::classify::has_rtl;
use crate::mirror::mirror;
use crate::segment::Row;

const PAIRS: [(char, char); 4] = [('(', ')'), ('[', ']'), ('{', '}'), ('«', '»')];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Side {
    Open,
    Close,
}

fn single_char(cell: &Cell) -> char {
    let mut chars = cell.text.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => c,
        _ => '\0',
    }
}

fn bracket(c: char) -> Option<(usize, Side)> {
    PAIRS.iter().enumerate().find_map(|(kind, (open, close))| {
        if c == *open {
            Some((kind, Side::Open))
        } else if c == *close {
            Some((kind, Side::Close))
        } else {
            None
        }
    })
}

pub(crate) fn reversed_pairs(chars: &[char]) -> Vec<(usize, usize)> {
    let mut stack: Vec<(usize, usize, Side)> = Vec::new();
    let mut pairs = Vec::new();
    for (column, &c) in chars.iter().enumerate() {
        let Some((kind, side)) = bracket(c) else {
            continue;
        };
        let wanted = match side {
            Side::Close => Side::Open,
            Side::Open => Side::Close,
        };
        match stack.last() {
            Some(&(start, top, first)) if top == kind && first == wanted => {
                stack.pop();
                if first == Side::Close {
                    pairs.push((start, column));
                }
            }
            _ => stack.push((column, kind, side)),
        }
    }
    pairs
}

pub fn mirror_reversed_brackets(row: &[Cell]) -> Vec<Cell> {
    let mut out = row.to_vec();
    if !row.iter().any(|c| has_rtl(&c.text)) {
        return out;
    }
    let layout = Row {
        cells: row,
        cursor: None,
    };
    for span in layout.segments() {
        if !row[span.clone()].iter().any(|c| has_rtl(&c.text)) {
            continue;
        }
        let chars: Vec<char> = row[span.clone()].iter().map(single_char).collect();
        for (start, end) in reversed_pairs(&chars) {
            for column in [span.start + start, span.start + end] {
                out[column].text = row[column].text.chars().map(mirror).collect();
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(s: &str) -> Vec<Cell> {
        s.chars().map(|c| Cell::plain(c.to_string())).collect()
    }

    fn fixed(s: &str) -> String {
        mirror_reversed_brackets(&row(s))
            .iter()
            .map(|c| c.text.as_str())
            .collect()
    }

    #[test]
    fn reversed_pairs_around_arabic_are_mirrored() {
        assert_eq!(fixed(")لودج( يلمعا"), "(لودج) يلمعا");
        assert_eq!(
            fixed(")pty الب ،bytes → frames( mirah-term"),
            "(pty الب ،bytes → frames) mirah-term"
        );
    }

    #[test]
    fn pairs_in_reading_order_stay() {
        assert_eq!(fixed("f(x) لاد"), "f(x) لاد");
        assert_eq!(fixed("(لودج)"), "(لودج)");
    }

    #[test]
    fn reversed_pairs_around_latin_in_arabic_text_are_mirrored() {
        assert_eq!(fixed("wrapper — )mirah( ةآرم"), "wrapper — (mirah) ةآرم");
    }

    #[test]
    fn segments_without_arabic_stay() {
        assert_eq!(fixed(")x(  بتكا"), ")x(  بتكا");
    }

    #[test]
    fn table_cells_are_handled_on_their_own() {
        assert_eq!(fixed(")ب │ ا("), ")ب │ ا(");
    }
}
