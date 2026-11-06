const PREFIX: &[u8] = b"\x1b[<";
const MAX_SEQUENCE: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MouseEvent {
    pub(crate) button: u16,
    pub(crate) column: u16,
    pub(crate) row: u16,
    pub(crate) press: bool,
}

#[derive(Default)]
pub(super) struct MouseRemap {
    pending: Vec<u8>,
}

fn parse(body: &[u8]) -> Option<(u16, u16, u16)> {
    let text = std::str::from_utf8(body).ok()?;
    let mut parts = text.split(';');
    let button = parts.next()?.parse().ok()?;
    let x = parts.next()?.parse().ok()?;
    let y = parts.next()?.parse().ok()?;
    parts.next().is_none().then_some((button, x, y))
}

fn is_body_byte(b: u8) -> bool {
    b.is_ascii_digit() || b == b';'
}

fn emit(
    out: &mut Vec<u8>,
    sequence: &[u8],
    kind: u8,
    route: &mut impl FnMut(MouseEvent) -> Option<MouseEvent>,
) {
    let body = &sequence[PREFIX.len()..sequence.len() - 1];
    match parse(body) {
        Some((button, x, y)) if x > 0 && y > 0 => {
            let event = MouseEvent {
                button,
                column: x - 1,
                row: y - 1,
                press: kind == b'M',
            };
            if let Some(e) = route(event) {
                let (col, row) = (e.column + 1, e.row + 1);
                out.extend_from_slice(format!("\x1b[<{};{col};{row}", e.button).as_bytes());
                out.push(kind);
            }
        }
        _ => out.extend_from_slice(sequence),
    }
}

impl MouseRemap {
    pub(super) fn apply(
        &mut self,
        input: &[u8],
        mut route: impl FnMut(MouseEvent) -> Option<MouseEvent>,
    ) -> Vec<u8> {
        let mut bytes = std::mem::take(&mut self.pending);
        bytes.extend_from_slice(input);
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            let rest = &bytes[i..];
            if !rest.starts_with(PREFIX) {
                out.push(bytes[i]);
                i += 1;
                continue;
            }
            let body_len = rest[PREFIX.len()..]
                .iter()
                .take_while(|b| is_body_byte(**b))
                .count();
            let end = PREFIX.len() + body_len;
            match rest.get(end) {
                None if end < MAX_SEQUENCE => {
                    self.pending = rest.to_vec();
                    break;
                }
                Some(&kind @ (b'M' | b'm')) => {
                    emit(&mut out, &rest[..=end], kind, &mut route);
                    i += end + 1;
                }
                _ => {
                    out.extend_from_slice(&rest[..end.min(rest.len())]);
                    i += end.min(rest.len()).max(1);
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mirror(e: MouseEvent) -> Option<MouseEvent> {
        Some(MouseEvent {
            column: 9 - e.column,
            ..e
        })
    }

    #[test]
    fn remaps_the_column_of_sgr_mouse_events() {
        let mut remap = MouseRemap::default();
        assert_eq!(remap.apply(b"\x1b[<0;10;3M", mirror), b"\x1b[<0;1;3M");
        assert_eq!(remap.apply(b"a\x1b[<0;1;1mb", mirror), b"a\x1b[<0;10;1mb");
    }

    #[test]
    fn keeps_keys_and_other_sequences_untouched() {
        let mut remap = MouseRemap::default();
        let keys = b"hello\x1b[A\x1b[1;5C\r";
        assert_eq!(remap.apply(keys, mirror), keys);
    }

    #[test]
    fn joins_a_sequence_split_across_reads() {
        let mut remap = MouseRemap::default();
        assert_eq!(remap.apply(b"x\x1b[<0;1", mirror), b"x");
        assert_eq!(remap.apply(b"0;2M", mirror), b"\x1b[<0;1;2M");
    }

    #[test]
    fn a_routed_event_can_be_dropped_or_moved() {
        let mut remap = MouseRemap::default();
        let drop_top = |e: MouseEvent| {
            (e.row > 0).then(|| MouseEvent {
                row: e.row - 1,
                ..e
            })
        };
        assert_eq!(remap.apply(b"\x1b[<0;5;1M", drop_top), b"");
        assert_eq!(remap.apply(b"\x1b[<0;5;3m", drop_top), b"\x1b[<0;5;2m");
    }

    #[test]
    fn a_lone_escape_is_sent_at_once() {
        let mut remap = MouseRemap::default();
        assert_eq!(remap.apply(b"\x1b", mirror), b"\x1b");
        assert_eq!(remap.apply(b"\x1b[", mirror), b"\x1b[");
    }

    #[test]
    fn a_malformed_sequence_passes_through() {
        let mut remap = MouseRemap::default();
        assert_eq!(remap.apply(b"\x1b[<0;x", mirror), b"\x1b[<0;x");
    }
}
