use mirah_bidi::{BidiOptions, Cell, Color, Shaping, Style, visual_row_with};
use std::ops::Range;
use std::time::Duration;

pub(crate) const BACKGROUND: Color = Color::Rgb(30, 30, 46);
pub(crate) const TEXT: Color = Color::Rgb(205, 214, 244);
pub(crate) const ACCENT: Color = Color::Rgb(137, 180, 250);
const RECORDING: Color = Color::Rgb(243, 139, 168);
const UPDATE: Color = Color::Rgb(166, 227, 161);
const GAP: usize = 2;
const SECONDS_PER_MINUTE: u64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Item {
    Settings,
    Record,
    Update,
    Close,
}

pub(crate) const ORDER: [Item; 4] = [Item::Settings, Item::Record, Item::Update, Item::Close];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct BarState {
    pub(crate) program: String,
    pub(crate) recording: Option<Duration>,
    pub(crate) update: Option<String>,
    pub(crate) focus: Option<Item>,
    pub(crate) notice: Option<String>,
}

pub(crate) struct Drawn {
    pub(crate) cells: Vec<Cell>,
    pub(crate) hits: Vec<(Item, Range<usize>)>,
}

pub(crate) fn style(fg: Color, bold: bool, focused: bool) -> Style {
    Style {
        fg,
        bg: BACKGROUND,
        bold,
        inverse: focused,
        ..Style::default()
    }
}

pub(crate) fn shaped(text: &str, style: Style, shaping: Shaping) -> Vec<Cell> {
    let logical: Vec<Cell> = text
        .chars()
        .map(|c| Cell {
            text: c.to_string().into(),
            style,
            ..Cell::default()
        })
        .collect();
    let options = BidiOptions {
        shaping,
        rtl_base: true,
    };
    visual_row_with(&logical, None, options).cells
}

fn clock(elapsed: Duration) -> String {
    let secs = elapsed.as_secs();
    format!(
        "{:02}:{:02}",
        secs / SECONDS_PER_MINUTE,
        secs % SECONDS_PER_MINUTE
    )
}

fn label(item: Item, state: &BarState) -> Option<(&'static str, String, Color)> {
    Some(match item {
        Item::Settings => ("⚙", "الإعدادات".to_string(), TEXT),
        Item::Record => match state.recording {
            Some(elapsed) => ("■", clock(elapsed), RECORDING),
            None => ("●", "تسجيل".to_string(), TEXT),
        },
        Item::Update => ("↻", state.update.clone()?, UPDATE),
        Item::Close => ("✕", String::new(), TEXT),
    })
}

fn button(icon: &str, word: &str, style: Style, shaping: Shaping) -> Vec<Cell> {
    let mut cells = shaped(" ", style, shaping);
    if !word.is_empty() {
        cells.extend(shaped(word, style, shaping));
        cells.extend(shaped(" ", style, shaping));
    }
    cells.extend(shaped(&format!("{icon} "), style, shaping));
    cells
}

fn items(state: &BarState, short: bool, shaping: Shaping) -> Vec<(Item, Vec<Cell>)> {
    ORDER
        .iter()
        .filter_map(|&item| {
            let (icon, word, color) = label(item, state)?;
            let style = style(color, false, state.focus == Some(item));
            let word = if short { "" } else { word.as_str() };
            Some((item, button(icon, word, style, shaping)))
        })
        .collect()
}

fn brand(state: &BarState, shaping: Shaping) -> Vec<Cell> {
    let text = match &state.notice {
        Some(notice) => notice.clone(),
        None => format!("مرآة · {}", state.program),
    };
    button("◆", &text, style(ACCENT, true, false), shaping)
}

fn fits(width: usize, brand: &[Cell], items: &[(Item, Vec<Cell>)]) -> bool {
    let used: usize = brand.len() + items.iter().map(|(_, c)| c.len() + GAP).sum::<usize>();
    used <= width
}

fn place(cells: &mut [Cell], start: usize, part: Vec<Cell>) {
    for (offset, cell) in part.into_iter().enumerate() {
        if let Some(slot) = cells.get_mut(start + offset) {
            *slot = cell;
        }
    }
}

