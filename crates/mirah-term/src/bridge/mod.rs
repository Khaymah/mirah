pub(crate) mod clipboard;
mod local;
mod passthrough;

use crate::sequences::{BEL, MODE_FOCUS_EVENTS, MODE_SYNCHRONIZED_OUTPUT};

#[derive(Default)]
pub(crate) struct Bridge {
    pub(crate) to_host: Vec<u8>,
    pub(crate) to_child: Vec<u8>,
    pub(crate) synchronized: bool,
    pub(crate) clipboard: clipboard::Clipboard,
    pub(crate) focus_events: bool,
    pub(crate) copied: Option<String>,
}

fn synchronized_output(i1: Option<u8>, i2: Option<u8>, params: &[&[u16]], c: char) -> Option<bool> {
    let toggles = i1 == Some(b'?')
        && i2.is_none()
        && params
            .iter()
            .any(|p| p.first() == Some(&MODE_SYNCHRONIZED_OUTPUT));
    match c {
        'h' if toggles => Some(true),
        'l' if toggles => Some(false),
        _ => None,
    }
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
        if !self.clipboard.allow {
            tracing::debug!(
                bytes = data.len(),
                "dropped a clipboard write from the program"
            );
            return;
        }
        let text = self
            .clipboard
            .visual_text
            .then(|| clipboard::logical_text(data))
            .flatten();
        let fixed = text.as_ref().map(|t| clipboard::encode(t.as_bytes()));
        if let Some(text) = text {
            self.copied = Some(text);
        }
        let data = fixed.as_deref().unwrap_or(data);
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
        if let Some(on) = synchronized_output(i1, i2, params, c) {
            self.synchronized = on;
        }
        if i1 == Some(b'?')
            && i2.is_none()
            && matches!(c, 'h' | 'l')
            && params.iter().any(|p| p.first() == Some(&MODE_FOCUS_EVENTS))
        {
            self.focus_events = c == 'h';
        }
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
    fn tracks_synchronized_output() {
        assert!(feed(b"\x1b[?2026h").callbacks().synchronized);
        assert!(!feed(b"\x1b[?2026h\x1b[?2026l").callbacks().synchronized);
        assert!(!feed(b"\x1b[?2004h").callbacks().synchronized);
    }

    #[test]
    fn clipboard_writes_can_be_dropped_or_put_in_reading_order() {
        let write = [
            b"\x1b]52;c;".as_slice(),
            &clipboard::encode("الهأ".as_bytes()),
            b"\x07",
        ]
        .concat();
        let plain = feed(&write).callbacks().to_host.clone();
        assert_eq!(plain, [&write[..write.len() - 1], b"\x1b\\"].concat());
        let rows = core::num::NonZeroU16::new(24).unwrap_or(core::num::NonZeroU16::MIN);
        let bridge = Bridge {
            clipboard: clipboard::Clipboard {
                allow: true,
                visual_text: true,
            },
            ..Bridge::default()
        };
        let mut p = vt100::Parser::new_with_callbacks(rows, rows, 0, bridge);
        p.process(&write);
        let expected = [
            b"\x1b]52;c;".as_slice(),
            &clipboard::encode("أهلا".as_bytes()),
            b"\x1b\\",
        ]
        .concat();
        assert_eq!(p.callbacks().to_host, expected);
        p.callbacks_mut().clipboard.allow = false;
        p.callbacks_mut().to_host.clear();
        p.process(&write);
        assert!(p.callbacks().to_host.is_empty());
    }

    #[test]
    fn tracks_focus_event_reporting() {
        assert!(feed(b"\x1b[?1004h").callbacks().focus_events);
        assert!(!feed(b"\x1b[?1004h\x1b[?1004l").callbacks().focus_events);
    }

    #[test]
    fn forwards_color_queries() {
        let p = feed(b"\x1b]11;?\x07");
        assert_eq!(p.callbacks().to_host, b"\x1b]11;?\x1b\\");
    }
}
