const PREFIX: &[u8] = b"\x1b[<";
const MAX_SEQUENCE: usize = 32;

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

impl MouseRemap {
    pub(super) fn apply(&mut self, input: &[u8], to_child: impl Fn(u16, u16) -> u16) -> Vec<u8> {
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
                    match parse(&rest[PREFIX.len()..end]) {
                        Some((button, x, y)) if x > 0 && y > 0 => {
                            let column = to_child(y - 1, x - 1).saturating_add(1);
                            out.extend_from_slice(
                                format!("\x1b[<{button};{column};{y}").as_bytes(),
                            );
                            out.push(kind);
                        }
                        _ => out.extend_from_slice(&rest[..=end]),
                    }
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

    fn mirror(_row: u16, col: u16) -> u16 {
        9 - col
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
