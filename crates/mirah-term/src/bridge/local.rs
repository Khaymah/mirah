use crate::sequences::CSI;

const DEVICE_STATUS: u16 = 5;
const CURSOR_POSITION: u16 = 6;
const STATUS_OK: &str = "\x1b[0n";
const TEXT_AREA_SIZE: u16 = 18;
const SCREEN_SIZE: u16 = 19;
const TEXT_AREA_REPLY: u16 = 8;
const SCREEN_REPLY: u16 = 9;
pub(super) const NO_KEYBOARD_FLAGS: &[u8] = b"\x1b[?0u";

pub(super) fn window_size_report(screen: &vt100::Screen, request: u16) -> Option<Vec<u8>> {
    let kind = match request {
        TEXT_AREA_SIZE => TEXT_AREA_REPLY,
        SCREEN_SIZE => SCREEN_REPLY,
        _ => return None,
    };
    let (rows, cols) = screen.size();
    Some(format!("{CSI}{kind};{rows};{cols}t").into_bytes())
}

pub(super) fn device_status_report(screen: &vt100::Screen, request: u16) -> Option<Vec<u8>> {
    match request {
        CURSOR_POSITION => {
            let (row, col) = screen.cursor_position();
            let reply = format!("{CSI}{};{}R", u32::from(row) + 1, u32::from(col) + 1);
            Some(reply.into_bytes())
        }
        DEVICE_STATUS => Some(STATUS_OK.as_bytes().to_vec()),
        _ => None,
    }
}
