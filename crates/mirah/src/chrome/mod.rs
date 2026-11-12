pub(crate) mod bar;
mod confirm;
pub(crate) mod keys;
pub(crate) mod launcher;
mod menu;
pub(crate) mod style;

use crate::config::{Scope, Settings};
use crate::session::MouseEvent;
use bar::{BarState, Item, TabLabel};
use confirm::Dialog;
use keys::Key;
use launcher::{Entry, Launcher, Pick};
use menu::{Change, Menu};
use mirah_bidi::{Cell, Shaping};
use mirah_term::{Overlay, OverlayRow};

const BAR_ROWS: u16 = 1;
const LEFT_BUTTON: u16 = 0;
const NEW_TAB_KEY: char = 't';
const QUIT_KEY: char = 'q';

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Action {
    Switch(usize),
    NewTab,
    Open(Entry),
    Quit,
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
    Launcher(Launcher),
}

pub(crate) struct Chrome {
    pub(crate) state: BarState,
    pub(crate) settings: Settings,
    pub(crate) dirty: bool,
    mode: Mode,
    shaping: Shaping,
    hits: Vec<(Item, std::ops::Range<usize>)>,
    boxed_at: Option<(usize, usize)>,
    boxed_width: usize,
    menu: Option<menu::Drawn>,
    dialog: Option<Dialog>,
}

