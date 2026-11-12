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
const AFTER_EXIT: Duration = Duration::from_millis(1_000);
const AFTER_CLOSE: Duration = Duration::from_millis(1_500);
const UNREAD_AFTER: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Spawn {
    pub(crate) agent: String,
    pub(crate) argv: Vec<String>,
    pub(crate) cwd: PathBuf,
    pub(crate) meta: Option<SessionMeta>,
    pub(crate) document: bool,
}

#[derive(Default)]
pub(super) struct Lifecycle {
    pub(super) code: Option<u32>,
    pub(super) output_closed: bool,
    pub(super) last_event: Option<Instant>,
    pub(super) exited_at: Option<Instant>,
    pub(super) closing_at: Option<Instant>,
}

impl Lifecycle {
    pub(super) fn exited(&mut self, code: u32, now: Instant) {
        self.code = Some(code);
        self.exited_at = Some(now);
        self.last_event = Some(now);
    }

    pub(super) fn done(&self, now: Instant) -> bool {
        let quiet = self.last_event.is_some_and(|t| now >= t + EXIT_GRACE);
        let exited = self.code.is_some() && (self.output_closed || quiet);
        let lingering = self.exited_at.is_some_and(|t| now >= t + AFTER_EXIT);
        let forced = self.closing_at.is_some_and(|t| now >= t + AFTER_CLOSE);
        exited || lingering || forced
    }

    pub(super) fn deadline(&self) -> Option<Instant> {
        let quiet = self.code.and(self.last_event).map(|t| t + EXIT_GRACE);
        [
            quiet,
            self.exited_at.map(|t| t + AFTER_EXIT),
            self.closing_at.map(|t| t + AFTER_CLOSE),
        ]
        .into_iter()
        .flatten()
        .min()
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
    group: crate::host::ProcessGroup,
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
    if open.spawn.document {
        return None;
    }
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
        let mut emulator = Emulator::with_options(open.size.rows, open.size.cols, options)?;
        let cmd = crate::command::command(&open.spawn.argv, &open.spawn.cwd);
        let (pty, child) = Pty::spawn(cmd, open.size)?;
        let mut killer = child.clone_killer();
        let mut group = crate::host::ProcessGroup::contain(&*child);
        let (reader, writer) = match pty.reader().and_then(|r| Ok((r, pty.writer()?))) {
            Ok(pipes) => pipes,
            Err(err) => {
                let _ = killer.kill();
                group.end();
                return Err(err);
            }
        };
        io::spawn_pty_reader(reader, open.id, open.events.clone());
        io::spawn_child_waiter(child, open.id, open.events.clone());
        let writer = Arc::new(Mutex::new(writer));
        let resuming = open.spawn.meta.is_some();
        let meta = session(&mut open);
        let output = meta
            .as_ref()
            .zip(open.store)
            .map(|(m, store)| store.output(&m.id));
        let continuing = resuming && output.as_ref().is_some_and(|p| p.exists());
        let wanted = !open.spawn.document && (open.record || settings.record || continuing);
        let path = match output {
            Some(output) => wanted.then_some(output),
            None => {
                wanted.then(|| crate::debug_files::next_in(&open.spawn.cwd, &program).recording)
            }
        };
        let recorder = Recorder::new(path, open.id == 0 && !open.spawn.document);
        emulator.allow_clipboard(settings.clipboard);
        emulator.keep_mouse_on(settings.bar && settings.bar_mouse);
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
            group,
            emulator,
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
        self.life.closing_at.get_or_insert_with(Instant::now);
        if self.life.code.is_some() {
            return;
        }
        if let Err(err) = self.killer.kill() {
            tracing::debug!(error = %err, "the program had already stopped");
        }
    }

    pub(super) fn apply(&mut self, config: &Config) {
        self.settings = config.settings(&self.program);
        self.emulator
            .set_options(self.settings.render_options(&self.program));
        self.emulator.allow_clipboard(self.settings.clipboard);
        self.emulator
            .keep_mouse_on(self.settings.bar && self.settings.bar_mouse);
    }

    pub(super) fn close(mut self, store: Option<&Store>) -> u32 {
        self.recorder.stop();
        if let (Some(store), Some(meta)) = (store, self.meta.as_mut()) {
            meta.ended = Some(crate::store::now());
            store.save(meta);
        }
        let asked = self.life.closing_at.is_some();
        if asked || self.life.code.is_none() {
            self.group.end();
        }
        if let Some(pty) = self.pty.take() {
            std::thread::spawn(move || {
                drop(pty);
                tracing::debug!("pty closed");
            });
        }
        let code = if asked {
            0
        } else {
            self.life.code.unwrap_or(1)
        };
        tracing::info!(tab = self.id, code, "tab closed");
        code
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tab_whose_program_exited_closes_even_if_output_keeps_coming() {
        let start = Instant::now();
        let mut life = Lifecycle::default();
        life.exited(0, start);
        let mut now = start;
        for _ in 0..30 {
            now += Duration::from_millis(30);
            life.last_event = Some(now);
        }
        assert!(life.done(start + AFTER_EXIT));
        assert!(!life.done(start + Duration::from_millis(50)));
    }

    #[test]
    fn a_tab_the_user_closed_ends_even_if_the_program_never_exits() {
        let start = Instant::now();
        let life = Lifecycle {
            closing_at: Some(start),
            last_event: Some(start + AFTER_CLOSE),
            ..Lifecycle::default()
        };
        assert!(!life.done(start + Duration::from_millis(500)));
        assert!(life.done(start + AFTER_CLOSE));
        assert_eq!(life.deadline(), Some(start + AFTER_CLOSE));
    }

    #[test]
    fn a_quiet_exit_still_closes_quickly() {
        let start = Instant::now();
        let mut life = Lifecycle::default();
        life.exited(3, start);
        assert!(life.done(start + EXIT_GRACE));
    }
}
