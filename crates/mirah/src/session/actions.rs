use super::event::Event;
use super::mouse::MouseEvent;
use super::{Spawn, Workspace};
use crate::agents;
use crate::chrome::Action;
use crate::chrome::launcher::{Entry, Launcher};
use anyhow::Result;
use std::time::{Duration, Instant};

const NOTICE_FOR: Duration = Duration::from_secs(6);
const CLOCK_TICK: Duration = Duration::from_secs(1);
const RECENT_SESSIONS: usize = 5;

fn spawn_for(entry: Entry) -> Spawn {
    match entry {
        Entry::Start(agent) => Spawn {
            argv: vec![agent.command.clone()],
            agent: agent.name,
            cwd: std::env::current_dir().unwrap_or_default(),
            meta: None,
            document: false,
        },
        Entry::Resume(meta, agent) => Spawn {
            argv: agents::resume_argv(&agent, &meta.argv),
            agent: meta.agent.clone(),
            cwd: meta.cwd.clone(),
            meta: Some(*meta),
            document: false,
        },
        Entry::Document(path) => crate::viewer::spawn(&path),
    }
}

impl Workspace {
    pub(super) fn input(&mut self, bytes: &[u8]) -> Result<()> {
        let mut actions = Vec::new();
        let chrome = &mut self.chrome;
        let emulator = self.tabs.get(self.active).map(|t| &t.emulator);
        let bytes = self.mouse.apply(bytes, |e| {
            let e = chrome.mouse(e, &mut actions)?;
            Some(match emulator {
                Some(emulator) if chrome.settings.mouse => MouseEvent {
                    column: emulator.child_column(e.row, e.column),
                    ..e
                },
                _ => e,
            })
        });
        let bytes = self.chrome.input(&bytes, &mut actions);
        if !bytes.is_empty()
            && let Some(tab) = self.tabs.get(self.active)
        {
            tab.write(&bytes);
        }
        for action in actions {
            self.act(action)?;
        }
        Ok(())
    }

    pub(super) fn launcher(&mut self) {
        let mut entries: Vec<Entry> = agents::installed_agents(&self.agents)
            .into_iter()
            .map(Entry::Start)
            .collect();
        if let Some(store) = &self.store {
            let open: Vec<&str> = self
                .tabs
                .iter()
                .filter_map(|t| t.meta.as_ref().map(|m| m.id.as_str()))
                .collect();
            let recent = store
                .list()
                .into_iter()
                .filter(|m| !open.contains(&m.id.as_str()))
                .filter_map(|m| {
                    let agent = agents::find(&m.agent, &self.agents)?;
                    (!agent.resume.is_empty()).then(|| Entry::Resume(Box::new(m), agent))
                })
                .take(RECENT_SESSIONS);
            entries.extend(recent);
        }
        let here = std::env::current_dir().unwrap_or_default();
        entries.extend(
            crate::viewer::recent_files(&here)
                .into_iter()
                .map(Entry::Document),
        );
        let last = agents::last();
        let launcher = Launcher::new(
            entries,
            last.as_deref(),
            !self.tabs.is_empty(),
            crate::store::now(),
        );
        self.chrome.launch(launcher);
    }

