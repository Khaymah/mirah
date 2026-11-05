use crate::cell::Cell;
use crate::classify::{MIN_GAP_COLUMNS, is_frame, is_strong};
use std::ops::Range;

fn fixed_columns(row: &[Cell]) -> Vec<bool> {
    let mut fixed: Vec<bool> = row
        .iter()
        .map(|c| c.wide || c.continuation || is_frame(c.base()))
        .collect();
    let mut i = 0;
    while i < row.len() {
        if !row[i].is_blank() {
            i += 1;
            continue;
        }
        let start = i;
        while i < row.len() && row[i].is_blank() {
            i += 1;
        }
        if i - start >= MIN_GAP_COLUMNS {
            fixed[start..i].iter_mut().for_each(|f| *f = true);
        }
    }
    fixed
}

pub(crate) fn segments(row: &[Cell]) -> Vec<Range<usize>> {
    let fixed = fixed_columns(row);
    let mut out = Vec::new();
    let mut i = 0;
    while i < row.len() {
        if fixed[i] {
            i += 1;
            continue;
        }
        let start = i;
        let key = (row[i].style.bg, row[i].style.inverse);
        while i < row.len() && !fixed[i] && (row[i].style.bg, row[i].style.inverse) == key {
            i += 1;
        }
        out.push(start..i);
    }
    out
}

pub(crate) fn trim(row: &[Cell], span: Range<usize>) -> Range<usize> {
    let (mut start, mut end) = (span.start, span.end);
    while start < end && !is_strong(row[start].base()) {
        start += 1;
    }
    while end > start && row[end - 1].is_blank() {
        end -= 1;
    }
    start..end
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(s: &str) -> Vec<Cell> {
        s.chars().map(|c| Cell::plain(c.to_string())).collect()
    }

    #[test]
    fn frames_and_wide_gaps_split_segments() {
        assert_eq!(segments(&row("│ ab  cd │")), vec![1..4, 6..9]);
    }

    #[test]
    fn single_space_does_not_split() {
        assert_eq!(segments(&row("ab cd")), vec![0..5]);
    }

    #[test]
    fn trim_skips_leading_neutrals_and_trailing_blanks() {
        let r = row("> ab  ");
        assert_eq!(trim(&r, 0..r.len()), 2..4);
    }
}
