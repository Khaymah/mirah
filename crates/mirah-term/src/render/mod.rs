mod cursor;
mod sgr;

use crate::screen::logical_row;
use crate::sequences::{
    CLEAR_SCREEN, CURSOR_HIDE, CURSOR_SHOW, SGR_RESET, SYNC_BEGIN, SYNC_END, cursor_to,
};
use mirah_bidi::{Cell, Style, VisualRow, visual_row};

pub(crate) struct Renderer {
    prev: Vec<Vec<Cell>>,
    prev_input_modes: String,
}

fn draw_row(out: &mut String, row: u16, cells: &[Cell]) {
    out.push_str(&cursor_to(row, 0));
    let mut current: Option<Style> = None;
    for cell in cells.iter().filter(|c| !c.continuation) {
        if current != Some(cell.style) {
            out.push_str(&sgr::sgr(&cell.style));
            current = Some(cell.style);
        }
        out.push_str(if cell.text.is_empty() {
            " "
        } else {
            &cell.text
        });
    }
    out.push_str(SGR_RESET);
}

impl Renderer {
    pub(crate) fn new() -> Self {
        Self {
            prev: Vec::new(),
            prev_input_modes: String::new(),
        }
    }

    pub(crate) fn invalidate(&mut self) {
        self.prev.clear();
    }

    pub(crate) fn frame(&mut self, screen: &vt100::Screen) -> String {
        let (rows, cols) = screen.size();
        let (rows, cols) = (rows.get(), cols.get());
        let mut out = format!("{SYNC_BEGIN}{CURSOR_HIDE}");
        if self.prev.len() != usize::from(rows) {
            out.push_str(SGR_RESET);
            out.push_str(CLEAR_SCREEN);
            self.prev = vec![Vec::new(); usize::from(rows)];
        }

        let (cursor_row, cursor_col) = screen.cursor_position();
        let mut cursor_visual = cursor_col;
        for row in 0..rows {
            let VisualRow {
                cells,
                logical_to_visual,
            } = visual_row(&logical_row(screen, row, cols));
            if row == cursor_row {
                cursor_visual = cursor::visual_column(&logical_to_visual, cursor_col);
            }
            if self.prev[usize::from(row)] != cells {
                draw_row(&mut out, row, &cells);
                self.prev[usize::from(row)] = cells;
            }
        }

        let modes = screen.input_mode_formatted();
        if modes != self.prev_input_modes {
            out.push_str(&modes);
            self.prev_input_modes = modes;
        }

        out.push_str(&cursor_to(cursor_row, cursor_visual));
        if !screen.hide_cursor() {
            out.push_str(CURSOR_SHOW);
        }
        out.push_str(SYNC_END);
        out
    }
}
