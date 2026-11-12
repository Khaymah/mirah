mod markdown;
pub(crate) mod play;

use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEventKind,
};
use crossterm::terminal;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

const ENTER: &str = "\x1b[?1049h\x1b[?25l";
const LEAVE: &str = "\x1b[0m\x1b[?25h\x1b[?1049l";
const STATUS: &str = "\x1b[0;48;2;30;31;35;38;2;140;144;154m";
const NOTICE: &str = "\x1b[0;48;2;30;31;35;38;2;217;145;104m";
const POLL: Duration = Duration::from_millis(500);
const WHEEL_STEP: usize = 3;
const MAX_DOWNLOAD_COPIES: u32 = 100;
const MARKDOWN_EXTENSIONS: [&str; 2] = ["md", "markdown"];
const SKIPPED_DIRS: [&str; 3] = ["target", "node_modules", "dist"];
const SEARCH_DEPTH: usize = 2;
const RECENT_FILES: usize = 5;
pub(crate) const COMMAND: &str = "__view";
pub(crate) const PLAY_COMMAND: &str = "__play";
const RECORDING_EXTENSION: &str = "vt";

pub(crate) fn spawn(path: &Path) -> crate::session::Spawn {
    let path = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    let exe =
        std::env::current_exe().map_or_else(|_| "mirah".to_string(), |e| e.display().to_string());
    crate::session::Spawn {
        agent: path
            .file_name()
            .map_or_else(|| "md".to_string(), |n| n.to_string_lossy().into_owned()),
        argv: vec![
            exe,
            if is_recording(&path) {
                PLAY_COMMAND
            } else {
                COMMAND
            }
            .to_string(),
            path.display().to_string(),
        ],
        cwd: path
            .parent()
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf),
        meta: None,
        document: true,
    }
}

pub(crate) fn is_recording(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case(RECORDING_EXTENSION))
}

pub(crate) fn is_markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| MARKDOWN_EXTENSIONS.contains(&e.to_lowercase().as_str()))
}

fn collect(dir: &Path, depth: usize, found: &mut Vec<(std::time::SystemTime, PathBuf)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if meta.is_dir() {
            if depth > 0 && !name.starts_with('.') && !SKIPPED_DIRS.contains(&name.as_str()) {
                collect(&path, depth - 1, found);
            }
        } else if is_markdown(&path) || is_recording(&path) {
            let modified = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
            found.push((modified, path));
        }
    }
}

pub(crate) fn recent_files(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect(dir, SEARCH_DEPTH, &mut found);
    found.sort_by_key(|f| std::cmp::Reverse(f.0));
    found
        .into_iter()
        .take(RECENT_FILES)
        .map(|(_, p)| p)
        .collect()
}

#[cfg(windows)]
fn downloads_dir() -> Option<PathBuf> {
    crate::host::downloads_folder().or_else(|| {
        std::env::var_os("USERPROFILE").map(|home| PathBuf::from(home).join("Downloads"))
    })
}

#[cfg(not(windows))]
fn downloads_dir() -> Option<PathBuf> {
    std::env::var_os("XDG_DOWNLOAD_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Downloads")))
}

fn free_name(dir: &Path, file: &Path) -> Option<PathBuf> {
    let stem = file.file_stem()?.to_string_lossy().into_owned();
    let ext = file
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    let first = dir.join(format!("{stem}{ext}"));
    if !first.exists() {
        return Some(first);
    }
    (2..MAX_DOWNLOAD_COPIES)
        .map(|n| dir.join(format!("{stem} ({n}){ext}")))
        .find(|p| !p.exists())
}

pub(crate) fn save_copy(file: &Path, dir: &Path) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    let target = free_name(dir, file).ok_or_else(|| std::io::Error::other("no free name"))?;
    std::fs::copy(file, &target)?;
    Ok(target)
}

fn clip_status(status: &str, width: usize) -> String {
    let mut out = String::new();
    let mut shown = 0;
    let mut chars = status.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            out.push(c);
            for d in chars.by_ref() {
                out.push(d);
                if d.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }
        if shown >= width {
            break;
        }
        out.push(c);
        shown += 1;
    }
    out
}

struct Pager {
    path: PathBuf,
    source: String,
    lines: Vec<String>,
    top: usize,
    cols: u16,
    rows: u16,
    notice: Option<String>,
}

impl Pager {
    fn page(&self) -> usize {
        usize::from(self.rows.saturating_sub(1)).max(1)
    }

    fn last_top(&self) -> usize {
        self.lines.len().saturating_sub(self.page())
    }

    fn scroll(&mut self, down: bool, by: usize) {
        self.top = if down {
            (self.top + by).min(self.last_top())
        } else {
            self.top.saturating_sub(by)
        };
    }

    fn layout(&mut self) {
        let (cols, rows) = terminal::size().unwrap_or((80, 24));
        if (cols, rows) != (self.cols, self.rows) || self.lines.is_empty() {
            self.cols = cols;
            self.rows = rows;
            self.lines = markdown::render(&self.source, usize::from(cols).saturating_sub(2));
            self.top = self.top.min(self.last_top());
        }
    }

