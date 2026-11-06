mod actions;
mod event;
mod io;
pub(crate) mod mouse;
mod pacing;
mod pty;
mod record;
mod resize;

use crate::chrome::Chrome;
use crate::config::Config;
use crate::host::{self, TerminalGuard, TerminalSize};
use anyhow::Result;
use event::Event;
use io::SharedWriter;
use mirah_term::Emulator;
pub(crate) use mouse::MouseEvent;
use mouse::MouseRemap;
use pacing::Pacer;
use portable_pty::{ChildKiller, CommandBuilder};
use pty::Pty;
use record::Recorder;
use std::io::Write;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
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
    events: Sender<Event>,
    pty: &'a Pty,
    writer: SharedWriter,
    killer: Box<dyn ChildKiller + Send + Sync>,
    emulator: Emulator,
    recorder: Recorder,
    life: Lifecycle,
    pacer: Pacer,
    mouse: MouseRemap,
    chrome: Chrome,
    config: Config,
    program: String,
    cols: u16,
    recording_since: Option<Instant>,
    notice_until: Option<Instant>,
}

fn child_size(size: TerminalSize, top: u16) -> TerminalSize {
    TerminalSize {
        rows: size.rows.saturating_sub(top).max(1),
        cols: size.cols,
    }
}

fn setup(program: &str) -> (Config, mirah_term::RenderOptions, Chrome) {
    let config = Config::load();
    let settings = config.settings(program);
    if settings.update_check && !settings.bar {
        crate::update::startup();
    }
    let options = settings.render_options(program);
    let chrome = Chrome::new(program, settings, options.shaping);
    host::prepare_pty();
    (config, options, chrome)
}

fn spawn_io(pty: &Pty, child: pty::ChildProcess, events: &Sender<Event>) -> Result<SharedWriter> {
    io::spawn_pty_reader(pty.reader()?, events.clone());
    io::spawn_child_waiter(child, events.clone());
    io::spawn_stdin_reader(events.clone());
    resize::spawn_watcher(events.clone())?;
    Ok(Arc::new(Mutex::new(pty.writer()?)))
}

fn check_updates(chrome: &Chrome, events: &Sender<Event>) {
    if !chrome.settings.update_check || !chrome.settings.bar {
        return;
    }
    let events = events.clone();
    std::thread::spawn(move || {
        if let Some(version) = crate::update::available() {
            let _ = events.send(Event::UpdateFound(version));
        }
    });
}

pub(crate) fn run(
    cmd: CommandBuilder,
    program: &str,
    recording: Option<std::path::PathBuf>,
) -> Result<u32> {
    let (config, options, chrome) = setup(program);
    let size = host::size();
    let inner = child_size(size, chrome.top());
    let (pty, child) = Pty::spawn(cmd, inner)?;
    let killer = child.clone_killer();
    let (events, inbox) = mpsc::channel();
    let writer = spawn_io(&pty, child, &events)?;
    check_updates(&chrome, &events);
    let _guard = TerminalGuard::enter()?;
    log_start(size, options);
    let recorder = Recorder::new(recording);
    let mut session = Session {
        recording_since: recorder.active().then(Instant::now),
        inbox,
        events,
        pty: &pty,
        writer,
        killer,
        emulator: Emulator::with_options(inner.rows, inner.cols, options)?,
        recorder,
        life: Lifecycle::default(),
        pacer: Pacer::default(),
        mouse: MouseRemap::default(),
        chrome,
        config,
        program: program.to_string(),
        cols: size.cols,
        notice_until: None,
    };
    session.pump()?;
    let code = session.finish();
    drop(session);
    close_in_background(pty);
    Ok(code)
}

impl Session<'_> {
    fn finish(&mut self) -> u32 {
        let code = self.life.code.unwrap_or(UNKNOWN_EXIT);
        tracing::info!(
            code,
            output_closed = self.life.output_closed,
            "session ended"
        );
        code
    }

    fn pump(&mut self) -> Result<()> {
        let mut deadline: Option<Instant> = None;
        while let Some(events) = self.receive(deadline) {
            if !events.is_empty() {
                self.life.last_event = Some(Instant::now());
            }
            record(&mut self.recorder, &events);
            self.apply(events)?;
            forward(&mut self.emulator, &self.writer)?;
            let now = Instant::now();
            let done = self.life.done(now);
            let tick = self.tick(now);
            deadline = earliest(self.paint(now, done)?, tick);
            if done {
                break;
            }
        }
        Ok(())
    }

    fn paint(&mut self, now: Instant, force: bool) -> Result<Option<Instant>> {
        if !self.emulator.needs_frame() && !self.chrome.dirty {
            return Ok(None);
        }
        let synchronizing = self.emulator.is_synchronizing();
        let ready = self.pacer.ready_at(now, synchronizing);
        if force || now >= ready {
            let overlay = self.chrome.overlay(self.cols);
            draw(&mut self.emulator, &overlay)?;
            self.pacer.drew(now, synchronizing);
            Ok(None)
        } else {
            Ok(Some(ready))
        }
    }

    fn apply(&mut self, events: Vec<Event>) -> Result<()> {
        for event in events {
            match event {
                Event::Output(bytes) => self.emulator.feed(&bytes),
                Event::Input(bytes) => self.input(&bytes)?,
                event @ (Event::UpdateFound(_) | Event::UpdateDone(_)) => self.update_event(event),
                Event::OutputClosed => self.life.output_closed = true,
                Event::ChildExited(code) => self.life.code = Some(code),
                Event::Resize => {
                    let size = host::size();
                    let inner = child_size(size, self.chrome.top());
                    self.pty.resize(inner);
                    self.emulator.resize(inner.rows, inner.cols)?;
                    self.cols = size.cols;
                    self.chrome.dirty = true;
                    tracing::debug!(rows = size.rows, cols = size.cols, "resized");
                }
            }
        }
        Ok(())
    }

    fn receive(&self, deadline: Option<Instant>) -> Option<Vec<Event>> {
        let exit = self.life.code.map(|_| Instant::now() + EXIT_GRACE);
        let until = earliest(deadline, exit);
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

fn earliest(a: Option<Instant>, b: Option<Instant>) -> Option<Instant> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
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

fn draw(emulator: &mut Emulator, overlay: &mirah_term::Overlay) -> Result<()> {
    let started = Instant::now();
    let frame = emulator.frame_with(overlay);
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
