use super::{Align, Cursor, RenderOptions, arrange, table, unchanged};
use mirah_bidi::{Cell, VisualRow, logical_row, mirror_reversed_brackets};

pub(super) fn layout(
    rows: Vec<Vec<Cell>>,
    cursor: &Cursor,
    options: RenderOptions,
) -> Vec<VisualRow> {
    if options.align == Align::Off {
        return rows.iter().map(|row| unchanged(row)).collect();
    }
    let mut settled = Vec::with_capacity(rows.len());
    let mut from_visual = Vec::with_capacity(rows.len());
    let mut logical = Vec::with_capacity(rows.len());
    let hints = table::frame_hints(&rows);
    for (row, hint) in rows.iter().zip(&hints) {
        match logical_row(row, hint.as_deref()) {
            Some(undone) => {
                settled.push(false);
                from_visual.push(undone.from_visual);
                logical.push(undone.cells);
            }
            None => {
                settled.push(true);
                from_visual.push((0..row.len()).collect());
                logical.push(mirror_reversed_brackets(row));
            }
        }
    }
    let logical_cursor = Cursor {
        col: from_visual
            .get(cursor.row)
            .and_then(|map: &Vec<usize>| map.get(cursor.col).copied())
            .unwrap_or(cursor.col),
        ..*cursor
    };
    let logical = table::mirror_rtl_tables(logical, &settled);
    let mut visual = arrange(logical, &logical_cursor, options, &settled);
    for (row, first) in visual.iter_mut().zip(&from_visual) {
        row.logical_to_visual = first
            .iter()
            .map(|&middle| row.logical_to_visual.get(middle).copied().unwrap_or(middle))
            .collect();
    }
    visual
}
