use super::style::{
    ACCENT, BAR_BG, CHIP_BG, DIM, RECORDING, TEXT, UPDATE, blank, put, shaped, style,
};
use mirah_bidi::{Cell, Color, Shaping};
use std::ops::Range;
use std::time::Duration;

const SECONDS_PER_MINUTE: u64 = 60;
const SHORT_NAME: usize = 6;
const CLOSE_AT: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Item {
    Tab(usize),
    NewTab,
    Record,
    Update,
    Settings,
    Close,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TabLabel {
    pub(crate) name: String,
    pub(crate) unread: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct BarState {
    pub(crate) tabs: Vec<TabLabel>,
    pub(crate) active: usize,
    pub(crate) recording: Option<Duration>,
    pub(crate) update: Option<String>,
    pub(crate) focus: Option<Item>,
    pub(crate) notice: Option<String>,
}

impl BarState {
    pub(crate) fn program(&self) -> String {
        self.tabs
            .get(self.active)
            .map(|t| t.name.clone())
            .unwrap_or_default()
    }

    pub(crate) fn items(&self) -> Vec<Item> {
        let mut items: Vec<Item> = (0..self.tabs.len()).map(Item::Tab).collect();
        items.push(Item::NewTab);
        if !self.tabs.is_empty() {
            items.push(Item::Record);
        }
        if self.update.is_some() {
            items.push(Item::Update);
        }
        items.push(Item::Settings);
        if !self.tabs.is_empty() {
            items.push(Item::Close);
        }
        items
    }
}

pub(crate) struct Drawn {
    pub(crate) cells: Vec<Cell>,
    pub(crate) hits: Vec<(Item, Range<usize>)>,
}

fn clock(elapsed: Duration) -> String {
    let secs = elapsed.as_secs();
    format!(
        "{:02}:{:02}",
        secs / SECONDS_PER_MINUTE,
        secs % SECONDS_PER_MINUTE
    )
}

fn chip(text: &str, fg: Color, focused: bool, shaping: Shaping) -> Vec<Cell> {
    let s = if focused {
        style(BAR_BG, ACCENT, true)
    } else {
        style(fg, BAR_BG, false)
    };
    shaped(&format!(" {text} "), s, shaping)
}

fn tab(label: &TabLabel, active: bool, focused: bool, short: bool, shaping: Shaping) -> Vec<Cell> {
    let mut name: String = label.name.clone();
    if short && name.chars().count() > SHORT_NAME {
        name = name.chars().take(SHORT_NAME - 1).collect::<String>() + "…";
    }
    let dot = |bg| shaped("•", style(ACCENT, bg, true), shaping);
    if focused {
        return shaped(&format!(" {name} "), style(BAR_BG, ACCENT, true), shaping);
    }
    if active {
        let edge = style(CHIP_BG, BAR_BG, false);
        let mut cells = shaped("▐", edge, shaping);
        cells.extend(shaped(
            &format!(" {name} "),
            style(TEXT, CHIP_BG, true),
            shaping,
        ));
        cells.extend(shaped("▌", edge, shaping));
        return cells;
    }
    let mut cells = shaped(&format!(" {name}"), style(DIM, BAR_BG, false), shaping);
    if label.unread {
        cells.extend(shaped(" ", style(DIM, BAR_BG, false), shaping));
        cells.extend(dot(BAR_BG));
    }
    cells.extend(shaped(" ", style(DIM, BAR_BG, false), shaping));
    cells
}

fn separator(shaping: Shaping) -> Vec<Cell> {
    shaped("┃", style(Color::Rgb(52, 54, 61), BAR_BG, false), shaping)
}

fn brand(compact: bool, shaping: Shaping) -> Vec<Cell> {
    let word = if compact { " " } else { " مرآة " };
    let mut cells = shaped(word, style(TEXT, BAR_BG, true), shaping);
    cells.extend(shaped("◆ ", style(ACCENT, BAR_BG, true), shaping));
    cells
}

fn button(item: Item, state: &BarState, compact: bool, shaping: Shaping) -> Option<Vec<Cell>> {
    let focused = state.focus == Some(item);
    Some(match item {
        Item::Record => match state.recording {
            Some(_) if compact => chip("■", RECORDING, focused, shaping),
            Some(elapsed) => chip(
                &format!("■ {}", clock(elapsed)),
                RECORDING,
                focused,
                shaping,
            ),
            None => chip("●", DIM, focused, shaping),
        },
        Item::Update => {
            let version = state.update.as_ref()?;
            let text = if compact {
                "↻".to_string()
            } else {
                format!("↻ {version}")
            };
            chip(&text, UPDATE, focused, shaping)
        }
        Item::Settings => chip("≡", TEXT, focused, shaping),
        Item::Close => chip("✕", TEXT, focused, shaping),
        Item::NewTab => chip("+", ACCENT, focused, shaping),
        Item::Tab(_) => return None,
    })
}

struct Layout {
    tabs: Vec<(Item, Vec<Cell>)>,
    hidden: usize,
}

fn tab_parts(state: &BarState, short: bool, shaping: Shaping) -> Vec<(Item, Vec<Cell>)> {
    state
        .tabs
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let item = Item::Tab(i);
            let cells = tab(
                label,
                i == state.active,
                state.focus == Some(item),
                short,
                shaping,
            );
            (item, cells)
        })
        .collect()
}

