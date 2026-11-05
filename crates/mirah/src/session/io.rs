use super::event::Event;
use anyhow::Result;
use std::io::{Read, Write};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

const PTY_READ_BUFFER: usize = 65_536;
const STDIN_READ_BUFFER: usize = 4_096;

pub(crate) type SharedWriter = Arc<Mutex<Box<dyn Write + Send>>>;

pub(crate) fn write_to_child(writer: &SharedWriter, bytes: &[u8]) {
    let Ok(mut w) = writer.lock() else {
        tracing::warn!("pty writer lock poisoned");
        return;
    };
    if let Err(err) = w.write_all(bytes).and_then(|()| w.flush()) {
        tracing::warn!(error = %err, "write to child");
    }
}

pub(crate) fn spawn_pty_reader(mut reader: Box<dyn Read + Send>, events: Sender<Event>) {
    std::thread::spawn(move || {
        let mut buf = vec![0u8; PTY_READ_BUFFER];
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => {
                    let _ = events.send(Event::ChildExited);
                    break;
                }
                Ok(n) => {
                    if events.send(Event::Output(buf[..n].to_vec())).is_err() {
                        break;
                    }
                }
            }
        }
    });
}

pub(crate) fn spawn_stdin_forwarder(writer: SharedWriter) {
    std::thread::spawn(move || {
        let mut stdin = std::io::stdin();
        let mut buf = vec![0u8; STDIN_READ_BUFFER];
        while let Ok(n) = stdin.read(&mut buf) {
            if n == 0 {
                break;
            }
            write_to_child(&writer, &buf[..n]);
        }
    });
}

pub(crate) fn spawn_resize_watcher(events: Sender<Event>) -> Result<()> {
    let mut signals = signal_hook::iterator::Signals::new([signal_hook::consts::SIGWINCH])?;
    std::thread::spawn(move || {
        for _ in signals.forever() {
            if events.send(Event::Resize).is_err() {
                break;
            }
        }
    });
    Ok(())
}
