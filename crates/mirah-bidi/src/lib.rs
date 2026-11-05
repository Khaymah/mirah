mod cell;
mod classify;
mod mirror;
mod reorder;
mod segment;
mod shape;

pub use cell::{Cell, Color, Style, VisualRow};
pub use shape::{Shaped, shape};

pub fn visual_row(row: &[Cell]) -> VisualRow {
    let mut out = VisualRow::identity(row);
    if !row.iter().any(|c| classify::has_rtl(&c.text)) {
        return out;
    }
    for span in segment::segments(row) {
        let span = segment::trim(row, span);
        if !span.is_empty() && row[span.clone()].iter().any(|c| classify::has_rtl(&c.text)) {
            reorder::reorder_segment(row, span, &mut out);
        }
    }
    out
}
