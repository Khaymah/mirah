mod local;
mod passthrough;

use crate::sequences::BEL;

#[derive(Default)]
pub(crate) struct Bridge {
    pub(crate) to_host: Vec<u8>,
    pub(crate) to_child: Vec<u8>,
}

fn first_param(params: &[&[u16]]) -> u16 {
    params.first().and_then(|p| p.first()).copied().unwrap_or(0)
}

impl vt100::Callbacks for Bridge {
    fn audible_bell(&mut self, _: &mut vt100::Screen) {
        self.to_host.push(BEL);
    }

    fn set_window_title(&mut self, _: &mut vt100::Screen, title: &[u8]) {
        self.to_host.extend(passthrough::window_title(title));
    }

    fn copy_to_clipboard(&mut self, _: &mut vt100::Screen, selection: &[u8], data: &[u8]) {
        tracing::debug!(bytes = data.len(), "forwarding OSC 52 clipboard write");
        self.to_host.extend(passthrough::clipboard(selection, data));
    }

    fn unhandled_csi(
        &mut self,
        screen: &mut vt100::Screen,
        i1: Option<u8>,
        i2: Option<u8>,
        params: &[&[u16]],
        c: char,
    ) {
        if (i1, i2, c) == (None, None, 'n') {
            if let Some(reply) = local::device_status_report(screen, first_param(params)) {
                self.to_child.extend(reply);
            }
            return;
        }
        if passthrough::forwards_csi(i1, i2, params, c) {
            let sequence = passthrough::rebuild_csi(i1, i2, params, c);
            tracing::debug!(sequence = ?String::from_utf8_lossy(&sequence), "forwarding CSI to host");
            self.to_host.extend(sequence);
        }
    }

    fn unhandled_osc(&mut self, _: &mut vt100::Screen, params: &[&[u8]]) {
        if let Some(query) = passthrough::osc_query(params) {
            self.to_host.extend(query);
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
