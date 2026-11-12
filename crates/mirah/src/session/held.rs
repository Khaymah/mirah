use std::time::{Duration, Instant};

const ESC: u8 = 0x1b;
const BEL: u8 = 0x07;
const HOLD: Duration = Duration::from_millis(30);
const LOOK_BACK: usize = 64;

fn unfinished(tail: &[u8]) -> bool {
    match tail.get(1) {
        None => true,
        Some(b'[') => tail[2..].iter().all(|b| (0x20..=0x3f).contains(b)),
        Some(b'O') => tail.len() == 2,
        Some(b']') => !tail.contains(&BEL) && !tail[2..].windows(2).any(|w| w == [ESC, b'\\']),
        Some(_) => false,
    }
}

pub(super) fn split(bytes: &[u8]) -> usize {
    let from = bytes.len().saturating_sub(LOOK_BACK);
    let window = &bytes[from..];
    let Some(at) = window.iter().rposition(|b| *b == ESC) else {
        return bytes.len();
    };
    if !unfinished(&window[at..]) {
        return bytes.len();
    }
    let open_osc = (at + 1 == window.len())
        .then(|| window[..at].iter().rposition(|b| *b == ESC))
        .flatten()
        .filter(|&osc| window.get(osc + 1) == Some(&b']') && unfinished(&window[osc..at]));
    from + open_osc.unwrap_or(at)
}

#[derive(Default)]
pub(super) struct Held {
    bytes: Vec<u8>,
    until: Option<Instant>,
}

impl Held {
    pub(super) fn take_ready(&mut self, incoming: &[u8], now: Instant) -> Vec<u8> {
        let mut data = std::mem::take(&mut self.bytes);
        data.extend_from_slice(incoming);
        let cut = split(&data);
        if cut < data.len() {
            self.bytes = data.split_off(cut);
            self.until = Some(now + HOLD);
        } else {
            self.until = None;
        }
        data
    }

    pub(super) fn expired(&mut self, now: Instant) -> Option<Vec<u8>> {
        if self.until.is_some_and(|t| now >= t) {
            self.until = None;
            return Some(std::mem::take(&mut self.bytes));
        }
        None
    }

    pub(super) fn deadline(&self) -> Option<Instant> {
        self.until
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_input_passes_at_once() {
        for bytes in [
            &b"abc"[..],
            b"\x1b[A",
            b"\x1bOB",
            b"x\x1b[<0;3;4M",
            b"\x1b]11;?\x07",
        ] {
            assert_eq!(split(bytes), bytes.len(), "{bytes:?}");
        }
    }

    #[test]
    fn a_cut_sequence_is_held() {
        assert_eq!(split(b"ab\x1b"), 2);
        assert_eq!(split(b"\x1b["), 0);
        assert_eq!(split(b"q\x1b[<0;12"), 1);
        assert_eq!(split(b"\x1bO"), 0);
        assert_eq!(split(b"k\x1b]11;rgb:1/2/3\x1b"), 1);
        assert_eq!(split(b"k\x1b]11;rgb:1/2/3\x1b\\x\x1b"), 18);
        assert_eq!(split(b"\x1b[A\x1b"), 3);
    }

    #[test]
    fn held_bytes_join_the_next_read_or_leave_after_a_moment() {
        let now = Instant::now();
        let mut held = Held::default();
        assert_eq!(held.take_ready(b"x\x1b", now), b"x");
        assert_eq!(held.take_ready(b"[A", now), b"\x1b[A");
        assert_eq!(held.deadline(), None);
        assert_eq!(held.take_ready(b"\x1b", now), b"");
        assert_eq!(held.expired(now), None);
        assert_eq!(held.expired(now + HOLD), Some(b"\x1b".to_vec()));
    }
}
