mod event;
mod io;
mod pacing;
mod pty;
mod record;
mod resize;

use crate::host::{self, TerminalGuard};
use anyhow::Result;
use event::Event;
use io::SharedWriter;
use mirah_term::Emulator;
use pacing::Pacer;
use portable_pty::CommandBuilder;
use pty::Pty;
use record::Recorder;
use std::io::Write;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const EXIT_GRACE: Duration = Duration::from_millis(100);
const UNKNOWN_EXIT: u32 = 1;

#[derive(Default)]
struct Lifecycle {
    code: Option<u32>,
    output_closed: bool,
    last_event: Option<Instant>,
}

impl Lifecycle {
    fn done(&self, now: Instant) -> bool {
        self.code.is_some()
            && (self.output_closed || self.last_event.is_some_and(|t| now >= t + EXIT_GRACE))
    }
}

struct Session<'a> {
    inbox: Receiver<Event>,
    pty: &'a Pty,
    writer: SharedWriter,
    emulator: Emulator,
    recorder: Recorder,
    life: Lifecycle,
    pacer: Pacer,
}

pub(crate) fn run(
    cmd: CommandBuilder,
    program: &str,
    recording: Option<std::path::PathBuf>,
) -> Result<u32> {
    host::prepare_pty();
    let size = host::size();
    let (pty, child) = Pty::spawn(cmd, size)?;
    let (events, inbox) = mpsc::channel();
    io::spawn_pty_reader(pty.reader()?, events.clone());
    io::spawn_child_waiter(child, events.clone());
    let writer: SharedWriter = Arc::new(Mutex::new(pty.writer()?));
    io::spawn_stdin_forwarder(Arc::clone(&writer));
    resize::spawn_watcher(events)?;

    let _guard = TerminalGuard::enter()?;
    let options = host::render_options(std::ffi::OsStr::new(program));
    log_start(size, options);
    let mut session = Session {
        inbox,
        pty: &pty,
        writer,
        emulator: Emulator::with_options(size.rows, size.cols, options)?,
        recorder: Recorder::new(recording),
        life: Lifecycle::default(),
        pacer: Pacer::default(),
    };
    session.pump()?;
    let code = session.life.code.unwrap_or(UNKNOWN_EXIT);
    tracing::info!(
        code,
        output_closed = session.life.output_closed,
        "session ended"
    );
    drop(session);
    close_in_background(pty);
    Ok(code)
}

impl Session<'_> {
    fn pump(&mut self) -> Result<()> {
        let mut deadline: Option<Instant> = None;
        while let Some(events) = self.receive(deadline) {
            if !events.is_empty() {
                self.life.last_event = Some(Instant::now());
            }
            record(&mut self.recorder, &events);
            apply(events, self.pty, &mut self.emulator, &mut self.life)?;
            forward(&mut self.emulator, &self.writer)?;
            let now = Instant::now();
            let done = self.life.done(now);
            deadline = self.paint(now, done)?;
            if done {
                break;
            }
        }
        Ok(())
    }

    fn paint(&mut self, now: Instant, force: bool) -> Result<Option<Instant>> {
        if !self.emulator.needs_frame() {
            return Ok(None);
        }
        let synchronizing = self.emulator.is_synchronizing();
        let ready = self.pacer.ready_at(now, synchronizing);
        if force || now >= ready {
            draw(&mut self.emulator)?;
            self.pacer.drew(now, synchronizing);
            Ok(None)
        } else {
            Ok(Some(ready))
        }
    }

    fn receive(&self, deadline: Option<Instant>) -> Option<Vec<Event>> {
        let exit = self.life.code.map(|_| Instant::now() + EXIT_GRACE);
        let until = match (deadline, exit) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        let first = match until {
            None => self.inbox.recv().ok()?,
            Some(until) => {
                match self
                    .inbox
                    .recv_timeout(until.saturating_duration_since(Instant::now()))
                {
                    Ok(event) => event,
                    Err(RecvTimeoutError::Timeout) => return Some(Vec::new()),
                    Err(RecvTimeoutError::Disconnected) => return None,
                }
            }
        };
        Some(drain(first, &self.inbox))
    }
}

fn log_start(size: host::TerminalSize, options: mirah_term::RenderOptions) {
    tracing::info!(
        rows = size.rows,
        cols = size.cols,
        term = std::env::var("TERM").unwrap_or_default(),
        term_program = std::env::var("TERM_PROGRAM").unwrap_or_default(),
        "session started with explicit bidi mode"
    );
    tracing::info!(?options, "render options");
}

fn close_in_background(pty: Pty) {
    std::thread::spawn(move || {
        drop(pty);
        tracing::debug!("pty closed");
    });
}

fn record(recorder: &mut Recorder, events: &[Event]) {
    for event in events {
        if let Event::Output(bytes) = event {
            recorder.write(bytes);
        }
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

fn forward(emulator: &mut Emulator, writer: &SharedWriter) -> Result<()> {
    let output = emulator.take_output();
    if !output.to_child.is_empty() {
        io::write_to_child(writer, &output.to_child);
    }
    if !output.to_host.is_empty() {
        let mut stdout = std::io::stdout();
        stdout.write_all(&output.to_host)?;
        stdout.flush()?;
    }
    Ok(())
}

fn draw(emulator: &mut Emulator) -> Result<()> {
    let started = Instant::now();
    let frame = emulator.frame();
    let mut stdout = std::io::stdout();
    stdout.write_all(frame.as_bytes())?;
    stdout.flush()?;
    tracing::debug!(
        bytes = frame.len(),
        micros = started.elapsed().as_micros(),
        "frame"
    );
    Ok(())
}
