#[derive(Debug, thiserror::Error)]
pub enum TermError {
    #[error("terminal size must be non-zero, got {rows}x{cols}")]
    ZeroSize { rows: u16, cols: u16 },
}
