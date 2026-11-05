mod cell;
mod classify;
mod mirror;
mod reorder;
mod segment;
mod shape;

pub use cell::{Cell, Color, Style, VisualRow};
pub use shape::{Shaped, shape};

pub fn visual_row(row: &[Cell]) -> VisualRow {
    visual_row_with_cursor(row, None)
}

pub fn visual_row_with_cursor(row: &[Cell], cursor: Option<usize>) -> VisualRow {
    let mut out = VisualRow::identity(row);
    if !row.iter().any(|c| classify::has_rtl(&c.text)) {
        return out;
    }
    let layout = segment::Row { cells: row, cursor };
    for span in layout.segments() {
        let span = layout.trim(span);
        if !span.is_empty() && row[span.clone()].iter().any(|c| classify::has_rtl(&c.text)) {
            reorder::reorder_segment(row, span, &mut out);
        }
    }
    out
}
