mod align;
mod classify;
mod memo;
mod table;
mod visual;

use classify::Kind;
pub(crate) use memo::Caches;
use mirah_bidi::{BidiOptions, Cell, Shaping, VisualRow, visual_row_with};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Prose,
    Arabic,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputOrder {
    #[default]
    Logical,
    Visual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RenderOptions {
    pub shaping: Shaping,
    pub align: Align,
    pub input: InputOrder,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Cursor {
    pub(crate) row: usize,
    pub(crate) col: usize,
    pub(crate) visible: bool,
    pub(crate) editor: bool,
}

pub(crate) fn layout(
    rows: Vec<Vec<Cell>>,
    cursor: &Cursor,
    options: RenderOptions,
    caches: &mut Caches,
) -> Vec<VisualRow> {
    let visual = match options.input {
        InputOrder::Logical => {
            let logical = if options.align == Align::Off {
                rows
            } else {
                table::mirror_rtl_tables(rows, &[])
            };
            arrange(logical, cursor, options, &[], caches)
        }
        InputOrder::Visual => visual::layout(rows, cursor, options, caches),
    };
    caches.end_frame();
    visual
}

fn unchanged(row: &[Cell]) -> VisualRow {
    VisualRow {
        cells: row.to_vec(),
        logical_to_visual: (0..row.len()).collect(),
    }
}

fn arrange(
    logical: Vec<Vec<Cell>>,
    cursor: &Cursor,
    options: RenderOptions,
    settled: &[bool],
    caches: &mut Caches,
) -> Vec<VisualRow> {
    let mut visual: Vec<VisualRow> = logical
        .iter()
        .enumerate()
        .map(|(i, row)| {
            if settled.get(i) == Some(&true) {
                return unchanged(row);
            }
            let col = (cursor.visible && i == cursor.row).then_some(cursor.col);
            let bidi = BidiOptions {
                shaping: options.shaping,
                rtl_base: options.align != Align::Off,
            };
            if !classify::has_rtl(row) {
                return visual_row_with(row, col, bidi);
            }
            caches
                .reorder
                .get_or(row, &(col, bidi), || visual_row_with(row, col, bidi))
        })
        .collect();
    align_rows(&logical, &mut visual, cursor, options.align);
    visual
}

fn align_rows(logical: &[Vec<Cell>], visual: &mut [VisualRow], cursor: &Cursor, align: Align) {
    if align == Align::Off {
        return;
    }
    let kinds = classify::classify(logical, cursor, align);
    for (i, kind) in kinds.into_iter().enumerate() {
        match kind {
            Kind::Prose => align::align_prose(&logical[i], &mut visual[i]),
            Kind::Framed => align::align_inside_frames(&mut visual[i]),
            Kind::Fixed => {}
        }
    }
}
