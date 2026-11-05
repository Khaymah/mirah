use crate::shape::{Shaped, shape};
use unicode_bidi::{BidiClass, BidiInfo, bidi_class};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Style {
    pub fg: vt100::Color,
    pub bg: vt100::Color,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub inverse: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cell {
    pub text: String,
    pub style: Style,
    pub wide: bool,
    pub continuation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualRow {
    pub cells: Vec<Cell>,
    pub logical_to_visual: Vec<usize>,
}

fn base(cell: &Cell) -> char {
    cell.text.chars().next().unwrap_or(' ')
}

fn is_rtl(c: char) -> bool {
    matches!(bidi_class(c), BidiClass::R | BidiClass::AL)
}

fn is_strong(c: char) -> bool {
    matches!(bidi_class(c), BidiClass::L | BidiClass::R | BidiClass::AL)
}

fn is_blank(cell: &Cell) -> bool {
    base(cell) == ' '
}

fn is_frame(c: char) -> bool {
    ('\u{2500}'..='\u{259F}').contains(&c)
}

fn mirror(c: char) -> char {
    match c {
        '(' => ')',
        ')' => '(',
        '[' => ']',
        ']' => '[',
        '{' => '}',
        '}' => '{',
        '<' => '>',
        '>' => '<',
        '«' => '»',
        '»' => '«',
        _ => c,
    }
}

fn fixed_columns(row: &[Cell]) -> Vec<bool> {
    let mut fixed: Vec<bool> = row
        .iter()
        .map(|c| c.wide || c.continuation || is_frame(base(c)))
        .collect();
    let mut i = 0;
    while i < row.len() {
        if is_blank(&row[i]) {
            let start = i;
            while i < row.len() && is_blank(&row[i]) {
                i += 1;
            }
            if i - start >= 2 {
                fixed[start..i].iter_mut().for_each(|f| *f = true);
            }
        } else {
            i += 1;
        }
    }
    fixed
}

fn segments(row: &[Cell]) -> Vec<(usize, usize)> {
    let fixed = fixed_columns(row);
    let mut out = Vec::new();
    let mut i = 0;
    while i < row.len() {
        if fixed[i] {
            i += 1;
            continue;
        }
        let start = i;
        let key = (row[i].style.bg, row[i].style.inverse);
        while i < row.len() && !fixed[i] && (row[i].style.bg, row[i].style.inverse) == key {
            i += 1;
        }
        out.push((start, i));
    }
    out
}

fn trim(row: &[Cell], (mut a, mut b): (usize, usize)) -> (usize, usize) {
    while a < b && !is_strong(base(&row[a])) {
        a += 1;
    }
    while b > a && is_blank(&row[b - 1]) {
        b -= 1;
    }
    (a, b)
}

fn reorder_segment(row: &[Cell], (a, b): (usize, usize), out: &mut VisualRow) {
    let texts: Vec<String> = row[a..b].iter().map(|c| c.text.clone()).collect();
    let shaped = shape(&texts);

    let mut items: Vec<(usize, String)> = Vec::with_capacity(b - a);
    let mut pads: Vec<usize> = Vec::new();
    for (offset, glyph) in shaped.into_iter().enumerate() {
        match glyph {
            Shaped::Glyph(g) => items.push((a + offset, g)),
            Shaped::Absorbed => pads.push(a + offset),
        }
    }
    items.extend(pads.into_iter().map(|col| (col, " ".to_string())));

    let mut text = String::new();
    let mut byte_to_item = Vec::new();
    for (idx, (col, _)) in items.iter().enumerate() {
        let c = if row[*col].text.is_empty() {
            ' '
        } else {
            base(&row[*col])
        };
        let c = if items[idx].1 == " " { ' ' } else { c };
        byte_to_item.push((text.len(), idx));
        text.push(c);
    }

    let info = BidiInfo::new(&text, None);
    let Some(para) = info.paragraphs.first() else {
        return;
    };
    let (levels, runs) = info.visual_runs(para, para.range.clone());

    let mut order: Vec<(usize, bool)> = Vec::with_capacity(items.len());
    for run in runs {
        let rtl = levels[run.start].is_rtl();
        let mut in_run: Vec<usize> = byte_to_item
            .iter()
            .filter(|(byte, _)| run.contains(byte))
            .map(|(_, idx)| *idx)
            .collect();
        if rtl {
            in_run.reverse();
        }
        order.extend(in_run.into_iter().map(|idx| (idx, rtl)));
    }

    for (slot, (idx, rtl)) in order.into_iter().enumerate() {
        let (col, glyph) = &items[idx];
        let visual_col = a + slot;
        let text = if rtl {
            glyph.chars().map(mirror).collect()
        } else {
            glyph.clone()
        };
        out.cells[visual_col] = Cell {
            text,
            style: row[*col].style,
            wide: false,
            continuation: false,
        };
        out.logical_to_visual[*col] = visual_col;
    }
}

pub fn visual_row(row: &[Cell]) -> VisualRow {
    let mut out = VisualRow {
        cells: row.to_vec(),
        logical_to_visual: (0..row.len()).collect(),
    };
    if !row.iter().any(|c| c.text.chars().any(is_rtl)) {
        return out;
    }
    for seg in segments(row) {
        let (a, b) = trim(row, seg);
        if a < b && row[a..b].iter().any(|c| c.text.chars().any(is_rtl)) {
            reorder_segment(row, (a, b), &mut out);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(s: &str) -> Vec<Cell> {
        s.chars()
            .map(|c| Cell {
                text: c.to_string(),
                ..Cell::default()
            })
            .collect()
    }

    fn render(v: &VisualRow) -> String {
        v.cells.iter().map(|c| c.text.as_str()).collect()
    }

    #[test]
    fn leaves_ltr_rows_untouched() {
        let r = row("hello │ world");
        assert_eq!(render(&visual_row(&r)), "hello │ world");
    }

    #[test]
    fn keeps_frame_borders_in_place() {
        let out = render(&visual_row(&row("│ مرحبا بك  │  status: جاهز")));
        assert!(out.starts_with("│ "));
        assert_eq!(out.chars().filter(|c| *c == '│').count(), 2);
        assert_eq!(out.chars().nth(12), Some('│'));
        assert!(out.contains("status: "));
    }

    #[test]
    fn reverses_arabic_and_keeps_latin_order() {
        let out = render(&visual_row(&row("افتح src/main.rs الآن")));
        assert!(out.contains("src/main.rs"));
        let latin = out.find("src").unwrap_or(0);
        let open = out.find('\u{FEA2}').unwrap_or(0);
        assert!(open > latin);
    }

    #[test]
    fn keeps_prompt_prefix_on_the_left() {
        let out = render(&visual_row(&row("> مرحبا")));
        assert!(out.starts_with("> "));
    }

    #[test]
    fn keeps_row_width_after_lam_alef() {
        let r = row("│ السلام عليكم │");
        let v = visual_row(&r);
        assert_eq!(v.cells.len(), r.len());
        assert_eq!(v.cells.last().map(|c| c.text.as_str()), Some("│"));
    }

    #[test]
    fn maps_every_logical_column_to_a_unique_visual_column() {
        let r = row("> اكتب هنا 42 مرة");
        let v = visual_row(&r);
        let mut seen = v.logical_to_visual.clone();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), r.len());
    }

    #[test]
    fn mirrors_brackets_inside_rtl_runs() {
        let out = render(&visual_row(&row("مرحبا (عالم)")));
        assert!(out.contains('(') && out.contains(')'));
        assert!(out.find('(').unwrap_or(0) < out.find(')').unwrap_or(0));
    }
}