impl Chrome {
    pub(crate) fn new(settings: Settings, shaping: Shaping) -> Self {
        Self {
            state: BarState::default(),
            settings,
            dirty: true,
            mode: Mode::Normal,
            shaping,
            hits: Vec::new(),
            boxed_at: None,
            boxed_width: 0,
            menu: None,
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

    pub(crate) fn set_tabs(&mut self, tabs: Vec<TabLabel>, active: usize) {
        if self.state.tabs != tabs || self.state.active != active {
            self.state.tabs = tabs;
            self.state.active = active;
            self.dirty = true;
        }
    }

    pub(crate) fn is_idle(&self) -> bool {
        self.mode == Mode::Normal
    }

    pub(crate) fn keep_open(&mut self) {
        if let Mode::Launcher(launcher) = &mut self.mode {
            launcher.can_cancel = false;
            self.dirty = true;
        }
    }

    pub(crate) fn launch(&mut self, launcher: Launcher) {
        self.set_mode(Mode::Launcher(launcher));
    }

    fn set_mode(&mut self, mode: Mode) {
        self.state.focus = match mode {
            Mode::Bar if self.state.tabs.is_empty() => Some(Item::NewTab),
            Mode::Bar => Some(Item::Tab(self.state.active)),
            _ => None,
        };
        self.mode = mode;
        self.dirty = true;
    }

    fn activate(&mut self, item: Item, actions: &mut Vec<Action>) {
        let immediate = match item {
            Item::Settings => {
                let menu = Menu::new(&self.state.program());
                self.set_mode(Mode::Menu(menu));
                return;
            }
            Item::Close => {
                self.set_mode(Mode::Confirm(false));
                return;
            }
            Item::Tab(i) => Action::Switch(i),
            Item::NewTab => Action::NewTab,
            Item::Record => Action::Record,
            Item::Update => Action::Update,
        };
        self.set_mode(Mode::Normal);
        actions.push(immediate);
    }

    fn move_focus(&mut self, leftwards: bool, actions: &mut Vec<Action>) {
        let items = self.state.items();
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
        if let Some(Item::Tab(i)) = self.state.focus
            && i != self.state.active
        {
            actions.push(Action::Switch(i));
        }
        self.dirty = true;
    }

    fn bar_key(&mut self, key: Key, actions: &mut Vec<Action>) {
        match key {
            Key::Left | Key::Tab => self.move_focus(true, actions),
            Key::Right => self.move_focus(false, actions),
            Key::Enter => {
                if let Some(item) = self.state.focus {
                    self.activate(item, actions);
                }
            }
            Key::Char(c) if keys::latin(c) == NEW_TAB_KEY => self.activate(Item::NewTab, actions),
            Key::Char(c) if keys::latin(c).to_ascii_lowercase() == QUIT_KEY => {
                self.set_mode(Mode::Normal);
                actions.push(Action::Quit);
            }
            Key::Char(c) => {
                if let Some(n) = keys::latin(c)
                    .to_digit(10)
                    .and_then(|d| usize::try_from(d).ok())
                    && n >= 1
                    && n <= self.state.tabs.len()
                {
                    self.activate(Item::Tab(n - 1), actions);
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

    fn launcher_pick(&mut self, pick: Option<Pick>, can_cancel: bool, actions: &mut Vec<Action>) {
        match pick {
            Some(Pick::Open(entry)) => {
                self.set_mode(Mode::Normal);
                actions.push(Action::Open(entry));
            }
            Some(Pick::Cancel) if can_cancel => self.set_mode(Mode::Normal),
            Some(Pick::Cancel) => actions.push(Action::Quit),
            None => self.dirty = true,
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
            Mode::Launcher(mut launcher) => {
                let pick = launcher.key(key);
                let can_cancel = launcher.can_cancel;
                self.mode = Mode::Launcher(launcher);
                self.launcher_pick(pick, can_cancel, actions);
            }
        }
    }

    pub(crate) fn input(&mut self, bytes: &[u8], actions: &mut Vec<Action>) -> Vec<u8> {
        if !self.settings.bar && self.mode == Mode::Normal {
            return bytes.to_vec();
        }
        let mut to_child = Vec::new();
        for (key, range) in keys::decode(bytes) {
            match (&self.mode, key) {
                (Mode::Normal | Mode::Launcher(_), Key::Toggle) if self.settings.bar => {
                    self.set_mode(Mode::Bar);
                }
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
        let Some((top, left)) = self.boxed_at else {
            return false;
        };
        let (row, column) = (usize::from(e.row), usize::from(e.column));
        let (Some(row), Some(column)) = (row.checked_sub(top), column.checked_sub(left)) else {
            return false;
        };
        match self.mode.clone() {
            Mode::Menu(mut menu) => {
                let Some(drawn) = self.menu.as_ref() else {
                    return false;
                };
                if column >= drawn.frame.width || row >= drawn.frame.rows.len() {
                    return false;
                }
                let change = menu.click(drawn, row, column, &self.settings);
                self.mode = Mode::Menu(menu);
                self.menu_change(change, actions);
                true
            }
            Mode::Confirm(_) => {
                let chosen = self.dialog.as_ref().and_then(|d| d.hit(row, column));
                match chosen {
                    Some(yes) => {
                        self.confirm_key(Key::Enter, yes, actions);
                        true
                    }
                    None => false,
                }
            }
            Mode::Launcher(mut launcher) => {
                if column >= self.boxed_width {
                    return false;
                }
                let pick = launcher.click(row);
                let can_cancel = launcher.can_cancel;
                self.mode = Mode::Launcher(launcher);
                let hit = pick.is_some();
                self.launcher_pick(pick, can_cancel, actions);
                hit
            }
            Mode::Normal | Mode::Bar => false,
        }
    }

    fn dismiss(&mut self) {
        match &self.mode {
            Mode::Launcher(l) if !l.can_cancel => {}
            _ => self.set_mode(Mode::Normal),
        }
    }

    pub(crate) fn mouse(&mut self, e: MouseEvent, actions: &mut Vec<Action>) -> Option<MouseEvent> {
        let top = self.top();
        let click = e.press && e.button == LEFT_BUTTON;
        if e.row < top {
            if click {
                let item = self
                    .hits
                    .iter()
                    .find(|(_, r)| r.contains(&usize::from(e.column)));
                match item.map(|(i, _)| *i) {
                    Some(item) => self.activate(item, actions),
                    None => self.dismiss(),
                }
            }
            return None;
        }
        if self.mode != Mode::Normal {
            if click && !self.click_overlay(e, actions) {
                self.dismiss();
            }
            return None;
        }
        Some(MouseEvent {
            row: e.row - top,
            ..e
        })
    }

    fn place(
        &mut self,
        rows: &mut Vec<OverlayRow>,
        cells: Vec<Vec<Cell>>,
        top: usize,
        left: usize,
    ) {
        self.boxed_at = Some((top, left));
        self.boxed_width = cells.first().map_or(0, Vec::len);
        for (i, row) in cells.into_iter().enumerate() {
            rows.push(OverlayRow {
                row: u16::try_from(top + i).unwrap_or(u16::MAX),
                column: u16::try_from(left).unwrap_or(0),
                cells: row,
            });
        }
    }

    fn layers(&mut self, width: usize, height: usize) -> Vec<OverlayRow> {
        let mut rows = Vec::new();
        self.boxed_at = None;
        self.menu = None;
        self.dialog = None;
        let under_bar = usize::from(self.top());
        match self.mode.clone() {
            Mode::Menu(menu) => {
                let drawn = menu.draw(&self.settings, self.shaping);
                let left = width.saturating_sub(drawn.frame.width);
                self.place(&mut rows, drawn.frame.rows.clone(), under_bar, left);
                self.menu = Some(drawn);
            }
            Mode::Confirm(yes) => {
                let dialog = confirm::draw(&self.state.program(), yes, self.shaping);
                let left = width.saturating_sub(dialog.frame.width);
                self.place(&mut rows, dialog.frame.rows.clone(), under_bar, left);
                self.dialog = Some(dialog);
            }
            Mode::Launcher(launcher) => {
                let frame = launcher.draw(self.shaping);
                let left = width.saturating_sub(frame.width) / 2;
                let top = height
                    .saturating_sub(frame.rows.len())
                    .div_ceil(2)
                    .max(under_bar);
                self.place(&mut rows, frame.rows, top, left);
            }
            Mode::Normal | Mode::Bar => {}
        }
        rows
    }

    pub(crate) fn overlay(&mut self, cols: u16, rows: u16) -> Overlay {
        let width = usize::from(cols);
        let mut layers = Vec::new();
        if self.settings.bar {
            let drawn = bar::draw(&self.state, width, self.shaping);
            self.hits = drawn.hits;
            layers.push(OverlayRow {
                row: 0,
                column: 0,
                cells: drawn.cells,
            });
        }
        layers.extend(self.layers(width, usize::from(rows)));
        self.dirty = false;
        Overlay {
            top: self.top(),
            rows: layers,
            hide_cursor: self.mode != Mode::Normal || self.state.tabs.is_empty(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::Agent;

    fn tabs(names: &[&str]) -> Vec<TabLabel> {
        names
            .iter()
            .map(|n| TabLabel {
                name: (*n).to_string(),
                unread: false,
            })
            .collect()
    }

    fn chrome() -> Chrome {
        let mut c = Chrome::new(Settings::default(), Shaping::Plain);
        c.set_tabs(tabs(&["claude"]), 0);
        c.overlay(80, 24);
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

    fn column_of(c: &Chrome, item: Item) -> u16 {
        c.hits
            .iter()
            .find(|(i, _)| *i == item)
            .map_or(u16::MAX, |(_, r)| u16::try_from(r.start).unwrap_or(0))
    }

    fn agent(name: &str) -> Agent {
        Agent {
            name: name.into(),
            title: name.into(),
            command: name.into(),
            resume: vec![],
        }
    }

    #[test]
    fn ctrl_bracket_opens_the_bar_and_keys_stop_reaching_the_program() {
        let mut c = chrome();
        let mut actions = Vec::new();
        assert_eq!(c.input(b"hi\x1d\x1b[D", &mut actions), b"hi");
        assert_eq!(c.state.focus, Some(Item::NewTab));
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
    fn arrows_on_tabs_switch_live_and_numbers_jump() {
        let mut c = chrome();
        c.set_tabs(tabs(&["claude", "opencode", "codex"]), 0);
        let mut actions = Vec::new();
        c.input(b"\x1d\x1b[D", &mut actions);
        assert_eq!(actions, [Action::Switch(1)]);
        actions.clear();
        c.input(b"3", &mut actions);
        assert_eq!(actions, [Action::Switch(2)]);
        assert_eq!(c.mode, Mode::Normal);
        actions.clear();
        c.input(b"\x1dt", &mut actions);
        assert_eq!(actions, [Action::NewTab]);
        actions.clear();
        c.input("\x1dف".as_bytes(), &mut actions);
        assert_eq!(actions, [Action::NewTab]);
        actions.clear();
        c.input("\x1dض".as_bytes(), &mut actions);
        assert_eq!(actions, [Action::Quit]);
        actions.clear();
        c.input("\x1d٢".as_bytes(), &mut actions);
        assert_eq!(actions, [Action::Switch(1)]);
    }

    #[test]
    fn enter_on_record_asks_the_session_to_record() {
        let mut c = chrome();
        let mut actions = Vec::new();
        c.input(b"\x1d\x1b[D\x1b[D\r", &mut actions);
        assert_eq!(actions, [Action::Record]);
    }

    #[test]
    fn closing_needs_a_yes() {
        let mut c = chrome();
        let mut actions = Vec::new();
        let close = column_of(&c, Item::Close);
        click(&mut c, 0, close, &mut actions);
        assert_eq!(c.mode, Mode::Confirm(false));
        c.input(b"\r", &mut actions);
        assert!(actions.is_empty());
        click(&mut c, 0, close, &mut actions);
        c.input(b"\x1b[C\r", &mut actions);
        assert_eq!(actions, [Action::Close]);
    }

    #[test]
    fn clicking_yes_in_the_box_closes() {
        let mut c = chrome();
        let mut actions = Vec::new();
        let close = column_of(&c, Item::Close);
        click(&mut c, 0, close, &mut actions);
        c.overlay(80, 24);
        let (top, left) = c.boxed_at.expect("the dialog is placed");
        let yes = c.dialog.as_ref().map(|d| d.yes.start).unwrap_or(0);
        let row = u16::try_from(top + style::BODY_TOP).unwrap_or(0);
        let column = u16::try_from(left + yes).unwrap_or(0);
        click(&mut c, row, column, &mut actions);
        assert_eq!(actions, [Action::Close]);
    }

    #[test]
    fn clicks_on_the_bar_stay_in_mirah_and_others_move_up_a_row() {
        let mut c = chrome();
        let mut actions = Vec::new();
        let settings = column_of(&c, Item::Settings);
        assert_eq!(click(&mut c, 0, settings, &mut actions), None);
        assert!(matches!(c.mode, Mode::Menu(_)));
        c.set_mode(Mode::Normal);
        let moved = click(&mut c, 5, 3, &mut actions).map(|e| (e.row, e.column));
        assert_eq!(moved, Some((4, 3)));
    }

    #[test]
    fn the_menu_changes_a_setting() {
        let mut c = chrome();
        let mut actions = Vec::new();
        let settings = column_of(&c, Item::Settings);
        click(&mut c, 0, settings, &mut actions);
        c.input(b"\r", &mut actions);
        assert_eq!(
            actions,
            [Action::Set(
                Scope::Global,
                crate::config::Key::Align,
                "arabic"
            )]
        );
        assert_eq!(c.settings.align, mirah_term::Align::Arabic);
        let overlay = c.overlay(80, 24);
        assert!(overlay.rows.len() > 8);
        assert!(overlay.hide_cursor);
    }

    #[test]
    fn ctrl_bracket_in_the_launcher_opens_the_bar_instead_of_quitting() {
        let mut c = Chrome::new(Settings::default(), Shaping::Plain);
        c.launch(Launcher::new(
            vec![Entry::Start(agent("claude"))],
            None,
            false,
            0,
        ));
        let mut actions = Vec::new();
        c.input(b"\x1d", &mut actions);
        assert!(actions.is_empty());
        assert_eq!(c.mode, Mode::Bar);
        c.input(b"\x1b", &mut actions);
        assert!(actions.is_empty());
        assert!(c.is_idle());
    }

    #[test]
    fn a_click_beside_the_launcher_box_chooses_nothing() {
        let mut c = Chrome::new(Settings::default(), Shaping::Plain);
        c.launch(Launcher::new(
            vec![Entry::Start(agent("claude"))],
            None,
            false,
            0,
        ));
        c.overlay(120, 30);
        let (top, left) = c.boxed_at.expect("the launcher is placed");
        let outside = u16::try_from(left + c.boxed_width).unwrap_or(0);
        let mut actions = Vec::new();
        for row in top..top + 12 {
            click(
                &mut c,
                u16::try_from(row).unwrap_or(0),
                outside,
                &mut actions,
            );
        }
        assert!(actions.is_empty());
        let inside = u16::try_from(left + 2).unwrap_or(0);
        for row in top..top + 12 {
            click(
                &mut c,
                u16::try_from(row).unwrap_or(0),
                inside,
                &mut actions,
            );
        }
        assert!(!actions.is_empty());
    }

    #[test]
    fn the_launcher_opens_a_choice_or_quits_when_nothing_runs() {
        let mut c = Chrome::new(Settings::default(), Shaping::Plain);
        let entries = vec![
            Entry::Start(agent("claude")),
            Entry::Start(agent("opencode")),
        ];
        c.launch(Launcher::new(entries.clone(), Some("opencode"), false, 0));
        let mut actions = Vec::new();
        c.input(b"\r", &mut actions);
        assert_eq!(actions, [Action::Open(entries[1].clone())]);
        assert!(c.is_idle());
        actions.clear();
        c.launch(Launcher::new(entries, None, false, 0));
        c.input(b"\x1b", &mut actions);
        assert_eq!(actions, [Action::Quit]);
        let overlay = c.overlay(80, 24);
        assert!(overlay.rows.len() > 5);
        assert!(overlay.hide_cursor);
    }

    #[test]
    fn without_the_bar_everything_passes_through() {
        let settings = Settings {
            bar: false,
            ..Settings::default()
        };
        let mut c = Chrome::new(settings, Shaping::Plain);
        c.set_tabs(tabs(&["claude"]), 0);
        let mut actions = Vec::new();
        assert_eq!(c.input(b"\x1d", &mut actions), b"\x1d");
        let overlay = c.overlay(80, 24);
        assert!(overlay.rows.is_empty());
        assert_eq!(overlay.top, 0);
    }

    const SAFE_GLYPHS: &str = "≡●■✕+↻↺◆◉○•‹›▌▐┃│─╭╮╰╯├┤↑↓←→·…[]؟،:";

    fn is_safe(c: char) -> bool {
        c.is_ascii_graphic()
            || c == ' '
            || ('\u{0600}'..='\u{06FF}').contains(&c)
            || ('\u{FB50}'..='\u{FDFF}').contains(&c)
            || ('\u{FE70}'..='\u{FEFF}').contains(&c)
            || SAFE_GLYPHS.contains(c)
    }

    fn every_drawn_text(width: u16) -> Vec<String> {
        let mut c = chrome();
        c.set_tabs(tabs(&["claude", "opencode"]), 0);
        c.state.update = Some("0.3.1".into());
        c.state.recording = Some(std::time::Duration::from_secs(75));
        c.state.notice = Some("جارٍ تثبيت 0.3.1…".into());
        let mut texts = Vec::new();
        let mut grab = |c: &mut Chrome| {
            for row in c.overlay(width, 30).rows {
                texts.push(row.cells.iter().map(|cell| cell.text.as_str()).collect());
            }
        };
        grab(&mut c);
        let mut menu = Menu::new("claude");
        menu.advanced = true;
        c.set_mode(Mode::Menu(menu));
        grab(&mut c);
        c.set_mode(Mode::Confirm(true));
        grab(&mut c);
        c.launch(Launcher::new(
            vec![Entry::Start(agent("claude"))],
            None,
            true,
            0,
        ));
        grab(&mut c);
        texts
    }

    #[test]
    fn every_glyph_is_terminal_safe() {
        for text in every_drawn_text(100) {
            let bad: Vec<char> = text.chars().filter(|c| !is_safe(*c)).collect();
            assert!(bad.is_empty(), "unsafe glyphs {bad:?} in {text}");
        }
    }

    #[test]
    fn a_narrow_bar_keeps_close_new_tab_and_the_open_tab() {
        let mut c = chrome();
        c.set_tabs(tabs(&["claude", "opencode", "codex", "gemini"]), 2);
        c.state.update = Some("0.3.1".into());
        c.overlay(40, 20);
        let items: Vec<Item> = c.hits.iter().map(|(i, _)| *i).collect();
        for item in [Item::Close, Item::NewTab, Item::Tab(2)] {
            assert!(items.contains(&item), "{item:?} missing from {items:?}");
        }
        for pair in c.hits.windows(2) {
            assert!(pair[0].1.end <= pair[1].1.start || pair[1].1.end <= pair[0].1.start);
        }
    }

    #[test]
    fn boxes_fit_inside_sixty_columns() {
        let mut c = chrome();
        let mut menu = Menu::new("opencode");
        menu.advanced = true;
        c.set_mode(Mode::Menu(menu));
        let mut layers = c.overlay(60, 30).rows;
        c.set_mode(Mode::Confirm(false));
        layers.extend(c.overlay(60, 30).rows);
        for row in layers {
            let end = usize::from(row.column) + row.cells.len();
            assert!(end <= 60, "row {} ends at {end}", row.row);
        }
    }
}
