mod cursor;
mod sgr;

use crate::layout::{Caches, Cursor, RenderOptions, layout};
use crate::overlay::Overlay;
use crate::screen::logical_row;
use crate::sequences::{
    CLEAR_SCREEN, CURSOR_HIDE, CURSOR_SHOW, RESET_INPUT_MODES, SGR_RESET, SYNC_BEGIN, SYNC_END,
    cursor_to,
};
use mirah_bidi::{Cell, Style, VisualRow};

const EDITOR_CURSOR_NUM: u32 = 2;
const BASE_MOUSE: &str = "\x1b[?1000h\x1b[?1006h";
const EDITOR_CURSOR_DEN: u32 = 3;

pub(crate) struct Renderer {
    prev: Vec<Vec<Cell>>,
    prev_input_modes: String,
    reset_modes: bool,
    pub(crate) modes_reset: bool,
    pub(crate) hold_map: bool,
    pub(crate) base_mouse: bool,
    options: RenderOptions,
    caches: Caches,
    to_child: Vec<Vec<u16>>,
}

fn invert(logical_to_visual: &[usize]) -> Vec<u16> {
    let width = logical_to_visual.len();
    let mut map: Vec<Option<u16>> = vec![None; width];
    for (logical, &visual) in logical_to_visual.iter().enumerate() {
        if let (Some(slot), Ok(column)) = (map.get_mut(visual), u16::try_from(logical)) {
            *slot = Some(column);
        }
    }
    (0..width)
        .map(|v| {
            (0..width)
                .flat_map(|d| [v.checked_sub(d), v.checked_add(d)])
                .flatten()
                .find_map(|i| map.get(i).copied().flatten())
                .unwrap_or_else(|| u16::try_from(v).unwrap_or(u16::MAX))
        })
        .collect()
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
            reset_modes: false,
            modes_reset: false,
            hold_map: false,
            base_mouse: false,
            options,
            caches: Caches::default(),
            to_child: Vec::new(),
        }
    }

    pub(crate) fn child_column(&self, row: u16, column: u16) -> u16 {
        self.to_child
            .get(usize::from(row))
            .and_then(|map| map.get(usize::from(column)))
            .copied()
            .unwrap_or(column)
    }

    pub(crate) fn invalidate(&mut self) {
        self.prev.clear();
    }

    pub(crate) fn redraw(&mut self) {
        self.invalidate();
        self.prev_input_modes.clear();
        self.reset_modes = true;
    }

    pub(crate) fn set_options(&mut self, options: RenderOptions) {
        self.options = options;
        self.caches = Caches::default();
        self.invalidate();
    }

    fn draw_rows(
        &mut self,
        out: &mut String,
        visual: &mut [VisualRow],
        overlay: &Overlay,
        cols: u16,
    ) {
        let host_rows = u16::try_from(self.prev.len()).unwrap_or(u16::MAX);
        for host in 0..host_rows {
            let mut cells = match host.checked_sub(overlay.top) {
                Some(child) => visual
                    .get_mut(usize::from(child))
                    .map(|row| std::mem::take(&mut row.cells))
                    .unwrap_or_default(),
                None => vec![Cell::plain(" "); usize::from(cols)],
            };
            overlay.compose(host, &mut cells);
            if self.prev[usize::from(host)] != cells {
                draw_row(out, host, &cells);
                self.prev[usize::from(host)] = cells;
            }
        }
    }

    fn cursor(screen: &vt100::Screen, rows: u16) -> Cursor {
        let (row, col) = screen.cursor_position();
        let visible = !screen.hide_cursor();
        Cursor {
            row: usize::from(row),
            col: usize::from(col),
            visible,
            editor: screen.alternate_screen()
                && visible
                && u32::from(row) * EDITOR_CURSOR_DEN < u32::from(rows) * EDITOR_CURSOR_NUM,
        }
    }

    pub(crate) fn frame(&mut self, screen: &vt100::Screen, overlay: &Overlay) -> String {
        let (rows, cols) = screen.size();
        let (rows, cols) = (rows.get(), cols.get());
        let host_rows = rows.saturating_add(overlay.top);
        let mut out = format!("{SYNC_BEGIN}{CURSOR_HIDE}");
        if self.prev.len() != usize::from(host_rows) {
            out.push_str(SGR_RESET);
            out.push_str(CLEAR_SCREEN);
            self.prev = vec![Vec::new(); usize::from(host_rows)];
        }
        let cursor = Self::cursor(screen, rows);
        let logical: Vec<Vec<Cell>> = (0..rows)
            .map(|row| logical_row(screen, row, cols))
            .collect();
        let mut visual = layout(logical, &cursor, self.options, &mut self.caches);
        if !self.hold_map || self.to_child.len() != visual.len() {
            self.to_child = visual
                .iter()
                .map(|line| invert(&line.logical_to_visual))
                .collect();
        }
        let cursor_visual = visual.get(cursor.row).map_or(cursor.col, |line| {
            usize::from(cursor::visual_column(
                &line.logical_to_visual,
                u16::try_from(cursor.col).unwrap_or(u16::MAX),
            ))
        });
        self.draw_rows(&mut out, &mut visual, overlay, cols);
        let mut modes = screen.input_mode_formatted();
        if self.base_mouse && screen.mouse_protocol_mode() == vt100::MouseProtocolMode::None {
            modes.push_str(BASE_MOUSE);
        }
        let changed = modes != self.prev_input_modes;
        if std::mem::take(&mut self.reset_modes) || changed {
            out.push_str(RESET_INPUT_MODES);
            out.push_str(&modes);
            self.prev_input_modes = modes;
            self.modes_reset = true;
        }
        let cursor_row = u16::try_from(cursor.row)
            .unwrap_or(0)
            .saturating_add(overlay.top);
        let cursor_col = u16::try_from(cursor_visual).unwrap_or(0);
        out.push_str(&cursor_to(cursor_row, cursor_col));
        if cursor.visible && !overlay.hide_cursor && !overlay.covers(cursor_row, cursor_col) {
            out.push_str(CURSOR_SHOW);
        }
        out.push_str(SYNC_END);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::invert;

    #[test]
    fn maps_every_screen_column_back_to_a_program_column() {
        assert_eq!(invert(&[3, 2, 1, 0]), [3, 2, 1, 0]);
        assert_eq!(invert(&[0, 3, 3, 3]), [0, 0, 3, 3]);
    }
}
