use super::event::Event;
use anyhow::Result;
use std::sync::mpsc::Sender;

#[cfg(unix)]
pub(crate) fn spawn_watcher(events: Sender<Event>) -> Result<()> {
    let mut signals = signal_hook::iterator::Signals::new([signal_hook::consts::SIGWINCH])?;
    std::thread::spawn(move || {
        for _ in signals.forever() {
            if events.send(Event::Resize).is_err() {
                break;
            }
        }
    });
    Ok(())
}

#[cfg(windows)]
const POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(200);

#[cfg(windows)]
pub(crate) fn spawn_watcher(events: Sender<Event>) -> Result<()> {
    std::thread::spawn(move || {
        let mut last = crossterm::terminal::size().ok();
        loop {
            std::thread::sleep(POLL_INTERVAL);
            let now = crossterm::terminal::size().ok();
            let usable = now.is_some_and(|(cols, rows)| cols > 0 && rows > 0);
            if !usable || now == last {
                continue;
            }
            last = now;
            if events.send(Event::Resize).is_err() {
                break;
            }
        }
    });
    Ok(())
}