fn layout(state: &BarState, room: usize, shaping: Shaping) -> Layout {
    let width = |parts: &[(Item, Vec<Cell>)]| parts.iter().map(|(_, c)| c.len() + 1).sum::<usize>();
    let full = tab_parts(state, false, shaping);
    if width(&full) <= room {
        return Layout {
            tabs: full,
            hidden: 0,
        };
    }
    let short = tab_parts(state, true, shaping);
    if width(&short) <= room {
        return Layout {
            tabs: short,
            hidden: 0,
        };
    }
    let keep = state.active.min(short.len().saturating_sub(1));
    let hidden = short.len().saturating_sub(1);
    Layout {
        tabs: short.into_iter().skip(keep).take(1).collect(),
        hidden,
    }
}

fn buttons(state: &BarState, compact: bool, shaping: Shaping) -> Vec<(Item, Vec<Cell>)> {
    let shown = state.items();
    [Item::Close, Item::Settings, Item::Update, Item::Record]
        .into_iter()
        .filter(|item| shown.contains(item))
        .filter_map(|item| Some((item, button(item, state, compact, shaping)?)))
        .collect()
}

fn smallest_tabs(state: &BarState, shaping: Shaping) -> usize {
    let active = state
        .tabs
        .get(state.active)
        .map_or(0, |label| tab(label, true, false, true, shaping).len() + 1);
    let more = if state.tabs.len() > 1 { 8 } else { 0 };
    active + more
}

