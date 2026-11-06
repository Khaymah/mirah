use super::event::{Event, TabId};
use super::pty::ChildProcess;
use std::io::{Read, Write};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

const PTY_READ_BUFFER: usize = 65_536;
const STDIN_READ_BUFFER: usize = 4_096;
const WAIT_FAILED: u32 = 1;

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

pub(crate) fn spawn_pty_reader(
    mut reader: Box<dyn Read + Send>,
    tab: TabId,
    events: Sender<Event>,
) {
    std::thread::spawn(move || {
        let mut buf = vec![0u8; PTY_READ_BUFFER];
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => {
                    let _ = events.send(Event::OutputClosed(tab));
                    break;
                }
                Ok(n) => {
                    if events.send(Event::Output(tab, buf[..n].to_vec())).is_err() {
                        break;
                    }
                }
            }
        }
    });
}

pub(crate) fn spawn_stdin_reader(events: Sender<Event>) {
    std::thread::spawn(move || {
        let mut stdin = std::io::stdin();
        let mut buf = vec![0u8; STDIN_READ_BUFFER];
        while let Ok(n) = stdin.read(&mut buf) {
            if n == 0 || events.send(Event::Input(buf[..n].to_vec())).is_err() {
                break;
            }
        }
    });
}

pub(crate) fn spawn_child_waiter(mut child: ChildProcess, tab: TabId, events: Sender<Event>) {
    std::thread::spawn(move || {
        let code = match child.wait() {
            Ok(status) => {
                if let Some(signal) = status.signal() {
                    tracing::info!(signal, "child terminated by signal");
                }
                status.exit_code()
            }
            Err(err) => {
                tracing::warn!(error = %err, "wait for child");
                WAIT_FAILED
            }
        };
        let _ = events.send(Event::ChildExited(tab, code));
    });
}
