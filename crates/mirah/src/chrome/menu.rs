use super::keys::Key;
use super::style::{
    ACCENT, BOX_BG, DIM, Frame, Line, TEXT, blank, boxed, content_start, right_to_left, shaped,
    style,
};
use crate::config::{SPECS, Scope, Settings, Spec};
use mirah_bidi::{Cell, Shaping};
use std::ops::Range;

const HINT: &str = "↑↓ للتنقل · ←→ للتغيير · Esc للإغلاق";
const SCOPE_LABEL: &str = "التطبيق على";
const ADVANCED_LABEL: &str = "خيارات متقدّمة";
const GAP: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Change {
    Set(Scope, crate::config::Key, &'static str),
    Close,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Menu {
    pub(crate) row: usize,
    pub(crate) scope: Scope,
    pub(crate) advanced: bool,
    program: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Row {
    Setting(usize),
    Advanced,
    Scope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Pick {
    Value(usize, &'static str),
    Scope(bool),
    Advanced(bool),
}

pub(crate) struct Drawn {
    pub(crate) frame: Frame,
    picks: Vec<(usize, Range<usize>, Pick)>,
}

fn step(spec: &Spec, settings: &Settings, forward: bool) -> &'static str {
    let current = settings.value(spec.key);
    let len = spec.values.len();
    let index = spec
        .values
        .iter()
        .position(|(v, _)| *v == current)
        .unwrap_or(0);
    let next = if forward {
        (index + 1) % len
    } else {
        (index + len - 1) % len
    };
    spec.values[next].0
}

fn option(text: &str, chosen: bool, shaping: Shaping) -> Vec<Cell> {
    if chosen {
        shaped(&format!("[{text}]"), style(ACCENT, BOX_BG, true), shaping)
    } else {
        shaped(&format!(" {text} "), style(DIM, BOX_BG, false), shaping)
    }
}

fn row(
    label: &str,
    label_width: usize,
    options: Vec<Vec<Cell>>,
    shaping: Shaping,
) -> (Vec<Cell>, Vec<Range<usize>>) {
    let mut name = shaped(label, style(TEXT, BOX_BG, false), shaping);
    let pad = label_width.saturating_sub(name.len());
    let mut padded = blank(pad, style(TEXT, BOX_BG, false));
    padded.append(&mut name);
    let mut parts = vec![padded];
    parts.extend(options);
    let width = parts.iter().map(Vec::len).sum::<usize>() + GAP * (parts.len() - 1);
    let mut cells = blank(width, style(TEXT, BOX_BG, false));
    right_to_left(&mut cells, width, &parts, GAP);
    let mut ranges = Vec::new();
    let mut edge = width;
    for (i, part) in parts.iter().enumerate() {
        let start = edge.saturating_sub(part.len());
        if i > 0 {
            ranges.push(start..edge);
        }
        edge = start.saturating_sub(GAP);
    }
    (cells, ranges)
}

impl Menu {
    pub(crate) fn new(program: &str) -> Self {
        Self {
            row: 0,
            scope: Scope::Global,
            advanced: false,
            program: program.to_string(),
        }
    }

    fn rows(&self) -> Vec<Row> {
        let pick = |advanced: bool| {
            SPECS
                .iter()
                .enumerate()
                .filter(move |(_, s)| s.advanced == advanced)
                .map(|(i, _)| Row::Setting(i))
        };
        let mut rows: Vec<Row> = pick(false).collect();
        rows.push(Row::Advanced);
        if self.advanced {
            rows.extend(pick(true));
        }
        if !self.program.is_empty() {
            rows.push(Row::Scope);
        }
        rows
    }

    fn program_scope(&self) -> Scope {
        Scope::Program(self.program.clone())
    }

    fn change(&mut self, settings: &Settings, forward: bool) -> Option<Change> {
        match self.rows().get(self.row).copied()? {
            Row::Setting(i) => {
                let spec = &SPECS[i];
                Some(Change::Set(
                    self.scope.clone(),
                    spec.key,
                    step(spec, settings, forward),
                ))
            }
            Row::Advanced => {
                self.advanced = !self.advanced;
                None
            }
            Row::Scope => {
                self.scope = match self.scope {
                    Scope::Global => self.program_scope(),
                    Scope::Program(_) => Scope::Global,
                };
                None
            }
        }
    }

    pub(crate) fn key(&mut self, key: Key, settings: &Settings) -> Option<Change> {
        let count = self.rows().len();
        match key {
            Key::Up => self.row = (self.row + count - 1) % count,
            Key::Down | Key::Tab => self.row = (self.row + 1) % count,
            Key::Left | Key::Enter => return self.change(settings, true),
            Key::Right => return self.change(settings, false),
            Key::Escape | Key::Toggle => return Some(Change::Close),
            _ => {}
        }
        None
    }

    pub(crate) fn click(
        &mut self,
        drawn: &Drawn,
        row: usize,
        column: usize,
        settings: &Settings,
    ) -> Option<Change> {
        let body = row.checked_sub(super::style::BODY_TOP)?;
        if body >= self.rows().len() {
            return None;
        }
        self.row = body;
        let pick = drawn
            .picks
            .iter()
            .find(|(r, range, _)| *r == body && range.contains(&column))
            .map(|(_, _, p)| p.clone());
        match pick {
            Some(Pick::Value(i, value)) => SPECS
                .get(i)
                .map(|spec| Change::Set(self.scope.clone(), spec.key, value)),
            Some(Pick::Scope(global)) => {
                self.scope = if global {
                    Scope::Global
                } else {
                    self.program_scope()
                };
                None
            }
            Some(Pick::Advanced(show)) => {
                self.advanced = show;
                None
            }
            None => self.change(settings, true),
        }
    }

    fn line(
        &self,
        row: Row,
        settings: &Settings,
        shaping: Shaping,
    ) -> (&str, Vec<Vec<Cell>>, Vec<Pick>) {
        match row {
            Row::Setting(i) => {
                let spec = &SPECS[i];
                let current = settings.value(spec.key);
                let options = spec
                    .values
                    .iter()
                    .map(|(v, label)| option(label, *v == current, shaping))
                    .collect();
                let picks = spec.values.iter().map(|(v, _)| Pick::Value(i, v)).collect();
                (spec.label, options, picks)
            }
            Row::Advanced => (
                ADVANCED_LABEL,
                vec![
                    option("إخفاء", !self.advanced, shaping),
                    option("إظهار", self.advanced, shaping),
                ],
                vec![Pick::Advanced(false), Pick::Advanced(true)],
            ),
            Row::Scope => {
                let global = self.scope == Scope::Global;
                (
                    SCOPE_LABEL,
                    vec![
                        option("كل البرامج", global, shaping),
                        option(&format!("{} فقط", self.program), !global, shaping),
                    ],
                    vec![Pick::Scope(true), Pick::Scope(false)],
                )
            }
        }
    }

    pub(crate) fn draw(&self, settings: &Settings, shaping: Shaping) -> Drawn {
        let label_width = SPECS
            .iter()
            .map(|s| s.label)
            .chain([SCOPE_LABEL, ADVANCED_LABEL])
            .map(|l| shaped(l, style(TEXT, BOX_BG, false), shaping).len())
            .max()
            .unwrap_or(0);
        let mut lines = Vec::new();
        let mut ranges = Vec::new();
        for (index, row_kind) in self.rows().into_iter().enumerate() {
            let (label, options, picks) = self.line(row_kind, settings, shaping);
            let (cells, r) = row(label, label_width, options, shaping);
            ranges.push((r, picks));
            lines.push(Line::new(cells, index == self.row));
        }
        let lens: Vec<usize> = lines.iter().map(|l| l.cells.len()).collect();
        let title = shaped(
            &format!("الإعدادات · {}", self.program),
            style(TEXT, BOX_BG, true),
            shaping,
        );
        let hint = shaped(HINT, style(DIM, BOX_BG, false), shaping);
        let frame = boxed(&title, lines, &hint, shaping);
        let mut picks = Vec::new();
        for (body, ((r, p), len)) in ranges.into_iter().zip(lens).enumerate() {
            let at = content_start(frame.width, len);
            for (range, pick) in r.into_iter().zip(p) {
                picks.push((body, at + range.start..at + range.end, pick));
            }
        }
        Drawn { frame, picks }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Key as Setting;

    fn text(cells: &[Cell]) -> String {
        cells.iter().map(|c| c.text.as_str()).collect()
    }

    fn basic() -> usize {
        SPECS.iter().filter(|s| !s.advanced).count()
    }

    #[test]
    fn arrows_move_and_wrap_over_basic_rows() {
        let mut menu = Menu::new("claude");
        let settings = Settings::default();
        assert_eq!(menu.key(Key::Up, &settings), None);
        assert_eq!(menu.row, basic() + 1);
        assert_eq!(menu.key(Key::Down, &settings), None);
        assert_eq!(menu.row, 0);
    }

    #[test]
    fn advanced_settings_stay_hidden_until_shown() {
        let mut menu = Menu::new("claude");
        let settings = Settings::default();
        let rows = |m: &Menu| m.draw(&settings, Shaping::Plain).frame.rows.len();
        let hidden = rows(&menu);
        menu.row = basic();
        assert_eq!(menu.key(Key::Enter, &settings), None);
        assert!(menu.advanced);
        assert_eq!(rows(&menu), hidden + SPECS.len() - basic());
        menu.row = basic() + 1;
        assert!(matches!(
            menu.key(Key::Enter, &settings),
            Some(Change::Set(Scope::Global, Setting::Shaping, _))
        ));
    }

    #[test]
    fn enter_cycles_the_value_for_the_chosen_scope() {
        let mut menu = Menu::new("claude");
        let settings = Settings::default();
        assert_eq!(
            menu.key(Key::Enter, &settings),
            Some(Change::Set(Scope::Global, Setting::Align, "arabic"))
        );
        menu.row = basic() + 1;
        assert_eq!(menu.key(Key::Enter, &settings), None);
        menu.row = 0;
        assert_eq!(
            menu.key(Key::Right, &settings),
            Some(Change::Set(
                Scope::Program("claude".into()),
                Setting::Align,
                "off"
            ))
        );
    }

    #[test]
    fn escape_closes() {
        let mut menu = Menu::new("claude");
        assert_eq!(
            menu.key(Key::Escape, &Settings::default()),
            Some(Change::Close)
        );
    }

    #[test]
    fn values_sit_inline_and_the_chosen_one_is_bracketed() {
        let menu = Menu::new("claude");
        let drawn = menu.draw(&Settings::default(), Shaping::Plain);
        let rows = &drawn.frame.rows;
        assert_eq!(rows.len(), basic() + 2 + 6);
        assert!(rows.iter().all(|r| r.len() == drawn.frame.width));
        let first = text(&rows[super::super::style::BODY_TOP]);
        assert!(first.starts_with("│▌"), "{first}");
        assert!(first.contains("[صنلا لك]"), "{first}");
        assert!(first.contains("طقف يبرعلا"), "{first}");
    }

    #[test]
    fn without_a_program_there_is_no_scope_to_switch_to() {
        let menu = Menu::new("");
        assert!(!menu.rows().contains(&Row::Scope));
        assert!(Menu::new("claude").rows().contains(&Row::Scope));
    }

    #[test]
    fn clicking_a_value_sets_exactly_that_value() {
        let mut menu = Menu::new("claude");
        let settings = Settings::default();
        let drawn = menu.draw(&settings, Shaping::Plain);
        let (row, range, _) = drawn
            .picks
            .iter()
            .find(|(_, _, p)| *p == Pick::Value(0, "off"))
            .cloned()
            .expect("off is drawn");
        let change = menu.click(
            &drawn,
            row + super::super::style::BODY_TOP,
            range.start,
            &settings,
        );
        assert_eq!(
            change,
            Some(Change::Set(Scope::Global, Setting::Align, "off"))
        );
    }
}
