use crate::brackets::reversed_pairs;
use crate::cell::Cell;
use crate::classify::{MIN_GAP_COLUMNS, has_rtl, is_frame};
use crate::mirror::mirror;
use std::ops::Range;
use unicode_bidi::{BidiInfo, Level};

const SENTENCE_END: &str = ".:!?؟،؛";
const STRONG_RTL: char = '\u{05D0}';
const MARKERS: &str = "●•◦-*⎿❯>";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicalRow {
    pub cells: Vec<Cell>,
    pub from_visual: Vec<usize>,
}

struct Item {
    column: usize,
    width: usize,
}

fn is_blank(cell: &Cell) -> bool {
    !cell.continuation && cell.is_blank()
}

fn trimmed(row: &[Cell], span: Range<usize>) -> Range<usize> {
    let (mut start, mut end) = (span.start, span.end);
    while start < end && is_blank(&row[start]) {
        start += 1;
    }
    while end > start && is_blank(&row[end - 1]) {
        end -= 1;
    }
    start..end
}

fn whole(row: &[Cell]) -> Vec<Range<usize>> {
    let span = trimmed(row, 0..row.len());
    if span.is_empty() {
        Vec::new()
    } else {
        vec![span]
    }
}

fn split_at_gaps(row: &[Cell]) -> Vec<Range<usize>> {
    let mut units = Vec::new();
    let (mut start, mut i) = (0, 0);
    while i < row.len() {
        if !is_blank(&row[i]) {
            i += 1;
            continue;
        }
        let gap = i;
        while i < row.len() && is_blank(&row[i]) {
            i += 1;
        }
        if i - gap >= MIN_GAP_COLUMNS {
            units.push(trimmed(row, start..gap));
            start = i;
        }
    }
    units.push(trimmed(row, start..row.len()));
    units.retain(|u| !u.is_empty());
    units
}

fn split_markers(row: &[Cell], units: Vec<Range<usize>>) -> Vec<Range<usize>> {
    let mut out = Vec::with_capacity(units.len() + 1);
    for unit in units {
        let marker = MARKERS.contains(row[unit.start].base())
            && row.get(unit.start + 1).is_some_and(is_blank);
        if marker && unit.len() > 2 {
            out.push(unit.start..unit.start + 1);
            out.push(trimmed(row, unit.start + 1..unit.end));
        } else {
            out.push(unit);
        }
    }
    out
}

fn display_order(chars: &[char], base: Option<Level>) -> Vec<usize> {
    let text: String = chars.iter().collect();
    let starts: Vec<usize> = text.char_indices().map(|(byte, _)| byte).collect();
    let info = BidiInfo::new(&text, base);
    let Some(para) = info.paragraphs.first() else {
        return (0..chars.len()).collect();
    };
    let (levels, runs) = info.visual_runs(para, para.range.clone());
    let mut order = Vec::with_capacity(chars.len());
    for run in runs {
        let first = starts.partition_point(|&b| b < run.start);
        let last = starts.partition_point(|&b| b < run.end);
        if levels[run.start].is_rtl() {
            order.extend((first..last).rev());
        } else {
            order.extend(first..last);
        }
    }
    order
}

fn balanced(logical: impl Iterator<Item = char>) -> bool {
    let mut open = Vec::new();
    for c in logical {
        match c {
            '(' | '[' | '{' => open.push(c),
            ')' | ']' | '}' if open.pop().map(mirror) != Some(c) => return false,
            _ => {}
        }
    }
    open.is_empty()
}

fn score(chars: &[char], order: &[usize]) -> i32 {
    let logical = order.iter().map(|&i| chars[i]);
    let brackets = if balanced(logical.clone()) { 2 } else { 0 };
    let first = logical.clone().find(|c| *c != ' ');
    let last = logical.rev().find(|c| *c != ' ');
    brackets + i32::from(last.is_some_and(|c| SENTENCE_END.contains(c)))
        - i32::from(first.is_some_and(|c| SENTENCE_END.contains(c)))
}

fn candidates(chars: &[char]) -> Vec<Vec<usize>> {
    let mirrored: Vec<char> = chars.iter().copied().map(mirror).collect();
    let mut paired = chars.to_vec();
    for (open, close) in reversed_pairs(chars) {
        paired[open] = STRONG_RTL;
        paired[close] = STRONG_RTL;
    }
    let mut texts: Vec<&[char]> = vec![&paired];
    for text in [&mirrored[..], chars] {
        if !texts.contains(&text) {
            texts.push(text);
        }
    }
    let mut found: Vec<(i32, Vec<usize>)> = Vec::new();
    let mut tried: Vec<Vec<usize>> = Vec::new();
    for base in [Level::rtl(), Level::ltr()] {
        for &text in &texts {
            let order = display_order(text, Some(base));
            if tried.contains(&order) {
                continue;
            }
            let logical: Vec<char> = order.iter().map(|&i| chars[i]).collect();
            let shown = display_order(&logical, None);
            let reproduces = shown.len() == order.len()
                && shown.iter().enumerate().all(|(slot, &j)| order[j] == slot);
            if reproduces {
                found.push((score(chars, &order), order.clone()));
            }
            tried.push(order);
        }
    }
    found.sort_by_key(|(rank, _)| -rank);
    found.into_iter().map(|(_, order)| order).collect()
}

