use crate::bridge::Bridge;
use crate::error::TermError;
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
}

fn non_zero(rows: u16, cols: u16) -> Result<(NonZeroU16, NonZeroU16), TermError> {
    match (NonZeroU16::new(rows), NonZeroU16::new(cols)) {
        (Some(r), Some(c)) => Ok((r, c)),
        _ => Err(TermError::ZeroSize { rows, cols }),
    }
}

impl Emulator {
    pub fn new(rows: u16, cols: u16) -> Result<Self, TermError> {
        let (rows, cols) = non_zero(rows, cols)?;
        Ok(Self {
            parser: vt100::Parser::new_with_callbacks(
                rows,
                cols,
                SCROLLBACK_ROWS,
                Bridge::default(),
            ),
            renderer: Renderer::new(),
        })
    }

    pub fn feed(&mut self, bytes: &[u8]) {
        self.parser.process(bytes);
    }

    pub fn resize(&mut self, rows: u16, cols: u16) -> Result<(), TermError> {
        let (rows, cols) = non_zero(rows, cols)?;
        self.parser.set_size(rows, cols);
        self.renderer.invalidate();
        Ok(())
    }

    pub fn take_output(&mut self) -> Output {
        let bridge = self.parser.callbacks_mut();
        Output {
            to_host: std::mem::take(&mut bridge.to_host),
            to_child: std::mem::take(&mut bridge.to_child),
        }
    }

    pub fn frame(&mut self) -> String {
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
    fn output_is_drained_once() {
        let mut e = Emulator::new(24, 80).expect("valid size");
        e.feed(b"\x1b[6n");
        assert_eq!(e.take_output().to_child, b"\x1b[1;1R");
        assert_eq!(e.take_output(), Output::default());
    }
}
