use super::event::{Event, TabId};
use super::io::{self, SharedWriter};
use super::pty::Pty;
use super::record::Recorder;
use crate::config::{Config, Settings};
use crate::host::TerminalSize;
use crate::store::{SessionMeta, Store};
use anyhow::Result;
use mirah_term::Emulator;
use portable_pty::ChildKiller;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const EXIT_GRACE: Duration = Duration::from_millis(100);
const UNREAD_AFTER: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Spawn {
    pub(crate) agent: String,
    pub(crate) argv: Vec<String>,
    pub(crate) cwd: PathBuf,
    pub(crate) meta: Option<SessionMeta>,
}

#[derive(Default)]
pub(super) struct Lifecycle {
    pub(super) code: Option<u32>,
    pub(super) output_closed: bool,
    pub(super) last_event: Option<Instant>,
}

impl Lifecycle {
    pub(super) fn done(&self, now: Instant) -> bool {
        self.code.is_some()
            && (self.output_closed || self.last_event.is_some_and(|t| now >= t + EXIT_GRACE))
    }

    pub(super) fn grace(&self) -> Option<Instant> {
        self.code.map(|_| Instant::now() + EXIT_GRACE)
    }
}

pub(super) struct Tab {
    pub(super) id: TabId,
    pub(super) name: String,
    pub(super) program: String,
    pub(super) meta: Option<SessionMeta>,
    pty: Option<Pty>,
    writer: SharedWriter,
    killer: Box<dyn ChildKiller + Send + Sync>,
    pub(super) emulator: Emulator,
    pub(super) recorder: Recorder,
    pub(super) life: Lifecycle,
    pub(super) settings: Settings,
    pub(super) unread: bool,
    pub(super) left_at: Option<Instant>,
    pub(super) recording_since: Option<Instant>,
    pub(super) recorded: Duration,
}

pub(super) struct Open<'a> {
    pub(super) id: TabId,
    pub(super) spawn: Spawn,
    pub(super) size: TerminalSize,
    pub(super) config: &'a Config,
    pub(super) store: Option<&'a Store>,
    pub(super) record: bool,
    pub(super) reserved: Option<String>,
    pub(super) events: &'a Sender<Event>,
}

fn session(open: &mut Open<'_>) -> Option<SessionMeta> {
    let store = open.store?;
    let mut meta = match open.spawn.meta.take() {
        Some(meta) => meta,
        None => store.create_with(
            open.reserved.take(),
            &open.spawn.agent,
            &open.spawn.argv,
            &open.spawn.cwd,
        ),
    };
    meta.ended = None;
    store.save(&meta);
    Some(meta)
}

impl Tab {
    pub(super) fn open(mut open: Open<'_>) -> Result<Self> {
        let program = open.spawn.argv.first().cloned().unwrap_or_default();
        let settings = open.config.settings(&program);
        let options = settings.render_options(&program);
        let cmd = crate::command::command(&open.spawn.argv, &open.spawn.cwd);
        let (pty, child) = Pty::spawn(cmd, open.size)?;
        let killer = child.clone_killer();
        io::spawn_pty_reader(pty.reader()?, open.id, open.events.clone());
        io::spawn_child_waiter(child, open.id, open.events.clone());
        let writer = Arc::new(Mutex::new(pty.writer()?));
        let resuming = open.spawn.meta.is_some();
        let meta = session(&mut open);
        let output = meta
            .as_ref()
            .zip(open.store)
            .map(|(m, store)| store.output(&m.id));
        let continuing = resuming && output.as_ref().is_some_and(|p| p.exists());
        let wanted = open.record || settings.record || continuing;
        let path = match output {
            Some(output) => wanted.then_some(output),
            None => {
                wanted.then(|| crate::debug_files::next_in(&open.spawn.cwd, &program).recording)
            }
        };
        let recorder = Recorder::new(path);
        tracing::info!(
            tab = open.id,
            argv = ?open.spawn.argv,
            id = meta.as_ref().map(|m| m.id.as_str()),
            ?options,
            "tab opened"
        );
        Ok(Self {
            id: open.id,
            name: open.spawn.agent.clone(),
            program,
            meta,
            pty: Some(pty),
            writer,
            killer,
            emulator: Emulator::with_options(open.size.rows, open.size.cols, options)?,
            recording_since: recorder.active().then(Instant::now),
            recorder,
            life: Lifecycle::default(),
            settings,
            unread: false,
            left_at: None,
            recorded: Duration::ZERO,
        })
    }

    pub(super) fn feed(&mut self, bytes: &[u8], active: bool) {
        self.recorder.write(bytes);
        self.emulator.feed(bytes);
        if !active && self.left_at.is_some_and(|t| t.elapsed() >= UNREAD_AFTER) {
            self.unread = true;
        }
    }

    pub(super) fn forward(&mut self, active: bool) -> Result<()> {
        let output = self.emulator.take_output();
        if !output.to_child.is_empty() {
            io::write_to_child(&self.writer, &output.to_child);
        }
        if active && !output.to_host.is_empty() {
            let mut stdout = std::io::stdout();
            stdout.write_all(&output.to_host)?;
            stdout.flush()?;
        }
        Ok(())
    }

    pub(super) fn write(&self, bytes: &[u8]) {
        io::write_to_child(&self.writer, bytes);
    }

    pub(super) fn resize(&mut self, size: TerminalSize) -> Result<()> {
        if let Some(pty) = &self.pty {
            pty.resize(size);
        }
        self.emulator.resize(size.rows, size.cols)?;
        Ok(())
    }

    pub(super) fn kill(&mut self) {
        tracing::info!(tab = self.id, "closing the tab");
        if let Err(err) = self.killer.kill() {
            tracing::debug!(error = %err, "the program had already stopped");
        }
    }

    pub(super) fn apply(&mut self, config: &Config) {
        self.settings = config.settings(&self.program);
        self.emulator
            .set_options(self.settings.render_options(&self.program));
    }

    pub(super) fn close(mut self, store: Option<&Store>) -> u32 {
        self.recorder.stop();
        if let (Some(store), Some(meta)) = (store, self.meta.as_mut()) {
            meta.ended = Some(crate::store::now());
            store.save(meta);
        }
        if let Some(pty) = self.pty.take() {
            std::thread::spawn(move || {
                drop(pty);
                tracing::debug!("pty closed");
            });
        }
        let code = self.life.code.unwrap_or(1);
        tracing::info!(tab = self.id, code, "tab closed");
        code
    }
}
