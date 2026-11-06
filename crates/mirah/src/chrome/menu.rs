use super::bar::{ACCENT, TEXT, shaped, style};
use super::keys::Key;
use crate::config::{SPECS, Scope, Settings, Spec};
use mirah_bidi::{Cell, Shaping};

const MARGIN: usize = 2;
const HINT: &str = "↑↓ تنقّل · ←→ تغيير · Esc قفل";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Change {
    Set(Scope, crate::config::Key, &'static str),
    Close,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Menu {
    pub(crate) row: usize,
    pub(crate) scope: Scope,
    program: String,
}

fn value_label(spec: &Spec, settings: &Settings) -> &'static str {
    let current = settings.value(spec.key);
    spec.values
        .iter()
        .find(|(v, _)| *v == current)
        .map_or(current, |(_, label)| label)
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

impl Menu {
    pub(crate) fn new(program: &str) -> Self {
        Self {
            row: 0,
            scope: Scope::Global,
            program: program.to_string(),
        }
    }

    fn rows() -> usize {
        SPECS.len() + 1
    }

    fn scope_label(&self) -> String {
        match self.scope {
            Scope::Global => "كل البرامج".to_string(),
            Scope::Program(_) => format!("{} بس", self.program),
        }
    }

    fn change(&mut self, settings: &Settings, forward: bool) -> Option<Change> {
        match SPECS.get(self.row) {
            Some(spec) => Some(Change::Set(
                self.scope.clone(),
                spec.key,
                step(spec, settings, forward),
            )),
            None => {
                self.scope = match self.scope {
                    Scope::Global => Scope::Program(self.program.clone()),
                    Scope::Program(_) => Scope::Global,
                };
                None
            }
        }
    }

    pub(crate) fn key(&mut self, key: Key, settings: &Settings) -> Option<Change> {
        match key {
            Key::Up => self.row = (self.row + Self::rows() - 1) % Self::rows(),
            Key::Down | Key::Tab => self.row = (self.row + 1) % Self::rows(),
            Key::Left | Key::Enter => return self.change(settings, true),
            Key::Right => return self.change(settings, false),
            Key::Escape | Key::Toggle => return Some(Change::Close),
            _ => {}
        }
        None
    }

    pub(crate) fn click(&mut self, row: usize, settings: &Settings) -> Option<Change> {
        if row >= Self::rows() {
            return None;
        }
        self.row = row;
        self.change(settings, true)
    }

    pub(crate) fn lines(&self, settings: &Settings, shaping: Shaping) -> Vec<Vec<Cell>> {
        let mut texts: Vec<(String, bool)> = SPECS
            .iter()
            .enumerate()
            .map(|(i, spec)| {
                (
                    format!("{}: {}", spec.label, value_label(spec, settings)),
                    i == self.row,
                )
            })
            .collect();
        texts.push((
            format!("يتطبّق على: {}", self.scope_label()),
            self.row == SPECS.len(),
        ));
        let mut lines: Vec<Vec<Cell>> = texts
            .iter()
            .map(|(text, focused)| shaped(text, style(TEXT, false, *focused), shaping))
            .collect();
        lines.push(shaped(HINT, style(ACCENT, false, false), shaping));
        let width = lines.iter().map(Vec::len).max().unwrap_or(0) + 2 * MARGIN;
        let pad = shaped(" ", style(TEXT, false, false), shaping);
        lines
            .into_iter()
            .map(|line| {
                let left = width - MARGIN - line.len();
                let mut row: Vec<Cell> = pad.iter().cycle().take(left).cloned().collect();
                row.extend(line);
                row.extend(pad.iter().cycle().take(MARGIN).cloned());
                row
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Key as Setting;

    #[test]
    fn arrows_move_and_wrap() {
        let mut menu = Menu::new("claude");
        let settings = Settings::default();
        assert_eq!(menu.key(Key::Up, &settings), None);
        assert_eq!(menu.row, SPECS.len());
        assert_eq!(menu.key(Key::Down, &settings), None);
        assert_eq!(menu.row, 0);
    }

    #[test]
    fn enter_cycles_the_value_for_the_chosen_scope() {
        let mut menu = Menu::new("claude");
        let settings = Settings::default();
        assert_eq!(
            menu.key(Key::Enter, &settings),
            Some(Change::Set(Scope::Global, Setting::Align, "arabic"))
        );
        menu.row = SPECS.len();
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
    fn every_line_has_the_same_width_and_shows_values_in_arabic() {
        let menu = Menu::new("claude");
        let lines = menu.lines(&Settings::default(), Shaping::Plain);
        assert_eq!(lines.len(), SPECS.len() + 2);
        assert!(lines.windows(2).all(|w| w[0].len() == w[1].len()));
        let first: String = lines[0].iter().map(|c| c.text.as_str()).collect();
        assert!(first.contains("هلك مالكلا"), "{first}");
        assert!(lines[0].iter().any(|c| c.style.inverse));
    }
}
