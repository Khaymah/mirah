mod event;
mod io;
mod pty;
mod resize;

use crate::host::{self, TerminalGuard};
use anyhow::Result;
use event::Event;
use io::SharedWriter;
use mirah_term::Emulator;
use portable_pty::CommandBuilder;
use pty::Pty;
use std::io::Write;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const EXIT_GRACE: Duration = Duration::from_millis(100);
const UNKNOWN_EXIT: u32 = 1;

#[derive(Default)]
struct Lifecycle {
    code: Option<u32>,
    output_closed: bool,
}

pub(crate) fn run(cmd: CommandBuilder) -> Result<u32> {
    let size = host::size();
    let (pty, child) = Pty::spawn(cmd, size)?;
    let (events, inbox) = mpsc::channel();
    io::spawn_pty_reader(pty.reader()?, events.clone());
    io::spawn_child_waiter(child, events.clone());
    let writer: SharedWriter = Arc::new(Mutex::new(pty.writer()?));
    io::spawn_stdin_forwarder(Arc::clone(&writer));
    resize::spawn_watcher(events)?;

    let _guard = TerminalGuard::enter()?;
    tracing::info!(
        rows = size.rows,
        cols = size.cols,
        term = std::env::var("TERM").unwrap_or_default(),
        term_program = std::env::var("TERM_PROGRAM").unwrap_or_default(),
        "session started with explicit bidi mode"
    );
    let mut emulator = Emulator::new(size.rows, size.cols)?;
    let mut life = Lifecycle::default();
    while let Some(first) = next_event(&inbox, &life) {
        apply(drain(first, &inbox), &pty, &mut emulator, &mut life)?;
        flush(&mut emulator, &writer)?;
        if life.code.is_some() && life.output_closed {
            break;
        }
    }
    let code = life.code.unwrap_or(UNKNOWN_EXIT);
    tracing::info!(code, output_closed = life.output_closed, "session ended");
    close_in_background(pty);
    Ok(code)
}

fn close_in_background(pty: Pty) {
    std::thread::spawn(move || {
        drop(pty);
        tracing::debug!("pty closed");
    });
}

fn next_event(inbox: &Receiver<Event>, life: &Lifecycle) -> Option<Event> {
    if life.code.is_some() {
        inbox.recv_timeout(EXIT_GRACE).ok()
    } else {
        inbox.recv().ok()
    }
}

fn drain(first: Event, inbox: &Receiver<Event>) -> Vec<Event> {
    let mut events = vec![first];
    events.extend(inbox.try_iter());
    events
}

fn apply(
    events: Vec<Event>,
    pty: &Pty,
    emulator: &mut Emulator,
    life: &mut Lifecycle,
) -> Result<()> {
    for event in events {
        match event {
            Event::Output(bytes) => emulator.feed(&bytes),
            Event::OutputClosed => life.output_closed = true,
            Event::ChildExited(code) => life.code = Some(code),
            Event::Resize => {
                let size = host::size();
                pty.resize(size);
                emulator.resize(size.rows, size.cols)?;
                tracing::debug!(rows = size.rows, cols = size.cols, "resized");
            }
        }
    }
    Ok(())
}

fn flush(emulator: &mut Emulator, writer: &SharedWriter) -> Result<()> {
    let started = Instant::now();
    let output = emulator.take_output();
    if !output.to_child.is_empty() {
        io::write_to_child(writer, &output.to_child);
    }
    let frame = emulator.frame();
    let mut stdout = std::io::stdout();
    stdout.write_all(&output.to_host)?;
    stdout.write_all(frame.as_bytes())?;
    stdout.flush()?;
    tracing::debug!(
        bytes = frame.len(),
        micros = started.elapsed().as_micros(),
        "frame"
    );
    Ok(())
}
