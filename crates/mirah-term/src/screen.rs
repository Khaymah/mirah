use mirah_bidi::{Cell, Color, Style};

fn color(c: vt100::Color) -> Color {
    match c {
        vt100::Color::Default => Color::Default,
        vt100::Color::Idx(n) => Color::Indexed(n),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}

fn cell(c: &vt100::Cell) -> Cell {
    Cell {
        text: c.contents().to_string(),
        style: Style {
            fg: color(c.fgcolor()),
            bg: color(c.bgcolor()),
            bold: c.bold(),
            dim: c.dim(),
            italic: c.italic(),
            underline: c.underline(),
            inverse: c.inverse(),
        },
        wide: c.is_wide(),
        continuation: c.is_wide_continuation(),
    }
}

pub(crate) fn logical_row(screen: &vt100::Screen, row: u16, cols: u16) -> Vec<Cell> {
    (0..cols)
        .map(|col| screen.cell(row, col).map(cell).unwrap_or_default())
        .collect()
}