pub(crate) fn draw(state: &BarState, width: usize, shaping: Shaping) -> Drawn {
    let mut cells = blank(width, style(TEXT, BAR_BG, false));
    let mut hits = Vec::new();
    let mut place = |cells: &mut Vec<Cell>, start: usize, item: Option<Item>, part: &[Cell]| {
        put(cells, start, part);
        if let Some(item) = item {
            hits.push((item, start..(start + part.len()).min(width)));
        }
    };
    let new_tab = button(Item::NewTab, state, false, shaping).unwrap_or_default();
    let sep = separator(shaping);
    let fixed = new_tab.len() + 2 * sep.len() + 2 + smallest_tabs(state, shaping);
    let used = |parts: &[(Item, Vec<Cell>)], brand: &[Cell]| {
        CLOSE_AT + parts.iter().map(|(_, c)| c.len() + 1).sum::<usize>() + brand.len() + fixed
    };
    let mut parts = buttons(state, false, shaping);
    let mut mark = brand(false, shaping);
    if used(&parts, &mark) > width {
        parts = buttons(state, true, shaping);
    }
    if used(&parts, &mark) > width {
        mark = brand(true, shaping);
    }
    let mut left = CLOSE_AT;
    for (item, part) in &parts {
        place(&mut cells, left, Some(*item), part);
        left += part.len() + 1;
    }
    let mut right = width.saturating_sub(mark.len());
    place(&mut cells, right, None, &mark);
    let room = right.saturating_sub(left + new_tab.len() + 2 * sep.len() + 2);
    let laid = layout(state, room, shaping);
    right = right.saturating_sub(sep.len());
    place(&mut cells, right, None, &sep);
    for (item, part) in &laid.tabs {
        right = right.saturating_sub(part.len());
        place(&mut cells, right, Some(*item), part);
        right = right.saturating_sub(sep.len());
        place(&mut cells, right, None, &sep);
    }
    if laid.hidden > 0 {
        let more = shaped(
            &format!("‹{}›", laid.hidden),
            style(DIM, BAR_BG, false),
            shaping,
        );
        right = right.saturating_sub(more.len());
        place(&mut cells, right, None, &more);
    }
    right = right.saturating_sub(new_tab.len());
    if right > left {
        place(&mut cells, right, Some(Item::NewTab), &new_tab);
    }
    if let Some(notice) = &state.notice {
        let text = shaped(
            &format!(" {notice} "),
            style(ACCENT, BAR_BG, false),
            shaping,
        );
        let end = right.saturating_sub(1);
        if end > left + text.len() {
            place(&mut cells, end - text.len(), None, &text);
        }
    }
    Drawn { cells, hits }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(drawn: &Drawn, column: usize) -> Option<Item> {
        drawn
            .hits
            .iter()
            .find(|(_, range)| range.contains(&column))
            .map(|(item, _)| *item)
    }

    fn start(drawn: &Drawn, item: Item) -> usize {
        drawn
            .hits
            .iter()
            .find(|(i, _)| *i == item)
            .map_or(usize::MAX, |(_, r)| r.start)
    }

    fn text(cells: &[Cell]) -> String {
        cells.iter().map(|c| c.text.as_str()).collect()
    }

    fn state(names: &[&str]) -> BarState {
        BarState {
            tabs: names
                .iter()
                .map(|n| TabLabel {
                    name: (*n).to_string(),
                    unread: false,
                })
                .collect(),
            ..BarState::default()
        }
    }

    #[test]
    fn the_brand_sits_at_the_right_and_close_at_the_left() {
        let drawn = draw(&state(&["claude"]), 80, Shaping::Plain);
        let line = text(&drawn.cells);
        assert_eq!(drawn.cells.len(), 80);
        assert!(line.trim_end().ends_with('◆'), "{line}");
        assert!(
            line.contains("ةآرم"),
            "Arabic is drawn in visual order: {line}"
        );
        assert_eq!(hit(&drawn, 2), Some(Item::Close));
    }

    #[test]
    fn tabs_run_right_to_left_and_new_tab_comes_after_them() {
        let drawn = draw(&state(&["claude", "opencode"]), 80, Shaping::Plain);
        let line = text(&drawn.cells);
        assert!(line.contains("▐ claude ▌"), "{line}");
        assert!(start(&drawn, Item::Tab(0)) > start(&drawn, Item::Tab(1)));
        assert!(start(&drawn, Item::Tab(1)) > start(&drawn, Item::NewTab));
        assert!(start(&drawn, Item::NewTab) > start(&drawn, Item::Settings));
        for pair in drawn.hits.windows(2) {
            assert!(pair[0].1.end <= pair[1].1.start || pair[1].1.end <= pair[0].1.start);
        }
    }

    #[test]
    fn an_unread_tab_has_a_dot() {
        let mut s = state(&["claude", "opencode"]);
        s.tabs[1].unread = true;
        let line = text(&draw(&s, 80, Shaping::Plain).cells);
        assert!(line.contains("opencode •"), "{line}");
    }

    #[test]
    fn the_update_item_appears_only_when_there_is_one() {
        let s = state(&["claude"]);
        assert_eq!(
            start(&draw(&s, 80, Shaping::Plain), Item::Update),
            usize::MAX
        );
        let with = BarState {
            update: Some("0.3.1".into()),
            ..s
        };
        assert_ne!(
            start(&draw(&with, 80, Shaping::Plain), Item::Update),
            usize::MAX
        );
    }

    #[test]
    fn many_tabs_shrink_and_then_collapse_to_the_active_one() {
        let names = ["claude", "opencode", "codex", "gemini", "aider", "crush"];
        let mut s = state(&names);
        s.active = 2;
        let drawn = draw(&s, 50, Shaping::Plain);
        let line = text(&drawn.cells);
        assert!(line.contains("codex"), "{line}");
        assert!(line.contains("‹5›"), "{line}");
        assert_ne!(start(&drawn, Item::Close), usize::MAX);
        assert_ne!(start(&drawn, Item::NewTab), usize::MAX);
    }

    #[test]
    fn recording_shows_a_clock() {
        let rec = BarState {
            recording: Some(Duration::from_secs(133)),
            ..state(&["claude"])
        };
        assert!(text(&draw(&rec, 80, Shaping::Plain).cells).contains("02:13"));
    }

    #[test]
    fn the_focused_item_uses_the_accent() {
        let focused = BarState {
            focus: Some(Item::Settings),
            ..state(&["claude"])
        };
        let drawn = draw(&focused, 80, Shaping::Plain);
        let at = start(&drawn, Item::Settings);
        assert_eq!(drawn.cells[at].style.bg, ACCENT);
    }

    #[test]
    fn without_tabs_only_new_tab_and_settings_show() {
        let drawn = draw(&BarState::default(), 80, Shaping::Plain);
        let items: Vec<Item> = drawn.hits.iter().map(|(i, _)| *i).collect();
        assert!(items.contains(&Item::NewTab) && items.contains(&Item::Settings));
        assert!(!items.contains(&Item::Close) && !items.contains(&Item::Record));
    }
}
