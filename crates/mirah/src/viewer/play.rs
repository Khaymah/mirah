use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

const FRAME_END: &[u8] = b"\x1b[?2026l";
const CHUNK: usize = 4_096;
const FRAME_DELAY: Duration = Duration::from_millis(30);
const FAST_FACTOR: u32 = 8;
const IDLE_POLL: Duration = Duration::from_millis(500);
const LEAVE: &str = "\x1b[?2026l\x1b[0m\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l\x1b[?1004l\x1b[?2004l\x1b[?25h\x1b[?1049l";
const DONE: &str = "انتهى التسجيل · Esc للإغلاق · R للإعادة";
const DONE_STYLE: &str = "\x1b[0;48;2;30;31;35;38;2;217;145;104m";

pub(crate) fn frames(bytes: &[u8]) -> Vec<&[u8]> {
    let mut out = Vec::new();
    let mut start = 0;
    while start < bytes.len() {
        let rest = &bytes[start..];
        let end = rest
            .windows(FRAME_END.len())
            .position(|w| w == FRAME_END)
            .map(|p| p + FRAME_END.len())
            .filter(|&p| p <= CHUNK * 64)
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
    for frame in frames(bytes) {
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
    fn output_without_frames_is_cut_into_chunks() {
        let bytes = vec![b'x'; CHUNK * 2 + 10];
        let parts = frames(&bytes);
        assert_eq!(parts.len(), 3);
        assert_eq!(parts.concat(), bytes);
    }
}
