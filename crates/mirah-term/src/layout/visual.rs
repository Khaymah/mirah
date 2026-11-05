use super::{Align, Cursor, RenderOptions, arrange};
use mirah_bidi::{Cell, VisualRow, logical_row};

fn identity(row: Vec<Cell>) -> VisualRow {
    VisualRow {
        logical_to_visual: (0..row.len()).collect(),
        cells: row,
    }
}

pub(super) fn layout(
    rows: Vec<Vec<Cell>>,
    cursor: &Cursor,
    options: RenderOptions,
) -> Vec<VisualRow> {
    if options.align == Align::Off {
        return rows.into_iter().map(identity).collect();
    }
    let undone: Vec<VisualRow> = rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            logical_row(
                row,
                (cursor.visible && i == cursor.row).then_some(cursor.col),
            )
        })
        .collect();
    let logical_cursor = Cursor {
        col: undone
            .get(cursor.row)
            .and_then(|row| row.logical_to_visual.get(cursor.col).copied())
            .unwrap_or(cursor.col),
        ..*cursor
    };
    let logical = undone.iter().map(|row| row.cells.clone()).collect();
    let mut visual = arrange(logical, &logical_cursor, options);
    for (row, first) in visual.iter_mut().zip(&undone) {
        row.logical_to_visual = first
            .logical_to_visual
            .iter()
            .map(|&middle| row.logical_to_visual.get(middle).copied().unwrap_or(middle))
            .collect();
    }
    visual
}
