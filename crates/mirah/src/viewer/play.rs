use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

const FRAME_END: &[u8] = b"\x1b[?2026l";
const CHUNK: usize = 4_096;
const LONGEST_FRAME: usize = CHUNK * 64;
const FRAME_DELAY: Duration = Duration::from_millis(30);
const FAST_FACTOR: u32 = 8;
const IDLE_POLL: Duration = Duration::from_millis(500);
const LEAVE: &str = "\x1b[?2026l\x1b[0m\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l\x1b[?1004l\x1b[?2004l\x1b[?25h\x1b[?1049l";
const DONE: &str = "انتهى التسجيل · Esc للإغلاق · R للإعادة";
const DONE_STYLE: &str = "\x1b[0;48;2;30;31;35;38;2;217;145;104m";

const ESC: u8 = 0x1b;
const BEL: u8 = 0x07;
const INPUT_MODES: [u16; 10] = [1000, 1002, 1003, 1004, 1005, 1006, 1015, 1016, 2004, 9001];

fn csi_end(bytes: &[u8], start: usize) -> Option<usize> {
    bytes[start..]
        .iter()
        .position(|b| (0x40..=0x7e).contains(b))
        .map(|p| start + p)
}

fn osc_end(bytes: &[u8], start: usize) -> Option<(usize, usize)> {
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            BEL => return Some((i, i + 1)),
            ESC if bytes.get(i + 1) == Some(&b'\\') => return Some((i, i + 2)),
            _ => i += 1,
        }
    }
    None
}

fn kept_csi(body: &[u8], last: u8) -> Option<Vec<u8>> {
    let private = body.first().copied();
    let asks = match last {
        b'n' | b'c' | b't' => true,
        b'q' => private == Some(b'>'),
        b'u' => matches!(private, Some(b'?' | b'>' | b'<' | b'=')),
        b'p' => body.contains(&b'$'),
        _ => false,
    };
    if asks {
        return None;
    }
    if private == Some(b'?') && matches!(last, b'h' | b'l') {
        let params: Vec<&[u8]> = body[1..]
            .split(|b| *b == b';')
            .filter(|p| {
                std::str::from_utf8(p)
                    .ok()
                    .and_then(|t| t.parse::<u16>().ok())
                    .is_none_or(|n| !INPUT_MODES.contains(&n))
            })
            .collect();
        if params.is_empty() {
            return None;
        }
        let mut out = b"\x1b[?".to_vec();
        out.extend(params.join(&b';'));
        out.push(last);
        return Some(out);
    }
    let mut out = b"\x1b[".to_vec();
    out.extend_from_slice(body);
    out.push(last);
    Some(out)
}

pub(crate) fn quiet(bytes: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != ESC {
            out.push(bytes[i]);
            i += 1;
            continue;
        }
        match bytes.get(i + 1) {
            Some(b'[') => match csi_end(bytes, i + 2) {
                Some(end) => {
                    if let Some(kept) = kept_csi(&bytes[i + 2..end], bytes[end]) {
                        out.extend(kept);
                    }
                    i = end + 1;
                }
                None => break,
            },
            Some(b']') => match osc_end(bytes, i + 2) {
                Some((body_end, next)) => {
                    let body = &bytes[i + 2..body_end];
                    if !body.starts_with(b"52;") && !body.ends_with(b"?") {
                        out.extend_from_slice(&bytes[i..next]);
                    }
                    i = next;
                }
                None => break,
            },
            _ => {
                out.push(ESC);
                i += 1;
            }
        }
    }
    out
}

