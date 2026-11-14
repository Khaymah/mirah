mod actions;
mod event;
mod held;
mod io;
pub(crate) mod mouse;
mod pacing;
mod pty;
mod record;
mod resize;
mod tab;

use crate::agents::Agent;
use crate::chrome::Chrome;
use crate::chrome::bar::TabLabel;
use crate::config::Config;
use crate::host::{self, TerminalGuard, TerminalSize};
use crate::store::Store;
use anyhow::Result;
use event::{Event, TabId};
use mirah_term::Emulator;
pub(crate) use mouse::MouseEvent;
use mouse::MouseRemap;
use pacing::Pacer;
use std::io::Write;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::Instant;
pub(crate) use tab::Spawn;
use tab::Tab;

const BLANK_MODES: &[u8] = b"\x1b[?25l\x1b[?1000h\x1b[?1006h";
const STARTUP_POLL: std::time::Duration = std::time::Duration::from_millis(100);
const SHUTDOWN_GRACE: std::time::Duration = std::time::Duration::from_millis(800);

pub(crate) struct Outcome {
    pub(crate) code: u32,
    pub(crate) sessions: Vec<String>,
    pub(crate) notes: Vec<String>,
    pub(crate) restart: bool,
}

pub(crate) struct Start {
    pub(crate) first: Option<Spawn>,
    pub(crate) record: bool,
    pub(crate) reserved: Option<String>,
    pub(crate) store: Option<Store>,
    pub(crate) config: Config,
    pub(crate) agents: Vec<Agent>,
}

struct Workspace {
    inbox: Receiver<Event>,
    events: Sender<Event>,
    tabs: Vec<Tab>,
    active: usize,
    next_id: TabId,
    blank: Emulator,
    pacer: Pacer,
    mouse: MouseRemap,
    chrome: Chrome,
    config: Config,
    store: Option<Store>,
    agents: Vec<Agent>,
    size: TerminalSize,
    record: bool,
    reserved: Option<String>,
    notice_until: Option<Instant>,
    held: held::Held,
    exit: Option<u32>,
    sessions: Vec<String>,
    notes: Vec<String>,
    installing: Option<(String, bool)>,
    restart_at: Option<Instant>,
    spin_at: Option<Instant>,
    restart: bool,
}

