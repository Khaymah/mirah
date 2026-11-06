use crate::bridge::Bridge;
use crate::error::TermError;
use crate::layout::RenderOptions;
use crate::render::Renderer;
use std::num::NonZeroU16;

const SCROLLBACK_ROWS: usize = 0;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Output {
    pub to_host: Vec<u8>,
    pub to_child: Vec<u8>,
}

pub struct Emulator {
    parser: vt100::Parser<Bridge>,
    renderer: Renderer,
    changed: bool,
}

fn non_zero(rows: u16, cols: u16) -> Result<(NonZeroU16, NonZeroU16), TermError> {
    match (NonZeroU16::new(rows), NonZeroU16::new(cols)) {
        (Some(r), Some(c)) => Ok((r, c)),
        _ => Err(TermError::ZeroSize { rows, cols }),
    }
}

impl Emulator {
    pub fn new(rows: u16, cols: u16) -> Result<Self, TermError> {
        Self::with_options(rows, cols, RenderOptions::default())
    }

    pub fn with_options(rows: u16, cols: u16, options: RenderOptions) -> Result<Self, TermError> {
        let (rows, cols) = non_zero(rows, cols)?;
        Ok(Self {
            parser: vt100::Parser::new_with_callbacks(
                rows,
                cols,
                SCROLLBACK_ROWS,
                Bridge::default(),
            ),
            renderer: Renderer::new(options),
            changed: true,
        })
    }

    pub fn feed(&mut self, bytes: &[u8]) {
        self.parser.process(bytes);
        self.changed |= !bytes.is_empty();
    }

    pub fn needs_frame(&self) -> bool {
        self.changed
    }

    pub fn is_synchronizing(&self) -> bool {
        self.parser.callbacks().synchronized
    }

    pub fn resize(&mut self, rows: u16, cols: u16) -> Result<(), TermError> {
        let (rows, cols) = non_zero(rows, cols)?;
        self.parser.set_size(rows, cols);
        self.renderer.invalidate();
        self.changed = true;
        Ok(())
    }

    pub fn take_output(&mut self) -> Output {
        let bridge = self.parser.callbacks_mut();
        Output {
            to_host: std::mem::take(&mut bridge.to_host),
            to_child: std::mem::take(&mut bridge.to_child),
        }
    }

    pub fn child_column(&self, row: u16, column: u16) -> u16 {
        self.renderer.child_column(row, column)
    }

    pub fn frame(&mut self) -> String {
        self.changed = false;
        self.renderer.frame(self.parser.screen())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_sizes() {
        assert!(matches!(
            Emulator::new(0, 80),
            Err(TermError::ZeroSize { .. })
        ));
        let mut e = Emulator::new(24, 80).expect("valid size");
        assert!(e.resize(24, 0).is_err());
    }

    #[test]
    fn a_frame_is_needed_only_after_new_output_or_a_resize() {
        let mut e = Emulator::new(24, 80).expect("valid size");
        assert!(e.needs_frame());
        e.frame();
        assert!(!e.needs_frame());
        e.feed(b"");
        assert!(!e.needs_frame());
        e.feed(b"x");
        assert!(e.needs_frame());
        e.frame();
        e.resize(10, 40).expect("valid size");
        assert!(e.needs_frame());
    }

    #[test]
    fn maps_screen_columns_to_program_columns() {
        let mut e = Emulator::new(1, 10).expect("valid size");
        e.feed("بتك".as_bytes());
        let frame = e.frame();
        assert!(
            frame.contains("       \u{FEDA}\u{FE98}\u{FE91}"),
            "{frame:?}"
        );
        let map: Vec<u16> = (0..10).map(|c| e.child_column(0, c)).collect();
        assert_eq!(&map[7..], [2, 1, 0]);
        let mut seen = map.clone();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(
            seen.len(),
            10,
            "every screen column maps to its own program column: {map:?}"
        );
    }

    #[test]
    fn reports_synchronized_output() {
        let mut e = Emulator::new(24, 80).expect("valid size");
        e.feed(b"\x1b[?2026hhalf");
        assert!(e.is_synchronizing());
        e.feed(b"\x1b[?2026l");
        assert!(!e.is_synchronizing());
    }

    #[test]
    fn output_is_drained_once() {
        let mut e = Emulator::new(24, 80).expect("valid size");
        e.feed(b"\x1b[6n");
        assert_eq!(e.take_output().to_child, b"\x1b[1;1R");
        assert_eq!(e.take_output(), Output::default());
    }
}
