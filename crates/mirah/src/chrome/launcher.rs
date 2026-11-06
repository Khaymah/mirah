use super::keys::Key;
use super::style::{
    ACCENT, BODY_TOP, BOX_BG, DIM, Frame, Line, TEXT, blank, boxed, right_to_left, shaped, style,
};
use crate::agents::Agent;
use crate::store::SessionMeta;
use mirah_bidi::{Cell, Shaping};

const GAP: usize = 2;
const MAX_PATH: usize = 28;
const MINUTE: u64 = 60;
const HOUR: u64 = 3_600;
const DAY: u64 = 86_400;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Entry {
    Start(Agent),
    Resume(Box<SessionMeta>, Agent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Pick {
    Open(Entry),
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Launcher {
    entries: Vec<Entry>,
    row: usize,
    pub(crate) can_cancel: bool,
    now: u64,
}

fn count(n: u64, one: &str, two: &str, few: &str) -> String {
    match n {
        1 => one.to_string(),
        2 => two.to_string(),
        3..=10 => format!("{n} {few}"),
        _ => format!("{n} {one}"),
    }
}

fn ago(now: u64, then: u64) -> String {
    let secs = now.saturating_sub(then);
    let span = match secs {
        s if s < MINUTE => return "الآن".to_string(),
        s if s < HOUR => count(s / MINUTE, "دقيقة", "دقيقتين", "دقائق"),
        s if s < DAY => count(s / HOUR, "ساعة", "ساعتين", "ساعات"),
        s => count(s / DAY, "يوم", "يومين", "أيام"),
    };
    format!("منذ {span}")
}

fn short_path(path: &std::path::Path) -> String {
    let text = path.display().to_string();
    let count = text.chars().count();
    if count <= MAX_PATH {
        return text;
    }
    let tail: String = text.chars().skip(count - (MAX_PATH - 1)).collect();
    format!("…{tail}")
}

fn line(parts: &[Vec<Cell>]) -> Vec<Cell> {
    let width = parts.iter().map(Vec::len).sum::<usize>() + GAP * parts.len().saturating_sub(1);
    let mut cells = blank(width, style(TEXT, BOX_BG, false));
    right_to_left(&mut cells, width, parts, GAP);
    cells
}

impl Launcher {
    pub(crate) fn new(
        entries: Vec<Entry>,
        preselect: Option<&str>,
        can_cancel: bool,
        now: u64,
    ) -> Self {
        let row = entries
            .iter()
            .position(|e| matches!(e, Entry::Start(a) if Some(a.name.as_str()) == preselect))
            .unwrap_or(0);
        Self {
            entries,
            row,
            can_cancel,
            now,
        }
    }

    fn agents(&self) -> usize {
        self.entries
            .iter()
            .filter(|e| matches!(e, Entry::Start(_)))
            .count()
    }

    fn chosen(&self) -> Option<Pick> {
        self.entries.get(self.row).cloned().map(Pick::Open)
    }

    pub(crate) fn key(&mut self, key: Key) -> Option<Pick> {
        let len = self.entries.len().max(1);
        match key {
            Key::Up => self.row = (self.row + len - 1) % len,
            Key::Down | Key::Tab => self.row = (self.row + 1) % len,
            Key::Enter => return self.chosen(),
            Key::Escape | Key::Toggle => return Some(Pick::Cancel),
            Key::Char(c) => {
                let n = c.to_digit(10).and_then(|d| usize::try_from(d).ok())?;
                if n >= 1 && n <= self.entries.len() {
                    self.row = n - 1;
                    return self.chosen();
                }
            }
            _ => {}
        }
        None
    }

    fn body_row(&self, entry: usize) -> usize {
        let header = usize::from(entry >= self.agents() && self.agents() < self.entries.len());
        entry + header
    }

    pub(crate) fn click(&mut self, row: usize) -> Option<Pick> {
        let body = row.checked_sub(BODY_TOP)?;
        let entry = (0..self.entries.len()).find(|&e| self.body_row(e) == body)?;
        self.row = entry;
        self.chosen()
    }

    fn entry_line(&self, index: usize, entry: &Entry, shaping: Shaping) -> Vec<Cell> {
        let focused = index == self.row;
        let number = shaped(
            &format!("{}", index + 1),
            style(DIM, BOX_BG, false),
            shaping,
        );
        match entry {
            Entry::Start(agent) => {
                let radio = if focused { "◉" } else { "○" };
                let mark = shaped(
                    radio,
                    style(if focused { ACCENT } else { DIM }, BOX_BG, false),
                    shaping,
                );
                let title = shaped(&agent.title, style(TEXT, BOX_BG, focused), shaping);
                let name = shaped(&agent.name, style(DIM, BOX_BG, false), shaping);
                line(&[mark, name, title, number])
            }
            Entry::Resume(meta, _) => {
                let mark = shaped(
                    "↺",
                    style(if focused { ACCENT } else { DIM }, BOX_BG, false),
                    shaping,
                );
                let name = shaped(&meta.agent, style(TEXT, BOX_BG, focused), shaping);
                let when = shaped(
                    &ago(self.now, meta.ended.unwrap_or(meta.started)),
                    style(DIM, BOX_BG, false),
                    shaping,
                );
                let path = shaped(&short_path(&meta.cwd), style(DIM, BOX_BG, false), shaping);
                line(&[mark, name, when, path, number])
            }
        }
    }

    pub(crate) fn draw(&self, shaping: Shaping) -> Frame {
        let mut lines = Vec::new();
        for (i, entry) in self.entries.iter().enumerate() {
            if i == self.agents() && i > 0 {
                lines.push(Line::new(
                    shaped("استئناف جلسة سابقة", style(DIM, BOX_BG, true), shaping),
                    false,
                ));
            }
            lines.push(Line::new(self.entry_line(i, entry, shaping), i == self.row));
        }
        let mut brand = shaped("مرآة ", style(DIM, BOX_BG, false), shaping);
        brand.extend(shaped("◆", style(ACCENT, BOX_BG, true), shaping));
        let question = shaped("اختر برنامجًا", style(TEXT, BOX_BG, true), shaping);
        let title = line(&[brand, question]);
        let hint = if self.can_cancel {
            "↑↓ للتنقل · Enter للفتح · Esc للرجوع"
        } else {
            "↑↓ للتنقل · Enter للفتح · Esc للخروج"
        };
        let footer = shaped(hint, style(DIM, BOX_BG, false), shaping);
        boxed(&title, lines, &footer, shaping)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn agent(name: &str, title: &str) -> Agent {
        Agent {
            name: name.into(),
            title: title.into(),
            command: name.into(),
            resume: vec!["--continue".into()],
        }
    }

    fn meta() -> SessionMeta {
        SessionMeta {
            id: "1106-0457-k3f2".into(),
            agent: "claude".into(),
            argv: vec!["claude".into()],
            cwd: PathBuf::from("/work/app"),
            started: 1_000,
            ended: Some(1_000),
        }
    }

    fn launcher() -> Launcher {
        let claude = agent("claude", "Claude Code");
        Launcher::new(
            vec![
                Entry::Start(claude.clone()),
                Entry::Start(agent("opencode", "OpenCode")),
                Entry::Resume(Box::new(meta()), claude),
            ],
            Some("opencode"),
            false,
            1_000 + 7_200,
        )
    }

    fn text(cells: &[Cell]) -> String {
        cells.iter().map(|c| c.text.as_str()).collect()
    }

    #[test]
    fn ages_agree_with_their_number() {
        assert_eq!(ago(100, 90), "الآن");
        assert_eq!(ago(60, 0), "منذ دقيقة");
        assert_eq!(ago(120, 0), "منذ دقيقتين");
        assert_eq!(ago(300, 0), "منذ 5 دقائق");
        assert_eq!(ago(11 * 60, 0), "منذ 11 دقيقة");
        assert_eq!(ago(3 * DAY, 0), "منذ 3 أيام");
    }

    #[test]
    fn the_last_agent_is_chosen_first() {
        let mut l = launcher();
        assert_eq!(
            l.key(Key::Enter),
            Some(Pick::Open(Entry::Start(agent("opencode", "OpenCode"))))
        );
    }

    #[test]
    fn arrows_wrap_and_escape_cancels() {
        let mut l = launcher();
        l.key(Key::Down);
        l.key(Key::Down);
        assert_eq!(l.row, 0);
        assert_eq!(l.key(Key::Escape), Some(Pick::Cancel));
    }

    #[test]
    fn a_number_opens_that_entry() {
        let mut l = launcher();
        assert!(matches!(
            l.key(Key::Char('3')),
            Some(Pick::Open(Entry::Resume(..)))
        ));
        assert_eq!(l.key(Key::Char('9')), None);
    }

    #[test]
    fn sessions_have_a_header_and_clicks_skip_it() {
        let mut l = launcher();
        let frame = l.draw(Shaping::Plain);
        let rows: Vec<String> = frame.rows.iter().map(|r| text(r)).collect();
        assert!(rows[BODY_TOP].contains("Claude Code"), "{rows:?}");
        assert!(
            rows[1].trim_end_matches([' ', '│']).ends_with("ةآرم ◆"),
            "{rows:?}"
        );
        assert!(rows[BODY_TOP + 2].contains("ةقباس ةسلج"), "{rows:?}");
        assert!(rows[BODY_TOP + 3].contains("/work/app"), "{rows:?}");
        assert!(rows[BODY_TOP + 3].contains("نيتعاس ذنم"), "{rows:?}");
        assert_eq!(l.click(BODY_TOP + 2), None);
        assert!(matches!(
            l.click(BODY_TOP + 3),
            Some(Pick::Open(Entry::Resume(..)))
        ));
        assert!(frame.rows.iter().all(|r| r.len() == frame.width));
    }
}
