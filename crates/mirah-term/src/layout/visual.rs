use super::{Align, Caches, Cursor, RenderOptions, arrange, table, unchanged};
use mirah_bidi::{Cell, VisualRow, logical_row, mirror_reversed_brackets};

struct Undone {
    settled: Vec<bool>,
    from_visual: Vec<Vec<usize>>,
    logical: Vec<Vec<Cell>>,
}

fn undo(rows: &[Vec<Cell>], caches: &mut Caches) -> Undone {
    let mut out = Undone {
        settled: Vec::with_capacity(rows.len()),
        from_visual: Vec::with_capacity(rows.len()),
        logical: Vec::with_capacity(rows.len()),
    };
    let hints = table::frame_hints(rows);
    for (row, hint) in rows.iter().zip(&hints) {
        let undone = if super::classify::has_rtl(row) {
            caches
                .unreorder
                .get_or(row, hint, || logical_row(row, hint.as_deref()))
        } else {
            logical_row(row, hint.as_deref())
        };
        match undone {
            Some(undone) => {
                out.settled.push(false);
                out.from_visual.push(undone.from_visual);
                out.logical.push(undone.cells);
            }
            None => {
                out.settled.push(true);
                out.from_visual.push((0..row.len()).collect());
                out.logical.push(mirror_reversed_brackets(row));
            }
        }
    }
    out
}

pub(super) fn layout(
    rows: Vec<Vec<Cell>>,
    cursor: &Cursor,
    options: RenderOptions,
    caches: &mut Caches,
) -> Vec<VisualRow> {
    if options.align == Align::Off {
        return rows.iter().map(|row| unchanged(row)).collect();
    }
    let Undone {
        settled,
        from_visual,
        logical,
    } = undo(&rows, caches);
    let logical_cursor = Cursor {
        col: from_visual
            .get(cursor.row)
            .and_then(|map: &Vec<usize>| map.get(cursor.col).copied())
            .unwrap_or(cursor.col),
        ..*cursor
    };
    let logical = if options.tables {
        table::mirror_rtl_tables(logical, &settled)
    } else {
        logical
    };
    let mut visual = arrange(logical, &logical_cursor, options, &settled, caches);
    for (row, first) in visual.iter_mut().zip(&from_visual) {
        row.logical_to_visual = first
            .iter()
            .map(|&middle| row.logical_to_visual.get(middle).copied().unwrap_or(middle))
            .collect();
    }
    visual
}
