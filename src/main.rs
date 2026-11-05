mod bridge;
mod render;
mod segment;
mod shape;

use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::{Read, Write};
use std::num::NonZeroU16;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

enum Msg {
    Output(Vec<u8>),
    Resize,
    Exit,
}

const ENTER: &str = "\x1b[?1049h\x1b[8l\x1b[2J";
const LEAVE: &str = "\x1b[?2026l\x1b[0m\x1b[?25h\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l\x1b[?2004l\x1b[?1l\x1b>\x1b[<u\x1b[?1049l\x1b[8h";

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self> {
        crossterm::terminal::enable_raw_mode().context("enable raw mode")?;
        let mut out = std::io::stdout();
        out.write_all(ENTER.as_bytes())?;
        out.flush()?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut out = std::io::stdout();
        let _ = out.write_all(LEAVE.as_bytes());
        let _ = out.flush();
        let _ = crossterm::terminal::disable_raw_mode();
    }
}

fn host_size() -> (NonZeroU16, NonZeroU16) {
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
    (
        NonZeroU16::new(rows).unwrap_or(NonZeroU16::MIN),
        NonZeroU16::new(cols).unwrap_or(NonZeroU16::MIN),
    )
}

fn pty_size((rows, cols): (NonZeroU16, NonZeroU16)) -> PtySize {
    PtySize {
        rows: rows.get(),
        cols: cols.get(),
        pixel_width: 0,
        pixel_height: 0,
    }
}

fn command() -> CommandBuilder {
    let mut args = std::env::args().skip(1);
    let program = args
        .next()
        .unwrap_or_else(|| std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into()));
    let mut cmd = CommandBuilder::new(program);
    cmd.args(args);
    if let Ok(dir) = std::env::current_dir() {
        cmd.cwd(dir);
    }
    cmd
}

fn run() -> Result<u32> {
    let size = host_size();
    let pair = native_pty_system()
        .openpty(pty_size(size))
        .context("open pty")?;
    let mut child = pair
        .slave
        .spawn_command(command())
        .context("spawn command")?;
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader()?;
    let writer = Arc::new(Mutex::new(pair.master.take_writer()?));
    let (tx, rx) = mpsc::channel::<Msg>();

    let out_tx = tx.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 65536];
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => {
                    let _ = out_tx.send(Msg::Exit);
                    break;
                }
                Ok(n) => {
                    if out_tx.send(Msg::Output(buf[..n].to_vec())).is_err() {
                        break;
                    }
                }
            }
        }
    });

    let input_writer = Arc::clone(&writer);
    std::thread::spawn(move || {
        let mut stdin = std::io::stdin();
        let mut buf = [0u8; 4096];
        while let Ok(n) = stdin.read(&mut buf) {
            if n == 0 {
                break;
            }
            let Ok(mut w) = input_writer.lock() else {
                break;
            };
            if w.write_all(&buf[..n]).and_then(|()| w.flush()).is_err() {
                break;
            }
        }
    });

    let resize_tx = tx;
    let mut signals = signal_hook::iterator::Signals::new([signal_hook::consts::SIGWINCH])?;
    std::thread::spawn(move || {
        for _ in signals.forever() {
            if resize_tx.send(Msg::Resize).is_err() {
                break;
            }
        }
    });

    let _guard = TerminalGuard::enter()?;
    let mut parser =
        vt100::Parser::new_with_callbacks(size.0, size.1, 0, bridge::Bridge::default());
    let mut renderer = render::Renderer::new();
    let mut stdout = std::io::stdout();

    'outer: while let Ok(first) = rx.recv() {
        let mut pending = vec![first];
        while let Ok(more) = rx.try_recv() {
            pending.push(more);
        }
        for msg in pending {
            match msg {
                Msg::Output(bytes) => parser.process(&bytes),
                Msg::Resize => {
                    let size = host_size();
                    let _ = pair.master.resize(pty_size(size));
                    parser.set_size(size.0, size.1);
                    renderer.invalidate();
                }
                Msg::Exit => break 'outer,
            }
        }

        let bridge = parser.callbacks_mut();
        let to_host = std::mem::take(&mut bridge.to_host);
        let to_child = std::mem::take(&mut bridge.to_child);
        if !to_child.is_empty()
            && let Ok(mut w) = writer.lock()
        {
            let _ = w.write_all(&to_child).and_then(|()| w.flush());
        }
        let frame = renderer.frame(parser.screen());
        stdout.write_all(&to_host)?;
        stdout.write_all(frame.as_bytes())?;
        stdout.flush()?;
    }

    let status = child.wait().context("wait for child")?;
    Ok(status.exit_code())
}

fn main() {
    let code = match run() {
        Ok(code) => code,
        Err(err) => {
            let _ = crossterm::terminal::disable_raw_mode();
            eprintln!("mirah: {err:#}");
            1
        }
    };
    std::process::exit(i32::try_from(code).unwrap_or(1));
}
