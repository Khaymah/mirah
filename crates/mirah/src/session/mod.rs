mod event;
mod io;
mod pty;

use crate::host::{self, TerminalGuard};
use anyhow::Result;
use event::Event;
use io::SharedWriter;
use mirah_term::Emulator;
use portable_pty::CommandBuilder;
use pty::Pty;
use std::io::Write;
use std::ops::ControlFlow;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub(crate) fn run(cmd: CommandBuilder) -> Result<u32> {
    let size = host::size();
    let pty = Pty::spawn(cmd, size)?;
    let (events, inbox) = mpsc::channel();
    io::spawn_pty_reader(pty.reader()?, events.clone());
    let writer: SharedWriter = Arc::new(Mutex::new(pty.writer()?));
    io::spawn_stdin_forwarder(Arc::clone(&writer));
    io::spawn_resize_watcher(events)?;

    let _guard = TerminalGuard::enter()?;
    tracing::info!(
        rows = size.rows,
        cols = size.cols,
        term = std::env::var("TERM").unwrap_or_default(),
        term_program = std::env::var("TERM_PROGRAM").unwrap_or_default(),
        "session started with explicit bidi mode"
    );
    let mut emulator = Emulator::new(size.rows, size.cols)?;
    while let Ok(first) = inbox.recv() {
        let flow = apply(drain(first, &inbox), &pty, &mut emulator)?;
        flush(&mut emulator, &writer)?;
        if flow.is_break() {
            break;
        }
    }
    let code = pty.wait()?;
    tracing::info!(code, "session ended");
    Ok(code)
}

fn drain(first: Event, inbox: &Receiver<Event>) -> Vec<Event> {
    let mut events = vec![first];
    events.extend(inbox.try_iter());
    events
}

fn apply(events: Vec<Event>, pty: &Pty, emulator: &mut Emulator) -> Result<ControlFlow<()>> {
    for event in events {
        match event {
            Event::Output(bytes) => emulator.feed(&bytes),
            Event::Resize => {
                let size = host::size();
                pty.resize(size);
                emulator.resize(size.rows, size.cols)?;
                tracing::debug!(rows = size.rows, cols = size.cols, "resized");
            }
            Event::ChildExited => return Ok(ControlFlow::Break(())),
        }
    }
    Ok(ControlFlow::Continue(()))
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
