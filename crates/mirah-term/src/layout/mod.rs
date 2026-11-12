mod align;
mod classify;
mod highlight;
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
    pub highlight: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            shaping: Shaping::default(),
            align: Align::default(),
            input: InputOrder::default(),
            tables: true,
            highlight: true,
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
            if options.align == Align::Off || !options.tables {
                return arrange(rows, cursor, options, &[], caches);
            }
            let mirrored = table::mirror_tables(rows, &[]);
            let cursor = Cursor {
                col: moved_column(&mirrored.moved, cursor.row, cursor.col),
                ..*cursor
            };
            let mut visual = arrange(mirrored.rows, &cursor, options, &[], caches);
            compose(&mut visual, &mirrored.moved);
            visual
        }
        InputOrder::Visual => visual::layout(rows, cursor, options, caches),
    }
}

fn moved_column(moved: &[Vec<usize>], row: usize, col: usize) -> usize {
    moved
        .get(row)
        .and_then(|map| map.get(col))
        .copied()
        .unwrap_or(col)
}

fn compose(visual: &mut [VisualRow], first: &[Vec<usize>]) {
    for (row, first) in visual.iter_mut().zip(first) {
        row.logical_to_visual = first
            .iter()
            .map(|&middle| row.logical_to_visual.get(middle).copied().unwrap_or(middle))
            .collect();
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
    if options.highlight && options.align != Align::Off {
        highlight::paint(&logical, &mut visual, &kinds, classify::palette(&logical));
    }
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
