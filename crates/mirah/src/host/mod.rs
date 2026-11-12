#[cfg(windows)]
mod windows;

use anyhow::{Context, Result};
use mirah_term::{InputOrder, Shaping, sequences};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

const WINDOWS_TERMINAL_ENV: &str = "WT_SESSION";
const SELF_REORDERING_ON_WINDOWS: &[&str] = &["claude"];
const FALLBACK_COLS: u16 = 80;
const FALLBACK_ROWS: u16 = 24;

static ACTIVE: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy)]
pub(crate) struct TerminalSize {
    pub(crate) rows: u16,
    pub(crate) cols: u16,
}

pub(crate) fn size() -> TerminalSize {
    let (cols, rows) = crossterm::terminal::size().unwrap_or((FALLBACK_COLS, FALLBACK_ROWS));
    TerminalSize {
        rows: rows.max(1),
        cols: cols.max(1),
    }
}

pub(crate) fn detected_shaping() -> Shaping {
    if std::env::var_os(WINDOWS_TERMINAL_ENV).is_some() {
        Shaping::Plain
    } else {
        Shaping::Full
    }
}

pub(crate) fn detected_input_order(program: &str, windows: bool) -> InputOrder {
    let stem = crate::config::program_name(program);
    if windows && SELF_REORDERING_ON_WINDOWS.contains(&stem.as_str()) {
        InputOrder::Visual
    } else {
        InputOrder::Logical
    }
}

const CLIPBOARD_RETRIES: [std::time::Duration; 2] = [
    std::time::Duration::from_millis(250),
    std::time::Duration::from_millis(900),
];

#[cfg(windows)]
static CLIPBOARD_GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[cfg(windows)]
pub(crate) fn keep_clipboard(text: String) {
    let generation = CLIPBOARD_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    std::thread::spawn(move || {
        for wait in CLIPBOARD_RETRIES {
            std::thread::sleep(wait);
            if CLIPBOARD_GENERATION.load(Ordering::SeqCst) != generation {
                return;
            }
            let stored = windows::set_clipboard(&text);
            tracing::debug!(
                stored,
                chars = text.chars().count(),
                "clipboard set in reading order"
            );
        }
    });
}

#[cfg(not(windows))]
pub(crate) fn keep_clipboard(text: String) {
    tracing::debug!(
        chars = text.chars().count(),
        "copy restored to reading order"
    );
    let _ = CLIPBOARD_RETRIES;
}

#[cfg(windows)]
pub(crate) struct ProcessGroup(Option<windows::Job>);

#[cfg(windows)]
impl ProcessGroup {
    pub(crate) fn contain(child: &dyn portable_pty::Child) -> Self {
        let job = child.as_raw_handle().and_then(windows::Job::contain);
        if job.is_none() {
            tracing::warn!(
                "could not put the program in a job; closing may leave children running"
            );
        }
        Self(job)
    }

    pub(crate) fn end(&mut self) {
        if let Some(job) = self.0.as_mut() {
            job.end();
        }
    }
}

#[cfg(unix)]
pub(crate) struct ProcessGroup(Option<i32>);

#[cfg(unix)]
impl ProcessGroup {
    pub(crate) fn contain(child: &dyn portable_pty::Child) -> Self {
        Self(child.process_id().and_then(|pid| i32::try_from(pid).ok()))
    }

    #[allow(
        unsafe_code,
        reason = "killpg has no safe wrapper; the group id is the pty session leader we spawned"
    )]
    pub(crate) fn end(&mut self) {
        if let Some(group) = self.0.take() {
            let sent = unsafe { libc::killpg(group, libc::SIGKILL) } == 0;
            tracing::debug!(group, sent, "process group ended");
        }
    }
}

#[cfg(windows)]
pub(crate) fn downloads_folder() -> Option<std::path::PathBuf> {
    windows::downloads_folder()
}

pub(crate) fn prepare_pty() {
    #[cfg(windows)]
    windows::install_conpty();
}

pub(crate) struct TerminalGuard;

impl TerminalGuard {
    pub(crate) fn enter() -> Result<Self> {
        crossterm::terminal::enable_raw_mode().context("enable raw mode")?;
        #[cfg(windows)]
        windows::enable_virtual_terminal();
        ACTIVE.store(true, Ordering::SeqCst);
        let mut out = std::io::stdout();
        out.write_all(sequences::enter().as_bytes())?;
        out.flush()?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore();
    }
}

pub(crate) fn restore() {
    if !ACTIVE.swap(false, Ordering::SeqCst) {
        return;
    }
    let mut out = std::io::stdout();
    let _ = out.write_all(sequences::leave().as_bytes());
    let _ = out.flush();
    #[cfg(windows)]
    windows::restore_console_modes();
    disable_raw_mode();
}

pub(crate) fn disable_raw_mode() {
    let _ = crossterm::terminal::disable_raw_mode();
}

pub(crate) fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore();
        previous(info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_on_windows_sends_visual_text() {
        for program in [
            "claude",
            "claude.exe",
            "C:\\npm\\Claude.cmd",
            "/usr/bin/claude",
        ] {
            assert_eq!(
                detected_input_order(program, true),
                InputOrder::Visual,
                "{program}"
            );
        }
    }

    #[test]
    fn other_programs_and_other_systems_send_logical_text() {
        assert_eq!(detected_input_order("codex", true), InputOrder::Logical);
        assert_eq!(detected_input_order("claude", false), InputOrder::Logical);
    }
}