pub(crate) fn frames(bytes: &[u8]) -> Vec<&[u8]> {
    let mut out = Vec::new();
    let mut start = 0;
    while start < bytes.len() {
        let rest = &bytes[start..];
        let window = &rest[..rest.len().min(LONGEST_FRAME)];
        let end = window
            .windows(FRAME_END.len())
            .position(|w| w == FRAME_END)
            .map(|p| p + FRAME_END.len())
            .unwrap_or_else(|| rest.len().min(CHUNK));
        out.push(&rest[..end]);
        start += end;
    }
    out
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Control {
    Go,
    Quit,
    Restart,
}

struct Player {
    paused: bool,
    fast: bool,
    skip: bool,
}

impl Player {
    fn key(&mut self, event: &Event) -> Control {
        let Event::Key(key) = event else {
            return Control::Go;
        };
        if key.kind == KeyEventKind::Release {
            return Control::Go;
        }
        match key.code {
            KeyCode::Esc => return Control::Quit,
            KeyCode::Right => self.fast = !self.fast,
            KeyCode::End => self.skip = true,
            KeyCode::Char(c) => match crate::chrome::keys::latin(c).to_ascii_lowercase() {
                'q' => return Control::Quit,
                'r' => return Control::Restart,
                ' ' => self.paused = !self.paused,
                _ => {}
            },
            _ => {}
        }
        Control::Go
    }

    fn wait(&mut self, delay: Duration) -> std::io::Result<Control> {
        loop {
            let wait = if self.paused { IDLE_POLL } else { delay };
            if event::poll(wait)? {
                let control = self.key(&event::read()?);
                if control != Control::Go {
                    return Ok(control);
                }
                if self.paused || self.skip {
                    continue;
                }
            }
            if !self.paused {
                return Ok(Control::Go);
            }
        }
    }
}

fn reset(out: &mut impl Write) -> std::io::Result<()> {
    out.write_all(b"\x1b[0m\x1b[?1049l\x1b[2J\x1b[H")?;
    out.flush()
}

fn play_once(bytes: &[u8], out: &mut impl Write) -> std::io::Result<Control> {
    let mut player = Player {
        paused: false,
        fast: false,
        skip: false,
    };
    reset(out)?;
    let bytes = quiet(bytes);
    for frame in frames(&bytes) {
        out.write_all(frame)?;
        out.flush()?;
        if player.skip {
            continue;
        }
        let delay = if player.fast {
            FRAME_DELAY / FAST_FACTOR
        } else {
            FRAME_DELAY
        };
        let control = player.wait(delay)?;
        if control != Control::Go {
            return Ok(control);
        }
    }
    let (_, rows) = terminal::size().unwrap_or((80, 24));
    out.write_all(
        format!("\x1b[?2026l\x1b[{rows};1H\x1b[2K{DONE_STYLE} {DONE} \x1b[K\x1b[0m").as_bytes(),
    )?;
    out.flush()?;
    loop {
        let control = player.key(&event::read()?);
        if control != Control::Go {
            return Ok(control);
        }
    }
}

pub(crate) fn run(path: &Path) -> u32 {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) => {
            let mut out = std::io::stdout();
            let _ = writeln!(out, "تعذّر فتح التسجيل {}: {err}", path.display());
            let _ = out.flush();
            std::thread::sleep(Duration::from_secs(3));
            return 1;
        }
    };
    if terminal::enable_raw_mode().is_err() {
        return 1;
    }
    let mut out = std::io::stdout();
    let mut result = Ok(Control::Restart);
    while matches!(result, Ok(Control::Restart)) {
        result = play_once(&bytes, &mut out);
    }
    let _ = out.write_all(LEAVE.as_bytes());
    let _ = out.flush();
    let _ = terminal::disable_raw_mode();
    u32::from(result.is_err())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_split_after_each_synchronized_update() {
        let bytes = b"a\x1b[?2026lbb\x1b[?2026lccc";
        let parts = frames(bytes);
        assert_eq!(
            parts,
            [b"a\x1b[?2026l".as_slice(), b"bb\x1b[?2026l", b"ccc"]
        );
        assert_eq!(parts.concat(), bytes);
    }

    #[test]
    fn questions_input_modes_and_copies_are_left_out() {
        let recorded = b"\x1b[1t\x1b[6n\x1b[c\x1b[?1004h\x1b[?9001h\x1b[?1049h\x1b[?1003;1006;25h\x1b[31mhi\x1b]52;c;aGk=\x07\x1b]11;?\x1b\\\x1b]0;title\x07\x1b[>1u\x1b[?2026$p";
        assert_eq!(
            quiet(recorded),
            b"\x1b[?1049h\x1b[?25h\x1b[31mhi\x1b]0;title\x07"
        );
    }

    #[test]
    fn output_without_frames_is_cut_into_chunks() {
        let bytes = vec![b'x'; CHUNK * 2 + 10];
        let parts = frames(&bytes);
        assert_eq!(parts.len(), 3);
        assert_eq!(parts.concat(), bytes);
    }
}
