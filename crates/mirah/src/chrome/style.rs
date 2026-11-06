use mirah_bidi::{BidiOptions, Cell, Color, Shaping, Style, visual_row_with};

pub(crate) const BAR_BG: Color = Color::Rgb(30, 31, 35);
pub(crate) const BOX_BG: Color = Color::Rgb(26, 27, 31);
pub(crate) const ROW_BG: Color = Color::Rgb(42, 44, 50);
pub(crate) const CHIP_BG: Color = Color::Rgb(58, 61, 69);
pub(crate) const TEXT: Color = Color::Rgb(206, 208, 214);
pub(crate) const DIM: Color = Color::Rgb(118, 122, 132);
pub(crate) const ACCENT: Color = Color::Rgb(217, 145, 104);
pub(crate) const RECORDING: Color = Color::Rgb(232, 112, 112);
pub(crate) const UPDATE: Color = Color::Rgb(132, 196, 140);

pub(crate) fn style(fg: Color, bg: Color, bold: bool) -> Style {
    Style {
        fg,
        bg,
        bold,
        ..Style::default()
    }
}

fn is_mark(c: char) -> bool {
    matches!(
        u32::from(c),
        0x0300..=0x036F
            | 0x0610..=0x061A
            | 0x064B..=0x065F
            | 0x0670
            | 0x06D6..=0x06DC
            | 0x06DF..=0x06E4
            | 0x06E7..=0x06E8
            | 0x06EA..=0x06ED
            | 0x200D
            | 0xFE00..=0xFE0F
    )
}

pub(crate) fn shaped(text: &str, style: Style, shaping: Shaping) -> Vec<Cell> {
    let mut logical: Vec<Cell> = Vec::new();
    for c in text.chars() {
        match logical.last_mut() {
            Some(last) if is_mark(c) => last.text.push(c),
            _ => logical.push(Cell {
                text: c.to_string().into(),
                style,
                ..Cell::default()
            }),
        }
    }
    let options = BidiOptions {
        shaping,
        rtl_base: true,
    };
    visual_row_with(&logical, None, options).cells
}

pub(crate) fn blank(width: usize, style: Style) -> Vec<Cell> {
    vec![
        Cell {
            text: " ".into(),
            style,
            ..Cell::default()
        };
        width
    ]
}

pub(crate) fn put(row: &mut [Cell], start: usize, part: &[Cell]) {
    for (offset, cell) in part.iter().enumerate() {
        if let Some(slot) = row.get_mut(start + offset) {
            *slot = cell.clone();
        }
    }
}

pub(crate) fn right_to_left(
    row: &mut [Cell],
    right: usize,
    parts: &[Vec<Cell>],
    gap: usize,
) -> usize {
    let mut edge = right;
    for part in parts {
        let start = edge.saturating_sub(part.len());
        put(row, start, part);
        edge = start.saturating_sub(gap);
    }
    edge
}

pub(crate) struct Line {
    pub(crate) cells: Vec<Cell>,
    pub(crate) focused: bool,
}

impl Line {
    pub(crate) fn new(cells: Vec<Cell>, focused: bool) -> Self {
        Self { cells, focused }
    }
}

pub(crate) struct Frame {
    pub(crate) rows: Vec<Vec<Cell>>,
    pub(crate) width: usize,
}

pub(crate) const BODY_TOP: usize = 3;
pub(crate) const PAD: usize = 2;

pub(crate) fn boxed(title: &[Cell], body: Vec<Line>, footer: &[Cell], shaping: Shaping) -> Frame {
    let inner = body
        .iter()
        .map(|l| l.cells.len())
        .chain([title.len(), footer.len()])
        .max()
        .unwrap_or(0)
        + 2 * PAD;
    let border = style(DIM, BOX_BG, false);
    let edge = |left: &str, right: &str| {
        let mut row = shaped(left, border, shaping);
        row.extend(std::iter::repeat_n(shaped("─", border, shaping), inner).flatten());
        row.extend(shaped(right, border, shaping));
        row
    };
    let line = |content: &[Cell], bg: Color, marker: bool| {
        let mut row = shaped("│", border, shaping);
        let mut middle = blank(inner, style(TEXT, bg, false));
        if marker {
            put(
                &mut middle,
                0,
                &shaped("▌", style(ACCENT, bg, false), shaping),
            );
        }
        right_to_left(&mut middle, inner - PAD, &[content.to_vec()], 0);
        row.extend(middle);
        row.extend(shaped("│", border, shaping));
        row
    };
    let mut rows = vec![edge("╭", "╮"), line(title, BOX_BG, false), edge("├", "┤")];
    for content in body {
        let bg = if content.focused { ROW_BG } else { BOX_BG };
        let cells: Vec<Cell> = content
            .cells
            .into_iter()
            .map(|mut c| {
                if c.style.bg == BOX_BG {
                    c.style.bg = bg;
                }
                c
            })
            .collect();
        rows.push(line(&cells, bg, content.focused));
    }
    rows.push(edge("├", "┤"));
    rows.push(line(footer, BOX_BG, false));
    rows.push(edge("╰", "╯"));
    Frame {
        rows,
        width: inner + 2,
    }
}

pub(crate) fn content_start(frame_width: usize, content_len: usize) -> usize {
    (frame_width - 1 - PAD).saturating_sub(content_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(cells: &[Cell]) -> String {
        cells.iter().map(|c| c.text.as_str()).collect()
    }

    #[test]
    fn marks_stay_on_their_letter() {
        let cells = shaped("سجّل", Style::default(), Shaping::Plain);
        assert_eq!(cells.len(), 3);
        assert!(cells.iter().any(|c| c.text == "ج\u{651}"));
    }

    #[test]
    fn parts_are_placed_from_the_right() {
        let mut row = blank(12, Style::default());
        let a = shaped("ab", Style::default(), Shaping::Plain);
        let b = shaped("cd", Style::default(), Shaping::Plain);
        let left = right_to_left(&mut row, 12, &[a, b], 1);
        assert_eq!(text(&row), "       cd ab");
        assert_eq!(left, 6);
    }

    #[test]
    fn a_box_has_rounded_corners_and_equal_rows() {
        let s = Style::default();
        let frame = boxed(
            &shaped("عنوان", s, Shaping::Plain),
            vec![Line::new(shaped("سطر", s, Shaping::Plain), true)],
            &shaped("مساعدة", s, Shaping::Plain),
            Shaping::Plain,
        );
        assert!(text(&frame.rows[0]).starts_with('╭'));
        assert!(text(&frame.rows[frame.rows.len() - 1]).ends_with('╯'));
        assert!(frame.rows.iter().all(|r| r.len() == frame.width));
        assert!(text(&frame.rows[1]).contains("ناونع"));
        assert!(text(&frame.rows[BODY_TOP]).starts_with("│▌"));
        let at = content_start(frame.width, 3);
        assert_eq!(text(&frame.rows[BODY_TOP][at..at + 3]), "رطس");
    }
}