    fn act(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Set(scope, key, value) => {
                self.config.set(&scope, key, value);
                if let Err(err) = self.config.save() {
                    tracing::warn!(error = %err, "could not save config.toml");
                }
                for tab in &mut self.tabs {
                    tab.apply(&self.config);
                }
                match self.tabs.get(self.active) {
                    Some(tab) => {
                        self.chrome.settings = tab.settings;
                        self.chrome
                            .set_shaping(tab.settings.render_options(&tab.program).shaping);
                    }
                    None => self.chrome.settings = self.config.settings(""),
                }
                tracing::info!(?key, value, "settings changed from the bar");
            }
            Action::Switch(index) => self.switch(index),
            Action::NewTab => self.launcher(),
            Action::Open(entry) => {
                let spawn = spawn_for(entry);
                let name = spawn.agent.clone();
                if !spawn.document {
                    agents::remember(&name);
                }
                if let Err(err) = self.open(spawn) {
                    tracing::warn!(error = format!("{err:#}"), "could not start the program");
                    self.notice(format!("تعذّر تشغيل {name}"));
                }
            }
            Action::Quit => self.exit = Some(0),
            Action::Record => self.toggle_recording(),
            Action::Update => self.start_update(),
            Action::Close => {
                if let Some(tab) = self.tabs.get_mut(self.active) {
                    tab.kill();
                }
            }
        }
        Ok(())
    }

    fn recording_path(&self) -> Option<std::path::PathBuf> {
        let tab = self.tabs.get(self.active)?;
        match (&self.store, &tab.meta) {
            (Some(store), Some(meta)) => Some(store.output(&meta.id)),
            _ => {
                let dir = std::env::current_dir().unwrap_or_default();
                Some(crate::debug_files::next_in(&dir, &tab.program).recording)
            }
        }
    }

    fn toggle_recording(&mut self) {
        let path = self.recording_path();
        let Some(tab) = self.tabs.get_mut(self.active) else {
            return;
        };
        if tab.recorder.active() {
            tab.recorder.stop();
            if let Some(since) = tab.recording_since.take() {
                tab.recorded = since.elapsed();
            }
            self.chrome.state.recording = None;
            let id = tab.meta.as_ref().map(|m| m.id.clone());
            match id {
                Some(id) => self.notice(format!("حُفظ التسجيل في الجلسة {id}")),
                None => self.notice("حُفظ التسجيل".to_string()),
            }
        } else {
            let started = match tab.recorder.resume() {
                Some(resumed) => resumed,
                None => path.is_some_and(|p| tab.recorder.start(p)),
            };
            if started {
                tab.recording_since = Some(Instant::now() - tab.recorded);
                self.chrome.state.recording = Some(tab.recorded);
            } else {
                self.notice("تعذّر بدء التسجيل".to_string());
            }
        }
        self.chrome.dirty = true;
    }

    fn start_update(&mut self) {
        let Some(version) = self.chrome.state.update.take() else {
            return;
        };
        self.notice(format!("جارٍ تثبيت {version}…"));
        let events = self.events.clone();
        std::thread::spawn(move || {
            let result = crate::update::install_version(&version).map(|()| version);
            let _ = events.send(Event::UpdateDone(result));
        });
    }

    pub(super) fn notice(&mut self, text: String) {
        self.chrome.state.notice = Some(text);
        self.notice_until = Some(Instant::now() + NOTICE_FOR);
        self.chrome.dirty = true;
    }

    pub(super) fn update_event(&mut self, event: Event) {
        match event {
            Event::UpdateFound(version) => {
                self.chrome.state.update = Some(version);
                self.chrome.dirty = true;
            }
            Event::UpdateDone(Ok(version)) => {
                self.notice(format!("ثُبّت {version}، ويعمل في المرة القادمة"));
            }
            Event::UpdateDone(Err(_)) => self.notice("تعذّر التحديث، راجع ملف السجل".to_string()),
            _ => {}
        }
    }

    pub(super) fn tick(&mut self, now: Instant) -> Option<Instant> {
        if self.notice_until.is_some_and(|until| now >= until) {
            self.notice_until = None;
            self.chrome.state.notice = None;
            self.chrome.dirty = true;
        }
        let since = self.tabs.get(self.active).and_then(|t| t.recording_since);
        if let Some(since) = since {
            let elapsed = Duration::from_secs(now.duration_since(since).as_secs());
            if self.chrome.state.recording != Some(elapsed) {
                self.chrome.state.recording = Some(elapsed);
                self.chrome.dirty = true;
            }
        }
        let clock = since.map(|_| now + CLOCK_TICK);
        match (clock, self.notice_until) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        }
    }
}