    fn status(&self) -> String {
        if let Some(notice) = &self.notice {
            return format!("{NOTICE} {notice}");
        }
        let name = self
            .path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let percent = if self.last_top() == 0 {
            100
        } else {
            self.top * 100 / self.last_top()
        };
        format!("{STATUS} {name} · {percent}% · ↑↓ للتمرير · S للحفظ في التنزيلات · Esc للإغلاق")
    }

    fn draw(&self, out: &mut impl Write) -> std::io::Result<()> {
        let mut frame = String::from("\x1b[?2026h");
        for row in 0..self.page() {
            let line = self.lines.get(self.top + row).map_or("", String::as_str);
            frame.push_str(&format!("\x1b[{};1H\x1b[0m\x1b[2K {line}\x1b[0m", row + 1));
        }
        let status = clip_status(&self.status(), usize::from(self.cols).saturating_sub(1));
        frame.push_str(&format!(
            "\x1b[{};1H\x1b[2K{status}\x1b[K\x1b[0m\x1b[?2026l",
            self.rows
        ));
        out.write_all(frame.as_bytes())?;
        out.flush()
    }

    fn save(&mut self) {
        self.notice = Some(
            match downloads_dir().map(|dir| save_copy(&self.path, &dir)) {
                Some(Ok(target)) => format!(
                    "حُفظت نسخة في التنزيلات: {}",
                    target
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default()
                ),
                _ => "تعذّر حفظ النسخة".to_string(),
            },
        );
    }

    fn handle(&mut self, event: Event) -> bool {
        tracing::debug!(?event, "viewer input");
        self.notice = None;
        match event {
            Event::Key(key) if key.kind != KeyEventKind::Release => match key.code {
                KeyCode::Esc => return false,
                KeyCode::Down | KeyCode::Enter => self.scroll(true, 1),
                KeyCode::Up => self.scroll(false, 1),
                KeyCode::PageDown => self.scroll(true, self.page()),
                KeyCode::PageUp => self.scroll(false, self.page()),
                KeyCode::Home => self.top = 0,
                KeyCode::End => self.top = self.last_top(),
                KeyCode::Char(c) => match crate::chrome::keys::latin(c).to_ascii_lowercase() {
                    'q' => return false,
                    's' => self.save(),
                    'j' => self.scroll(true, 1),
                    'k' => self.scroll(false, 1),
                    ' ' => self.scroll(true, self.page()),
                    'g' if c.is_ascii_uppercase() => self.top = self.last_top(),
                    'g' => self.top = 0,
                    _ => {}
                },
                _ => {}
            },
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollDown => self.scroll(true, WHEEL_STEP),
                MouseEventKind::ScrollUp => self.scroll(false, WHEEL_STEP),
                _ => {}
            },
            _ => {}
        }
        true
    }
}

fn session(pager: &mut Pager) -> std::io::Result<()> {
    let mut out = std::io::stdout();
    let mut shown = None;
    loop {
        pager.layout();
        let state = (pager.top, pager.cols, pager.rows, pager.notice.clone());
        if shown.as_ref() != Some(&state) {
            pager.draw(&mut out)?;
            shown = Some(state);
        }
        if !event::poll(POLL)? {
            continue;
        }
        if !pager.handle(event::read()?) {
            return Ok(());
        }
    }
}

pub(crate) fn run(path: &Path) -> u32 {
    let source = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) => format!("# تعذّر فتح الملف\n\n`{}`\n\n{err}", path.display()),
    };
    let mut pager = Pager {
        path: path.to_path_buf(),
        source,
        lines: Vec::new(),
        top: 0,
        cols: 0,
        rows: 0,
        notice: None,
    };
    let mut out = std::io::stdout();
    if terminal::enable_raw_mode().is_err() {
        return 1;
    }
    let _ = out.write_all(ENTER.as_bytes());
    let _ = crossterm::execute!(out, EnableMouseCapture);
    let result = session(&mut pager);
    let _ = crossterm::execute!(out, DisableMouseCapture);
    let _ = out.write_all(LEAVE.as_bytes());
    let _ = out.flush();
    let _ = terminal::disable_raw_mode();
    u32::from(result.is_err())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("mirah-viewer-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn saving_twice_keeps_both_copies() {
        let dir = temp("save");
        let file = dir.join("خطة.md");
        std::fs::write(&file, "# x").unwrap_or_default();
        let downloads = dir.join("Downloads");
        let first = save_copy(&file, &downloads).ok();
        let second = save_copy(&file, &downloads).ok();
        assert_eq!(first, Some(downloads.join("خطة.md")));
        assert_eq!(second, Some(downloads.join("خطة (2).md")));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn finds_recent_markdown_files_but_skips_build_folders() {
        let dir = temp("find");
        for path in [
            "README.md",
            "Plan/x.markdown",
            "claude-session-1.vt",
            "target/skip.md",
            ".git/skip.md",
            "a.txt",
        ] {
            let full = dir.join(path);
            let _ = std::fs::create_dir_all(full.parent().unwrap_or(&dir));
            std::fs::write(&full, "x").unwrap_or_default();
        }
        let mut names: Vec<String> = recent_files(&dir)
            .iter()
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .collect();
        names.sort();
        assert_eq!(names, ["README.md", "claude-session-1.vt", "x.markdown"]);
        let _ = std::fs::remove_dir_all(dir);
    }
}
