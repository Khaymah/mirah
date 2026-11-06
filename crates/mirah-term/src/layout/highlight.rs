use super::align::marker_len;
use super::classify::{Kind, Palette, base, is_blank, is_frame};
use mirah_bidi::{Cell, Color, VisualRow};

const ACCENT: Color = Color::Rgb(217, 145, 104);
const BORDER: Color = Color::Rgb(110, 114, 124);
const MIN_LETTERS: usize = 2;
const TOP_LEFT: [char; 2] = ['┌', '╭'];

fn plain(cell: &Cell, text: Palette) -> bool {
    cell.style.fg == text.fg || cell.style.fg == Color::Default
}

fn is_heading(row: &[Cell]) -> bool {
    let letters: Vec<&Cell> = row.iter().filter(|c| base(c).is_alphabetic()).collect();
    letters.len() >= MIN_LETTERS
        && letters.iter().all(|c| c.style.bold)
        && !row.iter().any(|c| is_frame(base(c)))
}

fn paint_heading(visual: &mut VisualRow, text: Palette) {
    for cell in &mut visual.cells {
        if !is_blank(cell) && plain(cell, text) {
            cell.style.fg = ACCENT;
        }
    }
}

fn paint_marker(logical: &[Cell], visual: &mut VisualRow, text: Palette) {
    let indent = logical.iter().take_while(|c| is_blank(c)).count();
    let len = marker_len(logical, indent);
    for index in indent..indent + len {
        let Some(&at) = visual.logical_to_visual.get(index) else {
            continue;
        };
        if let Some(cell) = visual.cells.get_mut(at)
            && !is_blank(cell)
            && plain(cell, text)
        {
            cell.style.fg = ACCENT;
        }
    }
}

fn paint_table(visual: &mut VisualRow, header: bool, text: Palette) {
    for cell in &mut visual.cells {
        if is_blank(cell) || !plain(cell, text) {
            continue;
        }
        if is_frame(base(cell)) {
            cell.style.fg = BORDER;
        } else if header {
            cell.style.bold = true;
        }
    }
}

pub(super) fn paint(
    logical: &[Vec<Cell>],
    visual: &mut [VisualRow],
    kinds: &[Kind],
    text: Palette,
) {
    let mut header_next = false;
    for (i, row) in logical.iter().enumerate() {
        let Some(out) = visual.get_mut(i) else {
            continue;
        };
        let framed = row.iter().any(|c| is_frame(base(c)));
        let top = row.iter().any(|c| TOP_LEFT.contains(&base(c)));
        if framed {
            let has_text = row.iter().any(|c| !is_blank(c) && !is_frame(base(c)));
            paint_table(out, header_next && has_text, text);
            header_next = top || (header_next && !has_text);
            continue;
        }
        header_next = false;
        if kinds.get(i) != Some(&Kind::Prose) {
            continue;
        }
        if is_heading(row) {
            paint_heading(out, text);
        } else {
            paint_marker(row, out, text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mirah_bidi::Style;

    fn row(text: &str, bold: bool) -> Vec<Cell> {
        text.chars()
            .map(|c| Cell {
                text: c.to_string().into(),
                style: Style {
                    bold,
                    ..Style::default()
                },
                ..Cell::default()
            })
            .collect()
    }

    fn visual(row: &[Cell]) -> VisualRow {
        VisualRow {
            cells: row.to_vec(),
            logical_to_visual: (0..row.len()).collect(),
        }
    }

    fn palette() -> Palette {
        Palette {
            fg: Color::Default,
            bg: Color::Default,
        }
    }

    #[test]
    fn headings_markers_and_tables_get_colour() {
        let logical = vec![
            row("نظام الأنواع", true),
            row("- كلام عادي", false),
            row("┌────┬────┐", false),
            row("│ Go │ رست │", false),
            row("├────┼────┤", false),
            row("│ a  │ b  │", false),
        ];
        let mut out: Vec<VisualRow> = logical.iter().map(|r| visual(r)).collect();
        let kinds = [
            Kind::Prose,
            Kind::Prose,
            Kind::Framed,
            Kind::Framed,
            Kind::Framed,
            Kind::Framed,
        ];
        paint(&logical, &mut out, &kinds, palette());
        assert!(
            out[0]
                .cells
                .iter()
                .filter(|c| !is_blank(c))
                .all(|c| c.style.fg == ACCENT)
        );
        assert_eq!(out[1].cells[0].style.fg, ACCENT);
        assert_eq!(out[1].cells[2].style.fg, Color::Default);
        assert_eq!(out[2].cells[0].style.fg, BORDER);
        assert!(out[3].cells[2].style.bold);
        assert!(!out[5].cells[2].style.bold);
    }

    #[test]
    fn coloured_text_is_left_alone() {
        let mut heading = row("عنوان ملوّن", true);
        for cell in &mut heading {
            cell.style.fg = Color::Rgb(1, 2, 3);
        }
        let mut out = vec![visual(&heading)];
        paint(&[heading], &mut out, &[Kind::Prose], palette());
        assert!(
            out[0]
                .cells
                .iter()
                .all(|c| c.style.fg == Color::Rgb(1, 2, 3))
        );
    }
}
