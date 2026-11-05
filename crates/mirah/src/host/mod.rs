#[cfg(windows)]
mod windows;

use anyhow::{Context, Result};
use mirah_term::{Align, RenderOptions, Shaping, sequences};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

const SHAPING_ENV: &str = "MIRAH_SHAPING";
const ALIGN_ENV: &str = "MIRAH_ALIGN";
const WINDOWS_TERMINAL_ENV: &str = "WT_SESSION";
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

fn shaping() -> Shaping {
    match std::env::var(SHAPING_ENV).as_deref() {
        Ok("full") => Shaping::Full,
        Ok("words") => Shaping::Words,
        Ok(other) => {
            tracing::warn!(value = other, "unknown MIRAH_SHAPING, detecting instead");
            detected_shaping()
        }
        Err(_) => detected_shaping(),
    }
}

fn detected_shaping() -> Shaping {
    if std::env::var_os(WINDOWS_TERMINAL_ENV).is_some() {
        Shaping::Words
    } else {
        Shaping::Full
    }
}

fn align() -> Align {
    match std::env::var(ALIGN_ENV).as_deref() {
        Ok("arabic") => Align::Arabic,
        Ok("off") => Align::Off,
        Ok("prose") | Err(_) => Align::Prose,
        Ok(other) => {
            tracing::warn!(value = other, "unknown MIRAH_ALIGN, using prose");
            Align::Prose
        }
    }
}

pub(crate) fn render_options() -> RenderOptions {
    RenderOptions {
        shaping: shaping(),
        align: align(),
    }
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