fn fitted(
    state: &BarState,
    width: usize,
    brand: &[Cell],
    shaping: Shaping,
) -> Vec<(Item, Vec<Cell>)> {
    let mut parts = items(state, false, shaping);
    if !fits(width, brand, &parts) {
        parts = items(state, true, shaping);
    }
    while !fits(width, brand, &parts) && parts.len() > 1 {
        parts.remove(parts.len() - 2);
    }
    parts
}

pub(crate) fn draw(state: &BarState, width: usize, shaping: Shaping) -> Drawn {
    let blank = Cell {
        text: " ".into(),
        style: style(TEXT, false, false),
        ..Cell::default()
    };
    let mut cells = vec![blank; width];
    let brand = brand(state, shaping);
    let parts = fitted(state, width, &brand, shaping);
    let mut right = width.saturating_sub(brand.len());
    place(&mut cells, right, brand);
    let mut hits = Vec::new();
    for (item, label) in parts {
        let begin = if item == Item::Close {
            0
        } else {
            right.saturating_sub(label.len() + GAP)
        };
        hits.push((item, begin..(begin + label.len()).min(width)));
        place(&mut cells, begin, label);
        if item != Item::Close {
            right = begin;
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

    fn text(cells: &[Cell]) -> String {
        cells.iter().map(|c| c.text.as_str()).collect()
    }

    fn state() -> BarState {
        BarState {
            program: "claude".into(),
            ..BarState::default()
        }
    }

    #[test]
    fn the_brand_sits_at_the_right_and_close_at_the_left() {
        let drawn = draw(&state(), 80, Shaping::Plain);
        let line = text(&drawn.cells);
        assert_eq!(drawn.cells.len(), 80);
        assert!(line.trim_end().ends_with("◆"), "{line}");
        assert!(line.contains("claude"), "{line}");
        assert_eq!(hit(&drawn, 1), Some(Item::Close));
        assert!(
            line.contains("ةآرم"),
            "Arabic is drawn in visual order: {line}"
        );
    }

    #[test]
    fn items_do_not_overlap_and_follow_right_to_left_order() {
        let drawn = draw(&state(), 80, Shaping::Plain);
        let starts: Vec<(Item, usize)> = drawn.hits.iter().map(|(i, r)| (*i, r.start)).collect();
        let settings = starts
            .iter()
            .find(|(i, _)| *i == Item::Settings)
            .map(|(_, s)| *s);
        let record = starts
            .iter()
            .find(|(i, _)| *i == Item::Record)
            .map(|(_, s)| *s);
        assert!(settings > record, "{starts:?}");
        for pair in drawn.hits.windows(2) {
            assert!(pair[0].1.end <= pair[1].1.start || pair[1].1.end <= pair[0].1.start);
        }
    }

    #[test]
    fn the_update_item_appears_only_when_there_is_one() {
        assert!(
            !draw(&state(), 80, Shaping::Plain)
                .hits
                .iter()
                .any(|(i, _)| *i == Item::Update)
        );
        let with = BarState {
            update: Some("0.2.1".into()),
            ..state()
        };
        assert!(
            draw(&with, 80, Shaping::Plain)
                .hits
                .iter()
                .any(|(i, _)| *i == Item::Update)
        );
    }

    #[test]
    fn a_narrow_bar_falls_back_to_icons_and_keeps_close() {
        let drawn = draw(&state(), 30, Shaping::Plain);
        let line = text(&drawn.cells);
        assert!(!line.contains("تادادعإلا"), "{line}");
        assert!(drawn.hits.iter().any(|(i, _)| *i == Item::Close));
    }

    #[test]
    fn recording_shows_a_clock() {
        let rec = BarState {
            recording: Some(Duration::from_secs(133)),
            ..state()
        };
        assert!(text(&draw(&rec, 80, Shaping::Plain).cells).contains("02:13"));
    }

    #[test]
    fn the_focused_item_is_inverse() {
        let focused = BarState {
            focus: Some(Item::Settings),
            ..state()
        };
        let drawn = draw(&focused, 80, Shaping::Plain);
        let range = drawn
            .hits
            .iter()
            .find(|(i, _)| *i == Item::Settings)
            .map(|(_, r)| r.clone());
        let range = range.unwrap_or(0..0);
        assert!(drawn.cells[range].iter().all(|c| c.style.inverse));
    }
}
