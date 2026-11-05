mod brackets;
mod cell;
mod classify;
mod mirror;
mod reorder;
mod segment;
mod shape;
mod unreorder;

pub use brackets::mirror_reversed_brackets;
pub use cell::{Cell, Color, Style, VisualRow};
pub use shape::{Shaped, shape};
pub use unreorder::{LogicalRow, logical_row};

pub fn visual_row(row: &[Cell]) -> VisualRow {
    visual_row_with_cursor(row, None)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Shaping {
    #[default]
    Full,
    Plain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BidiOptions {
    pub shaping: Shaping,
    pub rtl_base: bool,
}

pub fn visual_row_with_cursor(row: &[Cell], cursor: Option<usize>) -> VisualRow {
    visual_row_with(row, cursor, BidiOptions::default())
}

pub fn visual_row_with(row: &[Cell], cursor: Option<usize>, options: BidiOptions) -> VisualRow {
    let mut out = VisualRow::identity(row);
    if !row.iter().any(|c| classify::has_rtl(&c.text)) {
        return out;
    }
    let layout = segment::Row { cells: row, cursor };
    for span in layout.segments() {
        let span = layout.trim(span);
        if !span.is_empty() && row[span.clone()].iter().any(|c| classify::has_rtl(&c.text)) {
            reorder::reorder_segment(row, span, options, &mut out);
        }
    }
    out
}
