mod align;
mod classify;
mod memo;
mod panes;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderOptions {
    pub shaping: Shaping,
    pub align: Align,
    pub input: InputOrder,
    pub tables: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            shaping: Shaping::default(),
            align: Align::default(),
            input: InputOrder::default(),
            tables: true,
        }
    }
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
    let cuts = panes::boundaries(&rows);
    let visual = if cuts.is_empty() {
        pane(rows, cursor, options, caches)
    } else {
        let width = rows.first().map_or(0, Vec::len);
        let count = rows.len();
        let parts = panes::ranges(&cuts, width)
            .into_iter()
            .map(|range| {
                let sliced = panes::slice(&rows, &range);
                let inner = panes::cursor_in(cursor, &range);
                let visual = pane(sliced, &inner, options, caches);
                (range, visual)
            })
            .collect();
        panes::stitch(parts, count)
    };
    caches.end_frame();
    visual
}

fn pane(
    rows: Vec<Vec<Cell>>,
    cursor: &Cursor,
    options: RenderOptions,
    caches: &mut Caches,
) -> Vec<VisualRow> {
    match options.input {
        InputOrder::Logical => {
            let logical = if options.align == Align::Off || !options.tables {
                rows
            } else {
                table::mirror_rtl_tables(rows, &[])
            };
            arrange(logical, cursor, options, &[], caches)
        }
        InputOrder::Visual => visual::layout(rows, cursor, options, caches),
    }
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
    let kinds = if options.align == Align::Off {
        vec![Kind::Fixed; logical.len()]
    } else {
        classify::classify(&logical, cursor, options.align)
    };
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
                rtl_base: options.align != Align::Off && kinds[i] != Kind::Code,
            };
            if !classify::has_rtl(row) {
                return visual_row_with(row, col, bidi);
            }
            caches
                .reorder
                .get_or(row, &(col, bidi), || visual_row_with(row, col, bidi))
        })
        .collect();
    align_rows(&logical, &mut visual, &kinds);
    visual
}

fn align_rows(logical: &[Vec<Cell>], visual: &mut [VisualRow], kinds: &[Kind]) {
    for (i, kind) in kinds.iter().enumerate() {
        match kind {
            Kind::Prose => align::align_prose(&logical[i], &mut visual[i]),
            Kind::Framed => align::align_inside_frames(&mut visual[i]),
            Kind::Fixed | Kind::Code => {}
        }
    }
}
