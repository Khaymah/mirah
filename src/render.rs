use crate::segment::{Cell, Style, VisualRow, visual_row};
use std::fmt::Write as _;

pub struct Renderer {
    prev: Vec<Vec<Cell>>,
    prev_input_modes: String,
}

fn logical_row(screen: &vt100::Screen, row: u16, cols: u16) -> Vec<Cell> {
    (0..cols)
        .map(|col| match screen.cell(row, col) {
            Some(c) => Cell {
                text: c.contents().to_string(),
                style: Style {
                    fg: c.fgcolor(),
                    bg: c.bgcolor(),
                    bold: c.bold(),
                    dim: c.dim(),
                    italic: c.italic(),
                    underline: c.underline(),
                    inverse: c.inverse(),
                },
                wide: c.is_wide(),
                continuation: c.is_wide_continuation(),
            },
            None => Cell::default(),
        })
        .collect()
}

fn color(out: &mut String, c: vt100::Color, base: u8, bright: u8, ext: u8) {
    match c {
        vt100::Color::Default => {}
        vt100::Color::Idx(n) if n < 8 => {
            let _ = write!(out, ";{}", base + n);
        }
        vt100::Color::Idx(n) if n < 16 => {
            let _ = write!(out, ";{}", bright + n - 8);
        }
        vt100::Color::Idx(n) => {
            let _ = write!(out, ";{ext};5;{n}");
        }
        vt100::Color::Rgb(r, g, b) => {
            let _ = write!(out, ";{ext};2;{r};{g};{b}");
        }
    }
}

fn sgr(style: &Style) -> String {
    let mut out = String::from("\x1b[0");
    for (on, code) in [
        (style.bold, 1),
        (style.dim, 2),
        (style.italic, 3),
        (style.underline, 4),
        (style.inverse, 7),
    ] {
        if on {
            let _ = write!(out, ";{code}");
        }
    }
    color(&mut out, style.fg, 30, 90, 38);
    color(&mut out, style.bg, 40, 100, 48);
    out.push('m');
    out
}

fn draw_row(out: &mut String, row: u16, cells: &[Cell]) {
    let _ = write!(out, "\x1b[{};1H", row + 1);
    let mut current: Option<Style> = None;
    for cell in cells {
        if cell.continuation {
            continue;
        }
        if current != Some(cell.style) {
            out.push_str(&sgr(&cell.style));
            current = Some(cell.style);
        }
        if cell.text.is_empty() {
            out.push(' ');
        } else {
            out.push_str(&cell.text);
        }
    }
    out.push_str("\x1b[0m");
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            prev: Vec::new(),
            prev_input_modes: String::new(),
        }
    }

    pub fn invalidate(&mut self) {
        self.prev.clear();
    }

    pub fn frame(&mut self, screen: &vt100::Screen) -> String {
        let (rows, cols) = screen.size();
        let (rows, cols) = (rows.get(), cols.get());
        let mut out = String::from("\x1b[?2026h\x1b[?25l");
        if self.prev.len() != rows as usize {
            out.push_str("\x1b[0m\x1b[2J");
            self.prev = vec![Vec::new(); rows as usize];
        }

        let (cursor_row, cursor_col) = screen.cursor_position();
        let mut cursor_visual = cursor_col;
        for row in 0..rows {
            let VisualRow {
                cells,
                logical_to_visual,
            } = visual_row(&logical_row(screen, row, cols));
            if row == cursor_row
                && let Some(v) = logical_to_visual.get(cursor_col as usize)
            {
                cursor_visual = u16::try_from(*v).unwrap_or(cursor_col);
            }
            if self.prev[row as usize] != cells {
                draw_row(&mut out, row, &cells);
                self.prev[row as usize] = cells;
            }
        }

        let modes = screen.input_mode_formatted();
        if modes != self.prev_input_modes {
            out.push_str(&modes);
            self.prev_input_modes = modes;
        }

        let _ = write!(out, "\x1b[{};{}H", cursor_row + 1, cursor_visual + 1);
        if !screen.hide_cursor() {
            out.push_str("\x1b[?25h");
        }
        out.push_str("\x1b[?2026l");
        out
    }
}