fn child_size(size: TerminalSize, top: u16) -> TerminalSize {
    TerminalSize {
        rows: size.rows.saturating_sub(top).max(1),
        cols: size.cols,
    }
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

fn blank(size: TerminalSize) -> Result<Emulator> {
    let mut blank = Emulator::new(size.rows, size.cols)?;
    blank.feed(BLANK_MODES);
    Ok(blank)
}

pub(crate) fn run(start: Start) -> Result<Outcome> {
    let program = start
        .first
        .as_ref()
        .and_then(|s| s.argv.first().cloned())
        .unwrap_or_default();
    let settings = start.config.settings(&program);
    if settings.update_check && !settings.bar {
        crate::update::startup();
    }
    let shaping = settings.render_options(&program).shaping;
    let chrome = Chrome::new(settings, shaping);
    host::prepare_pty();
    let size = host::size();
    let (events, inbox) = mpsc::channel();
    let mut workspace = Workspace {
        blank: blank(child_size(size, chrome.top()))?,
        inbox,
        events,
        tabs: Vec::new(),
        active: 0,
        next_id: 0,
        pacer: Pacer::default(),
        mouse: MouseRemap::default(),
        chrome,
        config: start.config,
        store: start.store,
        agents: start.agents,
        size,
        record: start.record,
        reserved: start.reserved,
        notice_until: None,
        held: held::Held::default(),
        exit: None,
        sessions: Vec::new(),
        notes: Vec::new(),
        installing: None,
        restart_at: None,
        spin_at: None,
        restart: false,
    };
    let offer = (settings.update_check && settings.bar)
        .then(crate::update::startup_offer)
        .flatten();
    let first = start.first;
    let _guard = match offer {
        None => {
            workspace.start(first)?;
            resize::spawn_watcher(workspace.events.clone())?;
            TerminalGuard::enter()?
        }
        Some((current, version)) => {
            resize::spawn_watcher(workspace.events.clone())?;
            let guard = TerminalGuard::enter()?;
            workspace.chrome.offer_update(current, version);
            workspace.startup_update()?;
            if workspace.exit.is_none() {
                workspace.start(first)?;
            }
            guard
        }
    };
    check_updates(&workspace.chrome, &workspace.events);
    io::spawn_stdin_reader(workspace.events.clone());
    tracing::info!(
        rows = size.rows,
        cols = size.cols,
        term = std::env::var("TERM").unwrap_or_default(),
        term_program = std::env::var("TERM_PROGRAM").unwrap_or_default(),
        "workspace started"
    );
    workspace.pump()?;
    let code = workspace.exit.unwrap_or(0);
    workspace.shutdown();
    drop(_guard);
    Ok(Outcome {
        code,
        sessions: workspace.sessions,
        notes: workspace.notes,
        restart: workspace.restart,
    })
}

impl Workspace {
    fn inner(&self) -> TerminalSize {
        child_size(self.size, self.chrome.top())
    }

    fn open(&mut self, spawn: Spawn) -> Result<()> {
        let id = self.next_id;
        self.next_id += 1;
        let tab = Tab::open(tab::Open {
            id,
            spawn,
            size: self.inner(),
            config: &self.config,
            store: self.store.as_ref(),
            record: self.record,
            reserved: self.reserved.take(),
            events: &self.events,
        })?;
        if let Some(meta) = &tab.meta
            && !self.sessions.contains(&meta.id)
        {
            self.sessions.push(meta.id.clone());
        }
        self.tabs.push(tab);
        self.switch(self.tabs.len() - 1);
        Ok(())
    }

    fn switch(&mut self, index: usize) {
        if index >= self.tabs.len() {
            return;
        }
        if let Some(old) = self.tabs.get_mut(self.active)
            && self.active != index
        {
            old.left_at = Some(Instant::now());
        }
        self.active = index;
        let tab = &mut self.tabs[index];
        tab.unread = false;
        tab.left_at = None;
        tab.emulator.redraw();
        self.chrome.settings = tab.settings;
        self.chrome
            .set_shaping(tab.settings.render_options(&tab.program).shaping);
        self.chrome.state.recording = tab
            .recording_since
            .map(|s| std::time::Duration::from_secs(s.elapsed().as_secs()));
        self.sync_tabs();
    }

    fn sync_tabs(&mut self) {
        let mut labels: Vec<TabLabel> = Vec::new();
        for tab in &self.tabs {
            let same = labels
                .iter()
                .filter(|l| l.name.split(' ').next() == Some(tab.name.as_str()))
                .count();
            let name = if same == 0 {
                tab.name.clone()
            } else {
                format!("{} {}", tab.name, same + 1)
            };
            labels.push(TabLabel {
                name,
                unread: tab.unread,
            });
        }
        self.chrome.set_tabs(labels, self.active);
    }

    fn emulator(&mut self) -> &mut Emulator {
        match self.tabs.get_mut(self.active) {
            Some(tab) => &mut tab.emulator,
            None => &mut self.blank,
        }
    }

    fn index_of(&self, id: TabId) -> Option<usize> {
        self.tabs.iter().position(|t| t.id == id)
    }

    fn pump(&mut self) -> Result<()> {
        let mut deadline = Some(Instant::now());
        while let Some(events) = self.receive(deadline) {
            self.apply(events)?;
            if let Some(late) = self.held.expired(Instant::now()) {
                self.input(&late)?;
            }
            for (i, tab) in self.tabs.iter_mut().enumerate() {
                tab.forward(i == self.active)?;
                if let Some(text) = tab.emulator.take_copy() {
                    host::keep_clipboard(text);
                }
            }
            let now = Instant::now();
            if self.tabs.iter().any(|t| t.life.done(now)) {
                self.paint(now, true)?;
            }
            let closed = self.reap(now);
            if self.exit.is_some() {
                break;
            }
            if self.tabs.is_empty() && self.chrome.is_idle() {
                self.launcher();
            }
            let tick = earliest(self.tick(now), self.held.deadline());
            deadline = earliest(self.paint(now, closed)?, tick);
        }
        Ok(())
    }

    fn reap(&mut self, now: Instant) -> bool {
        let mut closed = false;
        let mut index = 0;
        while index < self.tabs.len() {
            if !self.tabs[index].life.done(now) {
                index += 1;
                continue;
            }
            let tab = self.tabs.remove(index);
            let left = tab
                .left_the_terminal()
                .then(|| (tab.program.clone(), tab.meta.as_ref().map(|m| m.id.clone())));
            let code = tab.close(self.store.as_ref());
            closed = true;
            if let Some((program, session)) = left {
                self.forget_session(session);
                self.notes.push(format!(
                    "`{program}` ended right away without showing anything, probably because it opened its own window. mirah runs programs that work inside the terminal, such as claude, codex and opencode."
                ));
                if !self.tabs.is_empty() || !self.chrome.is_idle() {
                    self.notice(format!("انتهى {program} فورًا دون أن يعرض شيئًا"));
                }
            }
            if self.tabs.is_empty() {
                if self.chrome.is_idle() {
                    self.exit.get_or_insert(code);
                } else {
                    self.chrome.keep_open();
                }
                return true;
            }
            if index < self.active || self.active >= self.tabs.len() {
                self.active = self.active.saturating_sub(1);
            }
            let active = self.active;
            self.switch(active);
        }
        closed
    }

    fn paint(&mut self, now: Instant, force: bool) -> Result<Option<Instant>> {
        if !self.emulator().needs_frame() && !self.chrome.dirty {
            return Ok(None);
        }
        let synchronizing = self.emulator().is_synchronizing();
        let ready = self.pacer.ready_at(now, synchronizing);
        if force || now >= ready {
            let overlay = self.chrome.overlay(self.size.cols, self.size.rows);
            let started = Instant::now();
            let frame = self.emulator().frame_with(&overlay);
            let mut stdout = std::io::stdout();
            stdout.write_all(frame.as_bytes())?;
            stdout.flush()?;
            tracing::debug!(
                bytes = frame.len(),
                micros = started.elapsed().as_micros(),
                "frame"
            );
            self.pacer.drew(now, synchronizing);
            Ok(None)
        } else {
            Ok(Some(ready))
        }
    }

    fn apply(&mut self, events: Vec<Event>) -> Result<()> {
        let mut unread = false;
        for event in events {
            match event {
                Event::Output(id, bytes) => {
                    let active = self.tabs.get(self.active).map(|t| t.id);
                    if let Some(i) = self.index_of(id) {
                        let tab = &mut self.tabs[i];
                        let was = tab.unread;
                        tab.life.last_event = Some(Instant::now());
                        tab.feed(&bytes, active == Some(id));
                        unread |= tab.unread != was;
                    }
                }
                Event::Input(bytes) => {
                    let ready = self.held.take_ready(&bytes, Instant::now());
                    if !ready.is_empty() {
                        self.input(&ready)?;
                    }
                }
                event @ (Event::UpdateFound(_) | Event::UpdateStep(_) | Event::UpdateDone(_)) => {
                    self.update_event(event);
                }
                Event::OutputClosed(id) => {
                    if let Some(i) = self.index_of(id) {
                        self.tabs[i].life.output_closed = true;
                    }
                }
                Event::ChildExited(id, code) => {
                    if let Some(i) = self.index_of(id) {
                        self.tabs[i].life.exited(code, Instant::now());
                    }
                }
                Event::Resize => self.resize()?,
                Event::Stop(code) => {
                    self.exit.get_or_insert(code);
                }
            }
        }
        if unread {
            self.sync_tabs();
        }
        Ok(())
    }

    fn resize(&mut self) -> Result<()> {
        self.size = host::size();
        let inner = self.inner();
        for tab in &mut self.tabs {
            tab.resize(inner)?;
        }
        self.blank.resize(inner.rows, inner.cols)?;
        self.chrome.dirty = true;
        tracing::debug!(rows = self.size.rows, cols = self.size.cols, "resized");
        Ok(())
    }

    fn receive(&self, deadline: Option<Instant>) -> Option<Vec<Event>> {
        let exit = self.tabs.iter().filter_map(|t| t.life.deadline()).min();
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
        let mut events = vec![first];
        events.extend(self.inbox.try_iter());
        Some(events)
    }

    fn start(&mut self, first: Option<Spawn>) -> Result<()> {
        match first {
            Some(spawn) => self.open(spawn),
            None => {
                self.launcher();
                Ok(())
            }
        }
    }

    fn startup_update(&mut self) -> Result<()> {
        use crossterm::event::{
            self as term, Event as Term, KeyEventKind, MouseButton, MouseEventKind,
        };
        loop {
            let pending: Vec<Event> = self.inbox.try_iter().collect();
            self.apply(pending)?;
            let now = Instant::now();
            let tick = self.tick(now);
            if self.exit.is_some() || !self.chrome.update_shown() {
                return Ok(());
            }
            self.paint(now, true)?;
            let wait = tick
                .map_or(STARTUP_POLL, |t| {
                    t.saturating_duration_since(Instant::now())
                })
                .min(STARTUP_POLL);
            if !term::poll(wait)? {
                continue;
            }
            let bytes: Vec<u8> = match term::read()? {
                Term::Key(key) if key.kind != KeyEventKind::Release => startup_key(key.code),
                Term::Mouse(m) if m.kind == MouseEventKind::Down(MouseButton::Left) => {
                    format!("\x1b[<0;{};{}M", m.column + 1, m.row + 1).into_bytes()
                }
                Term::Mouse(m) if m.kind == MouseEventKind::Up(MouseButton::Left) => {
                    format!("\x1b[<0;{};{}m", m.column + 1, m.row + 1).into_bytes()
                }
                _ => Vec::new(),
            };
            if !bytes.is_empty() {
                self.input(&bytes)?;
            }
        }
    }

    fn forget_session(&mut self, session: Option<String>) {
        let Some(id) = session else {
            return;
        };
        self.sessions.retain(|s| *s != id);
        if let Some(store) = &self.store {
            store.remove(&id);
        }
    }

    fn shutdown(&mut self) {
        for tab in &mut self.tabs {
            tab.kill();
        }
        let until = Instant::now() + SHUTDOWN_GRACE;
        while self.tabs.iter().any(|t| t.life.code.is_none()) {
            let left = until.saturating_duration_since(Instant::now());
            if left.is_zero() {
                break;
            }
            match self.inbox.recv_timeout(left) {
                Ok(Event::ChildExited(id, code)) => {
                    if let Some(i) = self.index_of(id) {
                        self.tabs[i].life.exited(code, Instant::now());
                    }
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
        for tab in std::mem::take(&mut self.tabs) {
            tab.close(self.store.as_ref());
        }
    }
}

fn startup_key(code: crossterm::event::KeyCode) -> Vec<u8> {
    use crossterm::event::KeyCode;
    match code {
        KeyCode::Left => b"\x1b[D".to_vec(),
        KeyCode::Right => b"\x1b[C".to_vec(),
        KeyCode::Up => b"\x1b[A".to_vec(),
        KeyCode::Down => b"\x1b[B".to_vec(),
        KeyCode::Tab => b"\t".to_vec(),
        KeyCode::Enter => b"\r".to_vec(),
        KeyCode::Esc => b"\x1b".to_vec(),
        KeyCode::Char(c) => c.to_string().into_bytes(),
        _ => Vec::new(),
    }
}

fn earliest(a: Option<Instant>, b: Option<Instant>) -> Option<Instant> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
}
