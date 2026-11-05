use crate::sequences::CSI;

const DEVICE_STATUS: u16 = 5;
const CURSOR_POSITION: u16 = 6;
const STATUS_OK: &str = "\x1b[0n";

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
