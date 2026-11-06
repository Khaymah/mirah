use mirah_bidi::Cell;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OverlayRow {
    pub row: u16,
    pub column: u16,
    pub cells: Vec<Cell>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Overlay {
    pub top: u16,
    pub rows: Vec<OverlayRow>,
    pub hide_cursor: bool,
}

impl Overlay {
    pub(crate) fn compose(&self, host_row: u16, base: &mut [Cell]) {
        for layer in self.rows.iter().filter(|r| r.row == host_row) {
            let start = usize::from(layer.column);
            for (offset, cell) in layer.cells.iter().enumerate() {
                if let Some(slot) = base.get_mut(start + offset) {
                    *slot = cell.clone();
                }
            }
        }
    }

    pub(crate) fn covers(&self, host_row: u16, column: u16) -> bool {
        self.rows.iter().any(|r| {
            r.row == host_row
                && (r.column..r.column.saturating_add(r.cells.len() as u16)).contains(&column)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(s: &str) -> Vec<Cell> {
        s.chars().map(|c| Cell::plain(c.to_string())).collect()
    }

    fn text(cells: &[Cell]) -> String {
        cells.iter().map(|c| c.text.as_str()).collect()
    }

    #[test]
    fn layers_are_drawn_over_the_row_and_clipped_at_its_end() {
        let overlay = Overlay {
            rows: vec![OverlayRow {
                row: 2,
                column: 3,
                cells: row("XYZ"),
            }],
            ..Overlay::default()
        };
        let mut base = row("abcde");
        overlay.compose(2, &mut base);
        assert_eq!(text(&base), "abcXY");
        let mut other = row("abcde");
        overlay.compose(1, &mut other);
        assert_eq!(text(&other), "abcde");
        assert!(overlay.covers(2, 4));
        assert!(!overlay.covers(2, 2));
    }
}
