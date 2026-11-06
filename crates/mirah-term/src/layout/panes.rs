use super::Cursor;
use mirah_bidi::{Cell, VisualRow};
use std::ops::Range;

const SHARE_NUM: usize = 3;
const SHARE_DEN: usize = 4;
const MIN_ROWS: usize = 4;
const MIN_PANE: usize = 10;

pub(super) fn boundaries(rows: &[Vec<Cell>]) -> Vec<usize> {
    let width = rows.iter().map(Vec::len).min().unwrap_or(0);
    if rows.len() < MIN_ROWS || width < 2 * MIN_PANE {
        return Vec::new();
    }
    let mut counts = vec![0usize; width];
    for row in rows {
        for c in 1..width {
            if row[c].style.bg != row[c - 1].style.bg {
                counts[c] += 1;
            }
        }
    }
    let needed = rows.len() * SHARE_NUM / SHARE_DEN;
    let mut cuts = Vec::new();
    let mut last = 0;
    for (c, &count) in counts.iter().enumerate().skip(MIN_PANE) {
        if count >= needed && c - last >= MIN_PANE && width - c >= MIN_PANE {
            cuts.push(c);
            last = c;
        }
    }
    cuts
}

pub(super) fn ranges(cuts: &[usize], width: usize) -> Vec<Range<usize>> {
    let mut starts = vec![0];
    starts.extend_from_slice(cuts);
    let mut ends = cuts.to_vec();
    ends.push(width);
    starts.into_iter().zip(ends).map(|(a, b)| a..b).collect()
}

pub(super) fn slice(rows: &[Vec<Cell>], range: &Range<usize>) -> Vec<Vec<Cell>> {
    rows.iter().map(|row| row[range.clone()].to_vec()).collect()
}

pub(super) fn cursor_in(cursor: &Cursor, range: &Range<usize>) -> Cursor {
    let inside = range.contains(&cursor.col);
    Cursor {
        col: if inside { cursor.col - range.start } else { 0 },
        visible: cursor.visible && inside,
        ..*cursor
    }
}

pub(super) fn stitch(parts: Vec<(Range<usize>, Vec<VisualRow>)>, rows: usize) -> Vec<VisualRow> {
    let mut out: Vec<VisualRow> = (0..rows)
        .map(|_| VisualRow {
            cells: Vec::new(),
            logical_to_visual: Vec::new(),
        })
        .collect();
    for (range, pane) in parts {
        for (row, part) in out.iter_mut().zip(pane) {
            row.cells.extend(part.cells);
            row.logical_to_visual
                .extend(part.logical_to_visual.into_iter().map(|v| v + range.start));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use mirah_bidi::Color;

    fn screen(rows: usize, split: usize, width: usize) -> Vec<Vec<Cell>> {
        (0..rows)
            .map(|_| {
                (0..width)
                    .map(|c| {
                        let mut cell = Cell::plain(" ");
                        cell.style.bg = if c < split {
                            Color::Rgb(10, 10, 10)
                        } else {
                            Color::Rgb(20, 20, 20)
                        };
                        cell
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn finds_a_column_where_the_background_changes_on_most_rows() {
        assert_eq!(boundaries(&screen(8, 30, 50)), vec![30]);
        assert_eq!(ranges(&[30], 50), vec![0..30, 30..50]);
    }

    #[test]
    fn a_plain_screen_or_a_short_highlight_has_no_panes() {
        assert!(boundaries(&screen(8, 50, 50)).is_empty());
        let mut rows = screen(8, 50, 50);
        for row in rows.iter_mut().take(3) {
            row[20].style.bg = Color::Rgb(238, 238, 238);
        }
        assert!(boundaries(&rows).is_empty());
    }

    #[test]
    fn a_narrow_strip_is_not_a_pane() {
        assert!(boundaries(&screen(8, 45, 50)).is_empty());
    }

    #[test]
    fn stitching_offsets_each_pane_back_into_place() {
        let part = |n: usize| VisualRow {
            cells: vec![Cell::plain("x"); n],
            logical_to_visual: (0..n).rev().collect(),
        };
        let rows = stitch(vec![(0..2, vec![part(2)]), (2..5, vec![part(3)])], 1);
        assert_eq!(rows[0].logical_to_visual, vec![1, 0, 4, 3, 2]);
        assert_eq!(rows[0].cells.len(), 5);
    }
}
