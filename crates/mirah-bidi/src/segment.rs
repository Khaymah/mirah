use crate::cell::Cell;
use crate::classify::{MIN_GAP_COLUMNS, is_frame, is_strong};
use std::ops::Range;

const PREFIX_MARKERS: &str = "●•◦-*>⎿❯#.)";

fn is_prefix(c: char) -> bool {
    !is_strong(c) && (c == ' ' || c.is_ascii_digit() || PREFIX_MARKERS.contains(c))
}

pub(crate) struct Row<'a> {
    pub(crate) cells: &'a [Cell],
    pub(crate) cursor: Option<usize>,
}

impl Row<'_> {
    fn inverse(&self, i: usize) -> bool {
        self.cells.get(i).is_some_and(|c| c.style.inverse)
    }

    fn is_lone_inverse(&self, i: usize) -> bool {
        self.inverse(i) && !(i > 0 && self.inverse(i - 1)) && !self.inverse(i + 1)
    }

    fn is_cursor(&self, i: usize) -> bool {
        self.cursor == Some(i) || self.is_lone_inverse(i)
    }

    fn is_open_blank(&self, i: usize) -> bool {
        !self.cells[i].continuation && self.cells[i].is_blank() && !self.is_cursor(i)
    }

    fn style_key(&self, i: usize) -> bool {
        self.inverse(i) && !self.is_lone_inverse(i)
    }

    fn fixed_columns(&self) -> Vec<bool> {
        let cells = self.cells;
        let mut fixed: Vec<bool> = cells.iter().map(|c| is_frame(c.base())).collect();
        let mut i = 0;
        while i < cells.len() {
            if !self.is_open_blank(i) {
                i += 1;
                continue;
            }
            let start = i;
            while i < cells.len() && self.is_open_blank(i) {
                i += 1;
            }
            if i - start >= MIN_GAP_COLUMNS {
                fixed[start..i].iter_mut().for_each(|f| *f = true);
            }
        }
        fixed
    }

    pub(crate) fn segments(&self) -> Vec<Range<usize>> {
        let fixed = self.fixed_columns();
        let mut out = Vec::new();
        let mut i = 0;
        while i < self.cells.len() {
            if fixed[i] {
                i += 1;
                continue;
            }
            let start = i;
            let key = self.style_key(i);
            while i < self.cells.len() && !fixed[i] && self.style_key(i) == key {
                i += 1;
            }
            out.push(start..i);
        }
        out
    }

    pub(crate) fn trim(&self, span: Range<usize>) -> Range<usize> {
        let floor = span.start;
        let (mut start, mut end) = (span.start, span.end);
        while start < end && is_prefix(self.cells[start].base()) {
            start += 1;
        }
        while start > floor && start < end && !self.cells[start - 1].is_blank() {
            start -= 1;
        }
        while end > start && self.is_open_blank(end - 1) {
            end -= 1;
        }
        start..end
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::cell::Style;

    fn row(s: &str) -> Vec<Cell> {
        s.chars().map(|c| Cell::plain(c.to_string())).collect()
    }

    fn segments(cells: &[Cell]) -> Vec<Range<usize>> {
        Row {
            cells,
            cursor: None,
        }
        .segments()
    }

    fn trim(cells: &[Cell], span: Range<usize>) -> Range<usize> {
        Row {
            cells,
            cursor: None,
        }
        .trim(span)
    }

    fn invert(cells: &mut [Cell], columns: &[usize]) {
        for &i in columns {
            cells[i].style = Style {
                inverse: true,
                ..Style::default()
            };
        }
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

    #[test]
    fn trim_keeps_punctuation_touching_the_first_word() {
        let r = row("● **ab** ");
        assert_eq!(trim(&r, 0..r.len()), 2..8);
    }

    #[test]
    fn a_background_change_does_not_split_the_sentence() {
        let mut r = row("ab cd");
        for cell in &mut r[3..] {
            cell.style.bg = crate::cell::Color::Rgb(238, 238, 238);
        }
        assert_eq!(segments(&r), vec![0..5]);
    }

    #[test]
    fn a_lone_inverse_cell_does_not_split_the_sentence() {
        let mut r = row("ab cd");
        invert(&mut r, &[3]);
        assert_eq!(segments(&r), vec![0..5]);
    }

    #[test]
    fn a_run_of_inverse_cells_still_splits() {
        let mut r = row("ab cd");
        invert(&mut r, &[3, 4]);
        assert_eq!(segments(&r), vec![0..3, 3..5]);
    }

    #[test]
    fn a_cursor_blank_after_the_text_stays_in_the_segment() {
        let mut r = row("ab    ");
        invert(&mut r, &[2]);
        assert_eq!(segments(&r), vec![0..3]);
        assert_eq!(trim(&r, 0..3), 0..3);
        let real = Row {
            cells: &row("ab    "),
            cursor: Some(2),
        };
        assert_eq!(real.trim(0..3), 0..3);
    }
}
