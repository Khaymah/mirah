use super::event::Event;
use super::mouse::MouseEvent;
use super::{Session, io};
use crate::chrome::Action;
use anyhow::Result;
use std::time::{Duration, Instant};

const NOTICE_FOR: Duration = Duration::from_secs(6);
const CLOCK_TICK: Duration = Duration::from_secs(1);

impl Session<'_> {
    pub(super) fn input(&mut self, bytes: &[u8]) -> Result<()> {
        let mut actions = Vec::new();
        let chrome = &mut self.chrome;
        let emulator = &self.emulator;
        let bytes = self.mouse.apply(bytes, |e| {
            let e = chrome.mouse(e, &mut actions)?;
            Some(if chrome.settings.mouse {
                MouseEvent {
                    column: emulator.child_column(e.row, e.column),
                    ..e
                }
            } else {
                e
            })
        });
        let bytes = self.chrome.input(&bytes, &mut actions);
        if !bytes.is_empty() {
            io::write_to_child(&self.writer, &bytes);
        }
        for action in actions {
            self.act(action)?;
        }
        Ok(())
    }

    fn act(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Set(scope, key, value) => {
                self.config.set(&scope, key, value);
                if let Err(err) = self.config.save() {
                    tracing::warn!(error = %err, "could not save config.toml");
                }
                let settings = self.config.settings(&self.program);
                let options = settings.render_options(&self.program);
                self.chrome.settings = settings;
                self.chrome.set_shaping(options.shaping);
                self.emulator.set_options(options);
                tracing::info!(?options, "settings changed from the bar");
            }
            Action::Record => self.toggle_recording(),
            Action::Update => self.start_update(),
            Action::Close => {
                tracing::info!("closing the session from the bar");
                if let Err(err) = self.killer.kill() {
                    tracing::warn!(error = %err, "could not stop the program");
                }
            }
        }
        Ok(())
    }

    fn toggle_recording(&mut self) {
        if self.recorder.active() {
            let saved = self.recorder.stop();
            self.recording_since = None;
            self.chrome.state.recording = None;
            if let Some(name) = saved.as_ref().and_then(|p| p.file_name()) {
                self.notice(format!("اتحفظ {}", name.to_string_lossy()));
            }
        } else {
            let dir = std::env::current_dir().unwrap_or_default();
            let path = crate::debug_files::next_in(&dir).recording;
            if self.recorder.start(path) {
                self.recording_since = Some(Instant::now());
                self.chrome.state.recording = Some(Duration::ZERO);
            } else {
                self.notice("التسجيل مااشتغلش".to_string());
            }
        }
        self.chrome.dirty = true;
    }

    fn start_update(&mut self) {
        let Some(version) = self.chrome.state.update.take() else {
            return;
        };
        self.notice(format!("بيتسطّب {version}…"));
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
                self.notice(format!("اتسطّبت {version} وهتشتغل المرة الجاية"));
            }
            Event::UpdateDone(Err(_)) => self.notice("التحديث فشل، شوف الـ log".to_string()),
            _ => {}
        }
    }

    pub(super) fn tick(&mut self, now: Instant) -> Option<Instant> {
        if self.notice_until.is_some_and(|until| now >= until) {
            self.notice_until = None;
            self.chrome.state.notice = None;
            self.chrome.dirty = true;
        }
        if let Some(since) = self.recording_since {
            let elapsed = Duration::from_secs(now.duration_since(since).as_secs());
            if self.chrome.state.recording != Some(elapsed) {
                self.chrome.state.recording = Some(elapsed);
                self.chrome.dirty = true;
            }
        }
        let clock = self.recording_since.map(|_| now + CLOCK_TICK);
        match (clock, self.notice_until) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        }
    }
}
