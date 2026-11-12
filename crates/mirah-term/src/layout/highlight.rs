use super::align::marker_len;
use super::classify::{Kind, Palette, base, has_rtl, is_blank};
use mirah_bidi::{Cell, Color, VisualRow};

const ACCENT: Color = Color::Rgb(217, 145, 104);
const BORDER: Color = Color::Rgb(110, 114, 124);
const MIN_LETTERS: usize = 2;
const TOP_LEFT: [char; 2] = ['┌', '╭'];
const RULE_CHARS: [char; 4] = ['-', '─', '—', '━'];
const MIN_RULE: usize = 3;
const JOINS: [char; 9] = ['┬', '┼', '┴', '╤', '╪', '╧', '╦', '╬', '╩'];
const BARS: [char; 3] = ['│', '┃', '║'];
const MIN_BARS: usize = 3;

fn is_line(c: char) -> bool {
    ('\u{2500}'..='\u{257F}').contains(&c)
}

fn is_table_row(row: &[Cell]) -> bool {
    row.iter().any(|c| JOINS.contains(&base(c)))
        || row.iter().filter(|c| BARS.contains(&base(c))).count() >= MIN_BARS
}

fn plain(cell: &Cell, text: Palette) -> bool {
    cell.style.fg == text.fg || cell.style.fg == Color::Default
}

fn is_heading(row: &[Cell]) -> bool {
    let letters: Vec<&Cell> = row.iter().filter(|c| base(c).is_alphabetic()).collect();
    letters.len() >= MIN_LETTERS
        && letters.iter().all(|c| c.style.bold)
        && !row.iter().any(|c| is_line(base(c)))
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
        if is_line(base(cell)) {
            cell.style.fg = BORDER;
        } else if header {
            cell.style.bold = true;
            cell.style.fg = ACCENT;
        }
    }
}

fn is_rule(row: &[Cell]) -> bool {
    let content: Vec<char> = row.iter().filter(|c| !is_blank(c)).map(base).collect();
    content.len() >= MIN_RULE && content.iter().all(|c| RULE_CHARS.contains(c))
}

fn paint_rule(visual: &mut VisualRow, text: Palette) {
    for cell in &mut visual.cells {
        if !is_blank(cell) && plain(cell, text) {
            cell.style.fg = BORDER;
        }
    }
}

fn paint_bold(visual: &mut VisualRow, text: Palette) {
    for cell in &mut visual.cells {
        if cell.style.bold && !is_blank(cell) && plain(cell, text) {
            cell.style.fg = ACCENT;
        }
    }
}

pub(super) fn paint(
    logical: &[Vec<Cell>],
    visual: &mut [VisualRow],
    kinds: &[Kind],
    text: Palette,
) {
    if !logical.iter().any(|row| has_rtl(row)) {
        return;
    }
    let mut header_next = false;
    for (i, row) in logical.iter().enumerate() {
        let Some(out) = visual.get_mut(i) else {
            continue;
        };
        if is_rule(row) {
            header_next = false;
            paint_rule(out, text);
            continue;
        }
        if row.iter().any(|c| is_line(base(c))) {
            if is_table_row(row) {
                let top = row.iter().any(|c| TOP_LEFT.contains(&base(c)));
                let has_text = row.iter().any(|c| !is_blank(c) && !is_line(base(c)));
                paint_table(out, header_next && has_text, text);
                header_next = top || (header_next && !has_text);
            } else {
                header_next = false;
            }
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
            paint_bold(out, text);
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
        assert_eq!(out[3].cells[2].style.fg, ACCENT);
        assert!(!out[5].cells[2].style.bold);
    }

    #[test]
    fn screens_without_arabic_and_boxes_that_are_not_tables_keep_their_colours() {
        let english = vec![row("Title", true), row("┌──┬──┐", false)];
        let mut out: Vec<VisualRow> = english.iter().map(|r| visual(r)).collect();
        paint(&english, &mut out, &[Kind::Prose, Kind::Framed], palette());
        assert!(
            out.iter()
                .flat_map(|r| &r.cells)
                .all(|c| c.style.fg == Color::Default)
        );
        let mixed = vec![
            row("مرحبا", false),
            row("╭──────╮", false),
            row("│ > hi │", false),
            row("├── src", false),
            row("████░░", false),
        ];
        let mut out: Vec<VisualRow> = mixed.iter().map(|r| visual(r)).collect();
        paint(
            &mixed,
            &mut out,
            &[
                Kind::Prose,
                Kind::Framed,
                Kind::Framed,
                Kind::Framed,
                Kind::Fixed,
            ],
            palette(),
        );
        assert!(
            out.iter()
                .flat_map(|r| &r.cells)
                .all(|c| c.style.fg == Color::Default)
        );
    }

    #[test]
    fn bold_words_and_rules_get_colour() {
        let mut line = row("هنا Rust مهم", false);
        for cell in &mut line[4..8] {
            cell.style.bold = true;
        }
        let rule = row("---", false);
        let logical = vec![line, rule];
        let mut out: Vec<VisualRow> = logical.iter().map(|r| visual(r)).collect();
        paint(&logical, &mut out, &[Kind::Prose, Kind::Fixed], palette());
        assert_eq!(out[0].cells[4].style.fg, ACCENT);
        assert_eq!(out[0].cells[0].style.fg, Color::Default);
        assert_eq!(out[1].cells[0].style.fg, BORDER);
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
