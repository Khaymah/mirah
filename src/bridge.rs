#[derive(Default)]
pub struct Bridge {
    pub to_host: Vec<u8>,
    pub to_child: Vec<u8>,
}

fn is_prefix(b: u8) -> bool {
    matches!(b, b'?' | b'>' | b'<' | b'=')
}

fn rebuild_csi(i1: Option<u8>, i2: Option<u8>, params: &[&[u16]], c: char) -> Vec<u8> {
    let mut out = b"\x1b[".to_vec();
    let mut suffix = Vec::new();
    for i in [i1, i2].into_iter().flatten() {
        if is_prefix(i) && out.len() == 2 {
            out.push(i);
        } else {
            suffix.push(i);
        }
    }
    let joined: Vec<String> = params
        .iter()
        .map(|p| p.iter().map(u16::to_string).collect::<Vec<_>>().join(":"))
        .collect();
    let only_default = params.len() == 1 && params[0] == [0];
    if !only_default {
        out.extend_from_slice(joined.join(";").as_bytes());
    }
    out.extend_from_slice(&suffix);
    let mut buf = [0u8; 4];
    out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
    out
}

fn first_param(params: &[&[u16]]) -> u16 {
    params.first().and_then(|p| p.first()).copied().unwrap_or(0)
}

impl vt100::Callbacks for Bridge {
    fn audible_bell(&mut self, _: &mut vt100::Screen) {
        self.to_host.push(0x07);
    }

    fn set_window_title(&mut self, _: &mut vt100::Screen, title: &[u8]) {
        self.to_host.extend_from_slice(b"\x1b]2;");
        self.to_host.extend_from_slice(title);
        self.to_host.extend_from_slice(b"\x1b\\");
    }

    fn copy_to_clipboard(&mut self, _: &mut vt100::Screen, ty: &[u8], data: &[u8]) {
        self.to_host.extend_from_slice(b"\x1b]52;");
        self.to_host.extend_from_slice(ty);
        self.to_host.push(b';');
        self.to_host.extend_from_slice(data);
        self.to_host.extend_from_slice(b"\x1b\\");
    }

    fn unhandled_csi(
        &mut self,
        screen: &mut vt100::Screen,
        i1: Option<u8>,
        i2: Option<u8>,
        params: &[&[u16]],
        c: char,
    ) {
        let passthrough = match (i1, i2, c) {
            (None, None, 'n') => {
                match first_param(params) {
                    6 => {
                        let (row, col) = screen.cursor_position();
                        self.to_child
                            .extend_from_slice(format!("\x1b[{};{}R", row + 1, col + 1).as_bytes());
                    }
                    5 => self.to_child.extend_from_slice(b"\x1b[0n"),
                    _ => {}
                }
                false
            }
            (None | Some(b'>' | b'='), None, 'c') => true,
            (Some(b'>'), None, 'q') => true,
            (Some(b'?' | b'>' | b'<' | b'='), None, 'u') => true,
            (_, Some(b'$'), 'p') | (Some(b'$'), None, 'p') => true,
            (None, None, 't') => true,
            (Some(b'?'), None, 'h' | 'l') => !params.iter().any(|p| p.first() == Some(&2026)),
            _ => false,
        };
        if passthrough {
            self.to_host.extend(rebuild_csi(i1, i2, params, c));
        }
    }

    fn unhandled_osc(&mut self, _: &mut vt100::Screen, params: &[&[u8]]) {
        if params.iter().any(|p| *p == b"?") {
            self.to_host.extend_from_slice(b"\x1b]");
            self.to_host.extend(params.join(&b';'));
            self.to_host.extend_from_slice(b"\x1b\\");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed(bytes: &[u8]) -> vt100::Parser<Bridge> {
        let rows = core::num::NonZeroU16::new(24).unwrap_or(core::num::NonZeroU16::MIN);
        let cols = core::num::NonZeroU16::new(80).unwrap_or(core::num::NonZeroU16::MIN);
        let mut p = vt100::Parser::new_with_callbacks(rows, cols, 0, Bridge::default());
        p.process(bytes);
        p
    }

    #[test]
    fn answers_cursor_position_report_from_logical_screen() {
        let p = feed(b"\x1b[5;10H\x1b[6n");
        assert_eq!(p.callbacks().to_child, b"\x1b[5;10R");
        assert!(p.callbacks().to_host.is_empty());
    }

    #[test]
    fn forwards_device_attribute_queries_to_host() {
        let p = feed(b"\x1b[c\x1b[>q\x1b[?u");
        assert_eq!(p.callbacks().to_host, b"\x1b[c\x1b[>q\x1b[?u");
    }

    #[test]
    fn forwards_decrqm_with_private_prefix() {
        let p = feed(b"\x1b[?2026$p");
        assert_eq!(p.callbacks().to_host, b"\x1b[?2026$p");
    }

    #[test]
    fn swallows_synchronized_output_mode() {
        let p = feed(b"\x1b[?2026h\x1b[?1004h");
        assert_eq!(p.callbacks().to_host, b"\x1b[?1004h");
    }

    #[test]
    fn forwards_color_queries() {
        let p = feed(b"\x1b]11;?\x07");
        assert_eq!(p.callbacks().to_host, b"\x1b]11;?\x1b\\");
    }
}
