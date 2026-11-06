use super::classify::{base, blank_like, is_blank, is_frame, is_rtl};
use mirah_bidi::{Cell, VisualRow};

const MARKERS: [char; 12] = ['●', '•', '◦', '▪', '■', '○', '·', '-', '*', '⏺', '❯', '>'];
const FRAME_PADDING: usize = 1;

pub(super) fn marker_len(row: &[Cell], indent: usize) -> usize {
    let at = |i: usize| row.get(i).map(base);
    let blank_at = |i: usize| row.get(i).is_some_and(is_blank);
    if at(indent).is_some_and(|c| MARKERS.contains(&c)) && blank_at(indent + 1) {
        return 2;
    }
    let mut end = indent;
    while at(end).is_some_and(|c| c.is_ascii_digit()) {
        end += 1;
    }
    if end > indent && at(end).is_some_and(|c| c == '.' || c == ')') && blank_at(end + 1) {
        return end + 2 - indent;
    }
    0
}

pub(super) fn has_marker(row: &[Cell], indent: usize) -> bool {
    marker_len(row, indent) > 0
}

fn has_rtl(cells: &[Cell]) -> bool {
    cells.iter().any(|c| c.text.chars().any(is_rtl))
}

fn place(visual: &mut VisualRow, order: &[usize], start: usize) {
    let moved: Vec<Cell> = order.iter().map(|&old| visual.cells[old].clone()).collect();
    let blank = blank_like(visual.cells.last());
    for &old in order {
        visual.cells[old] = blank.clone();
    }
    let mut old_to_new: Vec<usize> = (0..visual.cells.len()).collect();
    let destination = start..start + order.len();
    let vacated = order.iter().filter(|old| !destination.contains(old));
    let displaced = destination.clone().filter(|d| !order.contains(d));
    for (from, to) in displaced.zip(vacated) {
        old_to_new[from] = *to;
    }
    for (offset, (cell, &old)) in moved.into_iter().zip(order).enumerate() {
        visual.cells[start + offset] = cell;
        old_to_new[old] = start + offset;
    }
    for target in &mut visual.logical_to_visual {
        *target = old_to_new.get(*target).copied().unwrap_or(*target);
    }
}

pub(super) fn align_prose(logical: &[Cell], visual: &mut VisualRow) {
    let width = visual.cells.len();
    let indent = logical.iter().take_while(|c| is_blank(c)).count();
    let Some(last) = logical.iter().rposition(|c| !is_blank(c)) else {
        return;
    };
    let last = visual
        .cells
        .iter()
        .rposition(|c| !is_blank(c))
        .map_or(last, |v| v.max(last));
    let marker = marker_len(logical, indent);
    let body_start = indent + marker;
    if body_start > last || indent >= width {
        return;
    }
    let body: Vec<usize> = (body_start..=last).collect();
    let markers: Vec<usize> = (indent..body_start).collect();
    let end = width - indent;
    let total = body.len() + markers.len();
    if total > end {
        return;
    }
    let order: Vec<usize> = if has_rtl(&logical[body_start..=last]) {
        body.into_iter().chain(markers.into_iter().rev()).collect()
    } else {
        markers.into_iter().chain(body).collect()
    };
    place(visual, &order, end - total);
}

fn has_gap(cells: &[Cell]) -> bool {
    cells.windows(2).any(|w| is_blank(&w[0]) && is_blank(&w[1]))
}

pub(super) fn align_inside_frames(visual: &mut VisualRow) {
    let frames: Vec<usize> = visual
        .cells
        .iter()
        .enumerate()
        .filter(|(_, c)| is_frame(base(c)))
        .map(|(i, _)| i)
        .collect();
    for pair in frames.windows(2) {
        let (left, right) = (pair[0], pair[1]);
        let inner = &visual.cells[left + 1..right];
        let (Some(first), Some(last)) = (
            inner.iter().position(|c| !is_blank(c)),
            inner.iter().rposition(|c| !is_blank(c)),
        ) else {
            continue;
        };
        let content = &inner[first..=last];
        if !content.iter().any(|c| c.text.chars().any(is_rtl)) || has_gap(content) {
            continue;
        }
        let (first, last) = (left + 1 + first, left + 1 + last);
        let len = last + 1 - first;
        let Some(start) = right.checked_sub(FRAME_PADDING + len) else {
            continue;
        };
        if start <= first {
            continue;
        }
        let order: Vec<usize> = (first..=last).collect();
        place(visual, &order, start);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(s: &str) -> Vec<Cell> {
        s.chars().map(|c| Cell::plain(c.to_string())).collect()
    }

    fn text(v: &VisualRow) -> String {
        v.cells.iter().map(|c| c.text.as_str()).collect()
    }

    #[test]
    fn detects_bullets_and_numbers() {
        assert_eq!(marker_len(&row("● hi"), 0), 2);
        assert_eq!(marker_len(&row("  12. hi"), 2), 4);
        assert_eq!(marker_len(&row("hi"), 0), 0);
        assert_eq!(marker_len(&row("-x"), 0), 0);
    }

    fn aligned(s: &str) -> String {
        let logical = row(s);
        let mut v = VisualRow {
            cells: logical.clone(),
            logical_to_visual: (0..logical.len()).collect(),
        };
        align_prose(&logical, &mut v);
        text(&v)
    }

    #[test]
    fn ltr_prose_moves_right_with_its_marker_on_the_left() {
        assert_eq!(aligned("● ab      "), "      ● ab");
    }

    #[test]
    fn rtl_prose_moves_right_with_its_marker_on_the_right() {
        assert_eq!(aligned("● نص      "), "      نص ●");
    }

    #[test]
    fn prose_wider_than_the_row_stays() {
        assert_eq!(aligned("● abc"), "● abc");
    }

    #[test]
    fn indentation_becomes_a_right_margin() {
        let logical = row("  نص    ");
        let mut v = VisualRow {
            cells: logical.clone(),
            logical_to_visual: (0..logical.len()).collect(),
        };
        align_prose(&logical, &mut v);
        assert_eq!(text(&v), "    نص  ");
        assert_eq!(v.logical_to_visual[2], 4);
    }

    #[test]
    fn rtl_text_moves_right_inside_its_frame() {
        let cells = row("│ نص      │");
        let mut v = VisualRow {
            cells: cells.clone(),
            logical_to_visual: (0..cells.len()).collect(),
        };
        align_inside_frames(&mut v);
        assert_eq!(text(&v), "│      نص │");
    }

    #[test]
    fn english_inside_frames_stays() {
        let cells = row("│ hi      │");
        let mut v = VisualRow {
            cells: cells.clone(),
            logical_to_visual: (0..cells.len()).collect(),
        };
        align_inside_frames(&mut v);
        assert_eq!(text(&v), "│ hi      │");
    }
}
