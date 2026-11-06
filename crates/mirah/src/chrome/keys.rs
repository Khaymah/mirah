use std::ops::Range;

const CTRL_RIGHT_BRACKET: u8 = 0x1d;
const ESC: u8 = 0x1b;
const WIN32_FINAL: u8 = b'_';
const VK_TAB: u32 = 0x09;
const VK_RETURN: u32 = 0x0d;
const VK_ESCAPE: u32 = 0x1b;
const VK_LEFT: u32 = 0x25;
const VK_UP: u32 = 0x26;
const VK_RIGHT: u32 = 0x27;
const VK_DOWN: u32 = 0x28;
const KITTY_RIGHT_BRACKET: u32 = 93;
const KITTY_CTRL: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Key {
    Toggle,
    Left,
    Right,
    Up,
    Down,
    Enter,
    Escape,
    Tab,
    Release,
    Char(char),
    Other,
}

fn numbers(body: &[u8]) -> Vec<u32> {
    body.split(|b| *b == b';')
        .map(|part| {
            let head = part.split(|b| *b == b':').next().unwrap_or_default();
            std::str::from_utf8(head)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0)
        })
        .collect()
}

fn printable(code: u32) -> Key {
    match char::from_u32(code) {
        Some(c) if !c.is_control() && code != 0 => Key::Char(c),
        _ => Key::Other,
    }
}

fn single(run: &[u8]) -> Key {
    let mut chars = std::str::from_utf8(run).ok().map(str::chars);
    match chars.as_mut().map(|c| (c.next(), c.next())) {
        Some((Some(c), None)) if !c.is_control() => Key::Char(c),
        _ => Key::Other,
    }
}

fn win32(params: &[u32]) -> Key {
    let (vk, uc, down) = (
        params.first().copied().unwrap_or(0),
        params.get(2).copied().unwrap_or(0),
        params.get(3).copied().unwrap_or(1),
    );
    if down == 0 {
        return Key::Release;
    }
    if uc == u32::from(CTRL_RIGHT_BRACKET) {
        return Key::Toggle;
    }
    match vk {
        VK_LEFT => Key::Left,
        VK_RIGHT => Key::Right,
        VK_UP => Key::Up,
        VK_DOWN => Key::Down,
        VK_RETURN => Key::Enter,
        VK_ESCAPE => Key::Escape,
        VK_TAB => Key::Tab,
        _ => printable(uc),
    }
}

fn csi(body: &[u8], last: u8) -> Key {
    let params = numbers(body);
    match last {
        b'A' => Key::Up,
        b'B' => Key::Down,
        b'C' => Key::Right,
        b'D' => Key::Left,
        b'Z' => Key::Tab,
        b'u' => match (params.first().copied(), params.get(1).copied()) {
            (Some(KITTY_RIGHT_BRACKET), Some(KITTY_CTRL)) => Key::Toggle,
            (Some(13), _) => Key::Enter,
            (Some(27), _) => Key::Escape,
            (Some(9), _) => Key::Tab,
            (Some(code), None | Some(1)) => printable(code),
            _ => Key::Other,
        },
        WIN32_FINAL => win32(&params),
        _ => Key::Other,
    }
}

fn escape(bytes: &[u8]) -> (Key, usize) {
    match bytes.get(1) {
        None => (Key::Escape, 1),
        Some(b'[') => {
            let body_len = bytes[2..]
                .iter()
                .take_while(|b| matches!(b, b'0'..=b'9' | b';' | b':' | b'?' | b'<'))
                .count();
            match bytes.get(2 + body_len) {
                Some(&last) => (csi(&bytes[2..2 + body_len], last), 3 + body_len),
                None => (Key::Other, bytes.len()),
            }
        }
        Some(b'O') => match bytes.get(2) {
            Some(b'A') => (Key::Up, 3),
            Some(b'B') => (Key::Down, 3),
            Some(b'C') => (Key::Right, 3),
            Some(b'D') => (Key::Left, 3),
            Some(_) => (Key::Other, 3),
            None => (Key::Other, 2),
        },
        Some(_) => (Key::Other, 2),
    }
}

pub(crate) fn decode(bytes: &[u8]) -> Vec<(Key, Range<usize>)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let (key, len) = match bytes[i] {
            CTRL_RIGHT_BRACKET => (Key::Toggle, 1),
            ESC => escape(&bytes[i..]),
            b'\r' | b'\n' => (Key::Enter, 1),
            b'\t' => (Key::Tab, 1),
            _ => {
                let len = bytes[i..]
                    .iter()
                    .take_while(|b| {
                        !matches!(**b, CTRL_RIGHT_BRACKET | ESC | b'\r' | b'\n' | b'\t')
                    })
                    .count()
                    .max(1);
                (single(&bytes[i..i + len]), len)
            }
        };
        out.push((key, i..i + len));
        i += len;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(bytes: &[u8]) -> Vec<Key> {
        decode(bytes).into_iter().map(|(k, _)| k).collect()
    }

    #[test]
    fn reads_plain_keys() {
        assert_eq!(
            keys(b"ab\x1d\x1b[D\x1bOC\r\x1bt"),
            [
                Key::Other,
                Key::Toggle,
                Key::Left,
                Key::Right,
                Key::Enter,
                Key::Other
            ]
        );
    }

    #[test]
    fn reads_windows_terminal_win32_input_mode() {
        assert_eq!(
            keys(b"\x1b[221;27;29;1;8;1_\x1b[221;27;29;0;8;1_"),
            [Key::Toggle, Key::Release]
        );
        assert_eq!(
            keys(b"\x1b[37;75;0;1;0;1_\x1b[13;28;13;1;0;1_"),
            [Key::Left, Key::Enter]
        );
        assert_eq!(keys(b"\x1b[65;30;97;1;0;1_"), [Key::Char('a')]);
        assert_eq!(keys(b"\x1b[16;42;0;1;0;1_"), [Key::Other]);
    }

    #[test]
    fn reads_kitty_keys() {
        assert_eq!(
            keys(b"\x1b[93;5u\x1b[27u\x1b[13u"),
            [Key::Toggle, Key::Escape, Key::Enter]
        );
    }

    #[test]
    fn ranges_cover_every_byte_once() {
        let input = b"hi\x1b[1;5C\x1d\x1b[65;30;97;1;0;1_x";
        let mut end = 0;
        for (_, range) in decode(input) {
            assert_eq!(range.start, end);
            end = range.end;
        }
        assert_eq!(end, input.len());
    }
}
