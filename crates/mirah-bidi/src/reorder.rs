use crate::cell::{Cell, Style, VisualRow};
use crate::classify::{is_rtl, is_strong};
use crate::mirror::mirror;
use crate::shape::{Shaped, shape};
use crate::{BidiOptions, Shaping};
use std::ops::Range;
use unicode_bidi::{BidiInfo, Level};

const PAD: &str = " ";

struct Item {
    column: usize,
    glyph: String,
    style: Style,
}

fn plain_items(row: &[Cell], span: &Range<usize>) -> Vec<Item> {
    span.clone()
        .map(|column| Item {
            column,
            glyph: row[column].text.clone(),
            style: row[column].style,
        })
        .collect()
}

fn shaped_items(row: &[Cell], span: &Range<usize>) -> Vec<Item> {
    let texts: Vec<String> = row[span.clone()].iter().map(|c| c.text.clone()).collect();
    let shaped = shape(&texts);
    let mut items = Vec::with_capacity(span.len());
    let mut pads = Vec::new();
    for (offset, glyph) in shaped.iter().enumerate() {
        let column = span.start + offset;
        match glyph {
            Shaped::Glyph(glyph) => {
                let mut style = row[column].style;
                if matches!(shaped.get(offset + 1), Some(Shaped::Absorbed)) {
                    style.inverse |= row[column + 1].style.inverse;
                }
                items.push(Item {
                    column,
                    glyph: glyph.clone(),
                    style,
                });
            }
            Shaped::Absorbed => pads.push(column),
        }
    }
    items.extend(pads.into_iter().map(|column| Item {
        column,
        glyph: PAD.to_string(),
        style: Style {
            inverse: false,
            ..row[column].style
        },
    }));
    items
}

fn bidi_char(row: &[Cell], item: &Item) -> char {
    if item.glyph == PAD || row[item.column].text.is_empty() {
        ' '
    } else {
        row[item.column].base()
    }
}

fn visual_order(row: &[Cell], items: &[Item], base: Option<Level>) -> Vec<(usize, bool)> {
    let mut text = String::new();
    let mut starts = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        starts.push((text.len(), index));
        text.push(bidi_char(row, item));
    }
    let info = BidiInfo::new(&text, base);
    let Some(para) = info.paragraphs.first() else {
        return Vec::new();
    };
    let (levels, runs) = info.visual_runs(para, para.range.clone());
    let mut order = Vec::with_capacity(items.len());
    for run in runs {
        let rtl = levels[run.start].is_rtl();
        let mut in_run: Vec<usize> = starts
            .iter()
            .filter(|(byte, _)| run.contains(byte))
            .map(|(_, index)| *index)
            .collect();
        if rtl {
            in_run.reverse();
        }
        order.extend(in_run.into_iter().map(|index| (index, rtl)));
    }
    order
}

pub(crate) fn reorder_segment(
    row: &[Cell],
    span: Range<usize>,
    options: BidiOptions,
    out: &mut VisualRow,
) {
    let items = match options.shaping {
        Shaping::Full => shaped_items(row, &span),
        Shaping::Plain => plain_items(row, &span),
    };
    let base = options.rtl_base.then(Level::rtl);
    for (slot, (index, rtl)) in visual_order(row, &items, base).into_iter().enumerate() {
        let item = &items[index];
        let visual_column = span.start + slot;
        let text = if rtl {
            item.glyph.chars().map(mirror).collect()
        } else {
            item.glyph.clone()
        };
        out.cells[visual_column] = Cell {
            text,
            style: item.style,
            wide: false,
            continuation: false,
        };
        out.logical_to_visual[item.column] = visual_column;
    }
}

fn ends_right_to_left(row: &[Cell], span: &Range<usize>) -> bool {
    row[span.clone()]
        .iter()
        .rev()
        .map(Cell::base)
        .find(|c| is_strong(*c))
        .is_some_and(is_rtl)
}

pub(crate) fn unreorder_segment(row: &[Cell], span: Range<usize>, out: &mut VisualRow) {
    let items = plain_items(row, &span);
    let base = if ends_right_to_left(row, &span) {
        Level::rtl()
    } else {
        Level::ltr()
    };
    for (slot, (index, _)) in visual_order(row, &items, Some(base))
        .into_iter()
        .enumerate()
    {
        let item = &items[index];
        let column = span.start + slot;
        out.cells[column] = row[item.column].clone();
        out.logical_to_visual[item.column] = column;
    }
}
