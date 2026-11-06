pub(crate) mod bar;
mod confirm;
pub(crate) mod keys;
mod menu;

use crate::config::{Scope, Settings};
use crate::session::MouseEvent;
use bar::{BarState, Item};
use keys::Key;
use menu::{Change, Menu};
use mirah_bidi::Shaping;
use mirah_term::{Overlay, OverlayRow};

const BAR_ROWS: u16 = 1;
const LEFT_BUTTON: u16 = 0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Action {
    Record,
    Update,
    Close,
    Set(Scope, crate::config::Key, &'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Mode {
    Normal,
    Bar,
    Menu(Menu),
    Confirm(bool),
}

pub(crate) struct Chrome {
    pub(crate) state: BarState,
    pub(crate) settings: Settings,
    pub(crate) dirty: bool,
    mode: Mode,
    shaping: Shaping,
    hits: Vec<(Item, std::ops::Range<usize>)>,
    menu_at: Option<(usize, usize)>,
    dialog: Option<(std::ops::Range<usize>, std::ops::Range<usize>)>,
}

impl Chrome {
    pub(crate) fn new(program: &str, settings: Settings, shaping: Shaping) -> Self {
        Self {
            state: BarState {
                program: crate::config::program_name(program),
                ..BarState::default()
            },
            settings,
            dirty: true,
            mode: Mode::Normal,
            shaping,
            hits: Vec::new(),
            menu_at: None,
            dialog: None,
        }
    }

    pub(crate) fn top(&self) -> u16 {
        if self.settings.bar { BAR_ROWS } else { 0 }
    }

    pub(crate) fn set_shaping(&mut self, shaping: Shaping) {
        self.shaping = shaping;
        self.dirty = true;
    }

    fn set_mode(&mut self, mode: Mode) {
        self.state.focus = match mode {
            Mode::Bar => Some(Item::Settings),
            _ => None,
        };
        self.mode = mode;
        self.dirty = true;
    }

    fn activate(&mut self, item: Item, actions: &mut Vec<Action>) {
        match item {
            Item::Settings => self.set_mode(Mode::Menu(Menu::new(&self.state.program))),
            Item::Record => {
                self.set_mode(Mode::Normal);
                actions.push(Action::Record);
            }
            Item::Update => {
                self.set_mode(Mode::Normal);
                actions.push(Action::Update);
            }
            Item::Close => self.set_mode(Mode::Confirm(false)),
        }
    }

    fn visible_items(&self) -> Vec<Item> {
        bar::ORDER
            .iter()
            .copied()
            .filter(|i| *i != Item::Update || self.state.update.is_some())
            .collect()
    }

    fn move_focus(&mut self, leftwards: bool) {
        let items = self.visible_items();
        let current = self
            .state
            .focus
            .and_then(|f| items.iter().position(|i| *i == f));
        let index = current.unwrap_or(0);
        let next = if leftwards {
            (index + 1).min(items.len().saturating_sub(1))
        } else {
            index.saturating_sub(1)
        };
        self.state.focus = items.get(next).copied();
        self.dirty = true;
    }

    fn bar_key(&mut self, key: Key, actions: &mut Vec<Action>) {
        match key {
            Key::Left | Key::Tab => self.move_focus(true),
            Key::Right => self.move_focus(false),
            Key::Enter => {
                if let Some(item) = self.state.focus {
                    self.activate(item, actions);
                }
            }
            Key::Escape | Key::Toggle => self.set_mode(Mode::Normal),
            _ => {}
        }
    }

    fn menu_change(&mut self, change: Option<Change>, actions: &mut Vec<Action>) {
        match change {
            Some(Change::Set(scope, key, value)) => {
                self.settings.apply(key, value);
                actions.push(Action::Set(scope, key, value));
            }
            Some(Change::Close) => self.set_mode(Mode::Normal),
            None => {}
        }
        self.dirty = true;
    }

    fn confirm_key(&mut self, key: Key, yes: bool, actions: &mut Vec<Action>) {
        match key {
            Key::Left | Key::Right | Key::Tab => self.set_mode(Mode::Confirm(!yes)),
            Key::Enter if yes => {
                self.set_mode(Mode::Normal);
                actions.push(Action::Close);
            }
            Key::Enter | Key::Escape | Key::Toggle => self.set_mode(Mode::Normal),
            _ => {}
        }
    }

    fn key(&mut self, key: Key, actions: &mut Vec<Action>) {
        match self.mode.clone() {
            Mode::Normal => {}
            Mode::Bar => self.bar_key(key, actions),
            Mode::Menu(mut menu) => {
                let change = menu.key(key, &self.settings);
                self.mode = Mode::Menu(menu);
                self.menu_change(change, actions);
            }
            Mode::Confirm(yes) => self.confirm_key(key, yes, actions),
        }
    }

    pub(crate) fn input(&mut self, bytes: &[u8], actions: &mut Vec<Action>) -> Vec<u8> {
        if !self.settings.bar {
            return bytes.to_vec();
        }
        let mut to_child = Vec::new();
        for (key, range) in keys::decode(bytes) {
            match (&self.mode, key) {
                (Mode::Normal, Key::Toggle) => self.set_mode(Mode::Bar),
                (Mode::Normal, _) => to_child.extend_from_slice(&bytes[range]),
                (_, Key::Release) => {}
                (Mode::Bar, Key::Toggle) => {
                    self.set_mode(Mode::Normal);
                    to_child.extend_from_slice(&bytes[range]);
                }
                _ => self.key(key, actions),
            }
        }
        to_child
    }

    fn click_overlay(&mut self, e: MouseEvent, actions: &mut Vec<Action>) -> bool {
        let (row, column) = (usize::from(e.row), usize::from(e.column));
        if let (Mode::Menu(menu), Some((top, left))) = (&self.mode, self.menu_at)
            && row >= top
            && column >= left
        {
            let mut menu = menu.clone();
            let change = menu.click(row - top, &self.settings);
            self.mode = Mode::Menu(menu);
            self.menu_change(change, actions);
            return true;
        }
        if let (Mode::Confirm(_), Some((yes, no))) = (&self.mode, self.dialog.clone())
            && row == usize::from(BAR_ROWS)
            && (yes.contains(&column) || no.contains(&column))
        {
            let chosen = yes.contains(&column);
            self.confirm_key(Key::Enter, chosen, actions);
            return true;
        }
        false
    }

    pub(crate) fn mouse(&mut self, e: MouseEvent, actions: &mut Vec<Action>) -> Option<MouseEvent> {
        let top = self.top();
        if top == 0 {
            return Some(e);
        }
        let click = e.press && e.button == LEFT_BUTTON;
        if e.row < top {
            if click {
                let item = self
                    .hits
                    .iter()
                    .find(|(_, r)| r.contains(&usize::from(e.column)));
                match item.map(|(i, _)| *i) {
                    Some(item) => self.activate(item, actions),
                    None => self.set_mode(Mode::Normal),
                }
            }
            return None;
        }
        if self.mode != Mode::Normal {
            if click && !self.click_overlay(e, actions) {
                self.set_mode(Mode::Normal);
            }
            return None;
        }
        Some(MouseEvent {
            row: e.row - top,
            ..e
        })
    }

    fn layers(&mut self, width: usize) -> Vec<OverlayRow> {
        let mut rows = Vec::new();
        self.menu_at = None;
        self.dialog = None;
        match &self.mode {
            Mode::Menu(menu) => {
                let lines = menu.lines(&self.settings, self.shaping);
                let left = width.saturating_sub(lines.first().map_or(0, Vec::len));
                self.menu_at = Some((usize::from(BAR_ROWS), left));
                for (i, cells) in lines.into_iter().enumerate() {
                    rows.push(OverlayRow {
                        row: BAR_ROWS + u16::try_from(i).unwrap_or(0),
                        column: u16::try_from(left).unwrap_or(0),
                        cells,
                    });
                }
            }
            Mode::Confirm(yes) => {
                let dialog = confirm::draw(&self.state.program, *yes, self.shaping);
                self.dialog = Some((dialog.yes, dialog.no));
                rows.push(OverlayRow {
                    row: BAR_ROWS,
                    column: 0,
                    cells: dialog.cells,
                });
            }
            Mode::Normal | Mode::Bar => {}
        }
        rows
    }

    pub(crate) fn overlay(&mut self, cols: u16) -> Overlay {
        if !self.settings.bar {
            return Overlay::default();
        }
        let width = usize::from(cols);
        let drawn = bar::draw(&self.state, width, self.shaping);
        self.hits = drawn.hits;
        let mut rows = vec![OverlayRow {
            row: 0,
            column: 0,
            cells: drawn.cells,
        }];
        rows.extend(self.layers(width));
        self.dirty = false;
        Overlay {
            top: BAR_ROWS,
            rows,
            hide_cursor: self.mode != Mode::Normal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chrome() -> Chrome {
        let mut c = Chrome::new("claude.exe", Settings::default(), Shaping::Plain);
        c.overlay(80);
        c
    }

    fn click(
        c: &mut Chrome,
        row: u16,
        column: u16,
        actions: &mut Vec<Action>,
    ) -> Option<MouseEvent> {
        c.mouse(
            MouseEvent {
                button: 0,
                column,
                row,
                press: true,
            },
            actions,
        )
    }

    #[test]
    fn ctrl_bracket_opens_the_bar_and_keys_stop_reaching_the_program() {
        let mut c = chrome();
        let mut actions = Vec::new();
        assert_eq!(c.input(b"hi\x1d\x1b[D", &mut actions), b"hi");
        assert_eq!(c.state.focus, Some(Item::Record));
        assert_eq!(c.input(b"\x1b", &mut actions), b"");
        assert_eq!(c.mode, Mode::Normal);
        assert_eq!(c.input(b"x", &mut actions), b"x");
    }

    #[test]
    fn pressing_the_shortcut_twice_sends_it_to_the_program() {
        let mut c = chrome();
        let mut actions = Vec::new();
        assert_eq!(c.input(b"\x1d\x1d", &mut actions), b"\x1d");
        assert_eq!(c.mode, Mode::Normal);
    }

    #[test]
    fn enter_on_record_asks_the_session_to_record() {
        let mut c = chrome();
        let mut actions = Vec::new();
        c.input(b"\x1d\x1b[D\r", &mut actions);
        assert_eq!(actions, [Action::Record]);
    }

    #[test]
    fn closing_needs_a_yes() {
        let mut c = chrome();
        let mut actions = Vec::new();
        c.input(b"\x1d\x1b[D\x1b[D\r", &mut actions);
        assert_eq!(c.mode, Mode::Confirm(false));
        c.input(b"\r", &mut actions);
        assert!(actions.is_empty());
        c.input(b"\x1d\x1b[D\x1b[D\r\x1b[C\r", &mut actions);
        assert_eq!(actions, [Action::Close]);
    }

    #[test]
    fn clicks_on_the_bar_stay_in_mirah_and_others_move_up_a_row() {
        let mut c = chrome();
        let mut actions = Vec::new();
        assert_eq!(click(&mut c, 0, 1, &mut actions), None);
        assert_eq!(c.mode, Mode::Confirm(false));
        c.set_mode(Mode::Normal);
        let moved = click(&mut c, 5, 3, &mut actions).map(|e| (e.row, e.column));
        assert_eq!(moved, Some((4, 3)));
    }

    #[test]
    fn the_menu_changes_a_setting() {
        let mut c = chrome();
        let mut actions = Vec::new();
        c.input(b"\x1d\r\r", &mut actions);
        assert!(matches!(c.mode, Mode::Menu(_)));
        assert_eq!(
            actions,
            [Action::Set(
                Scope::Global,
                crate::config::Key::Align,
                "arabic"
            )]
        );
        assert_eq!(c.settings.align, mirah_term::Align::Arabic);
        let overlay = c.overlay(80);
        assert!(overlay.rows.len() > 2);
        assert!(overlay.hide_cursor);
    }

    #[test]
    fn without_the_bar_everything_passes_through() {
        let settings = Settings {
            bar: false,
            ..Settings::default()
        };
        let mut c = Chrome::new("claude", settings, Shaping::Plain);
        let mut actions = Vec::new();
        assert_eq!(c.input(b"\x1d", &mut actions), b"\x1d");
        assert_eq!(c.overlay(80), Overlay::default());
    }
}
