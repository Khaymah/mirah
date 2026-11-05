mod cursor;
mod sgr;

use crate::layout::{Cursor, RenderOptions, layout};
use crate::screen::logical_row;
use crate::sequences::{
    CLEAR_SCREEN, CURSOR_HIDE, CURSOR_SHOW, SGR_RESET, SYNC_BEGIN, SYNC_END, cursor_to,
};
use mirah_bidi::{Cell, Style};

const EDITOR_CURSOR_NUM: u32 = 2;
const EDITOR_CURSOR_DEN: u32 = 3;

pub(crate) struct Renderer {
    prev: Vec<Vec<Cell>>,
    prev_input_modes: String,
    options: RenderOptions,
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
    pub(crate) fn new(options: RenderOptions) -> Self {
        Self {
            prev: Vec::new(),
            prev_input_modes: String::new(),
            options,
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
        let visible = !screen.hide_cursor();
        let cursor = Cursor {
            row: usize::from(cursor_row),
            col: usize::from(cursor_col),
            visible,
            editor: screen.alternate_screen()
                && visible
                && u32::from(cursor_row) * EDITOR_CURSOR_DEN < u32::from(rows) * EDITOR_CURSOR_NUM,
        };
        let logical: Vec<Vec<Cell>> = (0..rows)
            .map(|row| logical_row(screen, row, cols))
            .collect();
        let visual = layout(logical, &cursor, self.options);
        let mut cursor_visual = cursor_col;
        for (row, line) in (0..rows).zip(visual) {
            if row == cursor_row {
                cursor_visual = cursor::visual_column(&line.logical_to_visual, cursor_col);
            }
            if self.prev[usize::from(row)] != line.cells {
                draw_row(&mut out, row, &line.cells);
                self.prev[usize::from(row)] = line.cells;
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
