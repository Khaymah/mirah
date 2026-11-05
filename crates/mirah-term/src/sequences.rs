pub const BEL: u8 = 0x07;
pub const CSI: &str = "\x1b[";
pub const OSC: &str = "\x1b]";
pub const ST: &str = "\x1b\\";

pub const SGR_RESET: &str = "\x1b[0m";
pub const CLEAR_SCREEN: &str = "\x1b[2J";
pub const CURSOR_HIDE: &str = "\x1b[?25l";
pub const CURSOR_SHOW: &str = "\x1b[?25h";
pub const SYNC_BEGIN: &str = "\x1b[?2026h";
pub const SYNC_END: &str = "\x1b[?2026l";
pub const ALT_SCREEN_ENTER: &str = "\x1b[?1049h";
pub const ALT_SCREEN_LEAVE: &str = "\x1b[?1049l";
pub const BDSM_EXPLICIT: &str = "\x1b[8l";
pub const BDSM_IMPLICIT: &str = "\x1b[8h";
pub const MOUSE_TRACKING_OFF: &str = "\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l";
pub const BRACKETED_PASTE_OFF: &str = "\x1b[?2004l";
pub const CURSOR_KEYS_NORMAL: &str = "\x1b[?1l";
pub const KEYPAD_NUMERIC: &str = "\x1b>";
pub const KITTY_KEYBOARD_POP: &str = "\x1b[<u";

pub const MODE_SYNCHRONIZED_OUTPUT: u16 = 2026;

const ENTER: [&str; 3] = [ALT_SCREEN_ENTER, BDSM_EXPLICIT, CLEAR_SCREEN];

const LEAVE: [&str; 10] = [
    SYNC_END,
    SGR_RESET,
    CURSOR_SHOW,
    MOUSE_TRACKING_OFF,
    BRACKETED_PASTE_OFF,
    CURSOR_KEYS_NORMAL,
    KEYPAD_NUMERIC,
    KITTY_KEYBOARD_POP,
    ALT_SCREEN_LEAVE,
    BDSM_IMPLICIT,
];

pub fn enter() -> String {
    ENTER.concat()
}

pub fn leave() -> String {
    LEAVE.concat()
}

pub fn cursor_to(row: u16, col: u16) -> String {
    format!("{CSI}{};{}H", u32::from(row) + 1, u32::from(col) + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_switches_to_alt_screen_and_explicit_bidi() {
        assert_eq!(enter(), "\x1b[?1049h\x1b[8l\x1b[2J");
    }

    #[test]
    fn leave_restores_every_mode_and_ends_with_implicit_bidi() {
        assert_eq!(
            leave(),
            "\x1b[?2026l\x1b[0m\x1b[?25h\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l\x1b[?2004l\x1b[?1l\x1b>\x1b[<u\x1b[?1049l\x1b[8h"
        );
    }

    #[test]
    fn cursor_to_is_one_based() {
        assert_eq!(cursor_to(0, 0), "\x1b[1;1H");
        assert_eq!(cursor_to(u16::MAX, 4), "\x1b[65536;5H");
    }
}