fn place(
    visual: &[Cell],
    span: &Range<usize>,
    items: &[Item],
    order: &[usize],
    out: &mut LogicalRow,
) {
    let mut column = span.start;
    for &index in order {
        let item = &items[index];
        for offset in 0..item.width {
            let from = item.column + offset;
            let to = column + offset;
            if let (Some(cell), Some(target)) = (visual.get(from), out.cells.get_mut(to)) {
                *target = cell.clone();
                out.from_visual[from] = to;
            }
        }
        column += item.width;
    }
}

fn lands_on_frames(
    visual: &[Cell],
    span: &Range<usize>,
    items: &[Item],
    order: &[usize],
    frames: &[usize],
) -> bool {
    let mut landed = vec![false; span.len()];
    let mut column = span.start;
    for &index in order {
        let item = &items[index];
        if let Some(slot) = landed.get_mut(column - span.start) {
            *slot = is_frame(visual[item.column].base());
        }
        column += item.width;
    }
    frames.iter().all(|&c| {
        if span.contains(&c) {
            landed[c - span.start]
        } else {
            visual.get(c).is_some_and(|cell| is_frame(cell.base()))
        }
    })
}

fn apply(visual: &[Cell], units: &[Range<usize>], frames: Option<&[usize]>) -> Option<LogicalRow> {
    let mut out = LogicalRow {
        cells: visual.to_vec(),
        from_visual: (0..visual.len()).collect(),
    };
    for span in units {
        if !visual[span.clone()].iter().any(|c| has_rtl(&c.text)) {
            continue;
        }
        let items: Vec<Item> = span
            .clone()
            .filter(|&c| !visual[c].continuation)
            .map(|column| Item {
                column,
                width: if visual[column].wide { 2 } else { 1 },
            })
            .collect();
        let chars: Vec<char> = items.iter().map(|i| visual[i.column].base()).collect();
        let orders = candidates(&chars);
        let chosen = match frames {
            Some(frames) if units.len() == 1 && orders.len() > 1 => orders
                .iter()
                .position(|order| lands_on_frames(visual, span, &items, order, frames))
                .unwrap_or(0),
            _ => 0,
        };
        place(visual, span, &items, orders.get(chosen)?, &mut out);
    }
    Some(out)
}

pub fn logical_row(visual: &[Cell], frames: Option<&[usize]>) -> Option<LogicalRow> {
    if !visual.iter().any(|c| has_rtl(&c.text)) {
        return apply(visual, &[], None);
    }
    let framed = visual.iter().any(|c| is_frame(c.base()));
    let (first, second) = if framed {
        (whole(visual), split_at_gaps(visual))
    } else {
        (split_at_gaps(visual), whole(visual))
    };
    let attempts = [split_markers(visual, first), split_markers(visual, second)];
    attempts
        .iter()
        .find_map(|units| apply(visual, units, frames))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(s: &str) -> Vec<Cell> {
        s.chars().map(|c| Cell::plain(c.to_string())).collect()
    }

    fn logical(s: &str) -> Option<String> {
        logical_row(&row(s), None).map(|r| r.cells.iter().map(|c| c.text.as_str()).collect())
    }

    #[test]
    fn undoes_a_right_to_left_line() {
        assert_eq!(logical(".)نينتا( فلملا").as_deref(), Some("الملف (اتنين)."));
    }

    #[test]
    fn keeps_a_left_to_right_line_that_ends_a_sentence() {
        assert_eq!(
            logical("- `unwrap` تارابتخالا ارب.").as_deref(),
            Some("- `unwrap` برا الاختبارات.")
        );
    }

    #[test]
    fn undoes_a_whole_table_row_the_program_reversed() {
        assert_eq!(logical("│ id │ مسالا │").as_deref(), Some("│ الاسم │ id │"));
    }

    fn in_table(s: &str, frames: &[usize]) -> Option<String> {
        logical_row(&row(s), Some(frames))
            .map(|r| r.cells.iter().map(|c| c.text.as_str()).collect())
    }

    #[test]
    fn table_columns_pick_between_two_valid_readings() {
        assert_eq!(
            in_table("│ Go │ ةكبش │", &[0, 5, 12]).as_deref(),
            Some("│ Go │ شبكة │")
        );
        assert_eq!(
            in_table("│ id │ مسالا │", &[0, 8, 13]).as_deref(),
            Some("│ الاسم │ id │")
        );
    }

    #[test]
    fn prefers_the_reading_with_balanced_brackets() {
        assert_eq!(
            logical("wrapper — )mirah( ةآرم").as_deref(),
            Some("مرآة (mirah) — wrapper")
        );
        assert_eq!(
            logical("BDSM (CSI 8 l) ـب لافقإ").as_deref(),
            Some("إقفال بـ BDSM (CSI 8 l)")
        );
    }

    #[test]
    fn keeps_a_marker_in_its_own_column() {
        assert_eq!(logical("●  فلملا حتفا").as_deref(), Some("●  افتح الملف"));
    }

    #[test]
    fn keeps_a_bullet_and_a_prompt_at_the_start() {
        assert_eq!(
            logical("● .)نينتا( فلملا").as_deref(),
            Some("● الملف (اتنين).")
        );
        assert_eq!(logical("❯ بتكا").as_deref(), Some("❯ اكتب"));
    }

    #[test]
    fn maps_each_visual_column_to_its_logical_column() {
        let r = logical_row(&row("بتكا"), None).expect("reversible");
        assert_eq!(r.from_visual, [3, 2, 1, 0]);
    }

    #[test]
    fn rows_without_arabic_stay() {
        assert_eq!(logical("ls -la").as_deref(), Some("ls -la"));
    }
}
