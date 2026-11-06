use compact_str::CompactString;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Color {
    #[default]
    Default,
    Indexed(u8),
    Rgb(u8, u8, u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub inverse: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Cell {
    pub text: CompactString,
    pub style: Style,
    pub wide: bool,
    pub continuation: bool,
}

impl Cell {
    pub fn plain(text: impl Into<CompactString>) -> Self {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }

    pub(crate) fn base(&self) -> char {
        self.text.chars().next().unwrap_or(' ')
    }

    pub(crate) fn is_blank(&self) -> bool {
        self.base() == ' '
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualRow {
    pub cells: Vec<Cell>,
    pub logical_to_visual: Vec<usize>,
}

impl VisualRow {
    pub(crate) fn identity(row: &[Cell]) -> Self {
        Self {
            cells: row.to_vec(),
            logical_to_visual: (0..row.len()).collect(),
        }
    }
}
