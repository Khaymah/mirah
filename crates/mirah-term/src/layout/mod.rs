mod align;
mod classify;
mod table;

use classify::Kind;
use mirah_bidi::{BidiOptions, Cell, Shaping, VisualRow, visual_row_with};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Prose,
    Arabic,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RenderOptions {
    pub shaping: Shaping,
    pub align: Align,
}

pub(crate) struct Cursor {
    pub(crate) row: usize,
    pub(crate) col: usize,
    pub(crate) visible: bool,
    pub(crate) editor: bool,
}

pub(crate) fn layout(
    logical: Vec<Vec<Cell>>,
    cursor: &Cursor,
    options: RenderOptions,
) -> Vec<VisualRow> {
    let logical = if options.align == Align::Off {
        logical
    } else {
        table::mirror_rtl_tables(logical)
    };
    let mut visual: Vec<VisualRow> = logical
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let col = (cursor.visible && i == cursor.row).then_some(cursor.col);
            let bidi = BidiOptions {
                shaping: options.shaping,
                rtl_base: options.align != Align::Off,
            };
            visual_row_with(row, col, bidi)
        })
        .collect();
    if options.align == Align::Off {
        return visual;
    }
    let kinds = classify::classify(&logical, cursor, options.align);
    for (i, kind) in kinds.into_iter().enumerate() {
        match kind {
            Kind::Prose => align::align_prose(&logical[i], &mut visual[i]),
            Kind::Framed => align::align_inside_frames(&mut visual[i]),
            Kind::Fixed => {}
        }
    }
    visual
}
