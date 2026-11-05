mod cell;
mod classify;
mod mirror;
mod reorder;
mod segment;
mod shape;

pub use cell::{Cell, Color, Style, VisualRow};
pub use shape::{Shaped, shape};
use std::ops::Range;

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
    for span in rtl_segments(row, cursor, Edge::Strong) {
        reorder::reorder_segment(row, span, options, &mut out);
    }
    out
}

pub fn logical_row(visual: &[Cell], cursor: Option<usize>) -> VisualRow {
    let mut out = VisualRow::identity(visual);
    for span in rtl_segments(visual, cursor, Edge::Attached) {
        reorder::unreorder_segment(visual, span, &mut out);
    }
    out
}

#[derive(Clone, Copy)]
enum Edge {
    Strong,
    Attached,
}

fn rtl_segments(row: &[Cell], cursor: Option<usize>, edge: Edge) -> Vec<Range<usize>> {
    if !row.iter().any(|c| classify::has_rtl(&c.text)) {
        return Vec::new();
    }
    let layout = segment::Row { cells: row, cursor };
    layout
        .segments()
        .into_iter()
        .map(|span| match edge {
            Edge::Strong => layout.trim(span),
            Edge::Attached => layout.trim_attached(span),
        })
        .filter(|span| {
            !span.is_empty() && row[span.clone()].iter().any(|c| classify::has_rtl(&c.text))
        })
        .collect()
}
