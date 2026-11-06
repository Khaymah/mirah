use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use unicode_width::UnicodeWidthChar;

const RESET: &str = "\x1b[0m";
const ACCENT: &str = "38;2;217;145;104";
const DIM: &str = "38;2;140;144;154";
const CODE_FG: &str = "38;2;132;196;220";
const CODE_INDENT: usize = 4;
const MIN_WIDTH: usize = 20;
const CELL_PAD: usize = 1;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Look {
    bold: bool,
    italic: bool,
    strike: bool,
    link: bool,
    code: bool,
    heading: u8,
}

impl Look {
    fn sgr(self) -> String {
        let mut parts: Vec<&str> = Vec::new();
        if self.bold || self.heading > 0 {
            parts.push("1");
        }
        if self.italic {
            parts.push("3");
        }
        if self.link {
            parts.push("4");
        }
        if self.strike {
            parts.push("9");
        }
        match self.heading {
            1 => parts.push(ACCENT),
            3.. => parts.push(DIM),
            _ => {}
        }
        if self.code {
            parts.push(CODE_FG);
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!("\x1b[{}m", parts.join(";"))
        }
    }
}

#[derive(Debug, Clone)]
struct Span {
    text: String,
    look: Look,
}

enum ListKind {
    Bullet,
    Ordered(u64),
}

#[derive(Default)]
struct Table {
    rows: Vec<Vec<String>>,
    head: usize,
    cell: String,
}

struct Renderer {
    width: usize,
    lines: Vec<String>,
    spans: Vec<Span>,
    look: Look,
    lists: Vec<ListKind>,
    marker: Option<String>,
    quotes: usize,
    code: Option<String>,
    table: Option<Table>,
    link: Option<String>,
}

fn width_of(text: &str) -> usize {
    text.chars().map(|c| c.width().unwrap_or(0)).sum()
}

fn pad_to(text: &str, width: usize) -> String {
    let have = width_of(text);
    format!("{text}{}", " ".repeat(width.saturating_sub(have)))
}

fn clip(text: &str, width: usize) -> String {
    if width_of(text) <= width {
        return text.to_string();
    }
    let mut out = String::new();
    for c in text.chars() {
        if width_of(&out) + width_of(&c.to_string()) + 1 > width {
            break;
        }
        out.push(c);
    }
    out.push('…');
    out
}

impl Renderer {
    fn new(width: usize) -> Self {
        Self {
            width: width.max(MIN_WIDTH),
            lines: Vec::new(),
            spans: Vec::new(),
            look: Look::default(),
            lists: Vec::new(),
            marker: None,
            quotes: 0,
            code: None,
            table: None,
            link: None,
        }
    }

    fn quote_prefix(&self) -> String {
        format!("\x1b[{DIM}m{}{RESET}", "│ ".repeat(self.quotes))
    }

    fn indent(&self) -> usize {
        2 * self.lists.len().saturating_sub(1) + 2 * self.quotes
    }

    fn blank(&mut self) {
        if self.lines.last().is_some_and(|l| !l.is_empty()) {
            self.lines.push(String::new());
        }
    }

    fn push_line(&mut self, body: &str) {
        self.lines
            .push(format!("{}{body}{RESET}", self.quote_prefix()));
    }

    fn text(&mut self, text: &str) {
        if let Some(table) = self.table.as_mut() {
            table.cell.push_str(text);
            return;
        }
        if let Some(code) = self.code.as_mut() {
            code.push_str(text);
            return;
        }
        self.spans.push(Span {
            text: text.to_string(),
            look: self.look,
        });
    }

    fn words(&self) -> Vec<Span> {
        let mut words: Vec<Span> = Vec::new();
        for span in &self.spans {
            for (i, part) in span.text.split(' ').enumerate() {
                if i > 0 {
                    words.push(Span {
                        text: " ".to_string(),
                        look: span.look,
                    });
                }
                if !part.is_empty() {
                    words.push(Span {
                        text: part.to_string(),
                        look: span.look,
                    });
                }
            }
        }
        words
    }

    fn flush(&mut self) {
        if self.spans.is_empty() && self.marker.is_none() {
            return;
        }
        let marker = self.marker.take();
        let lead = " ".repeat(self.indent().saturating_sub(2 * self.quotes));
        let first = match &marker {
            Some(m) => format!("{lead}\x1b[{ACCENT}m{m}{RESET}"),
            None if self.lists.is_empty() => String::new(),
            None => format!("{lead}  "),
        };
        let hang = " ".repeat(width_of(&strip(&first)));
        let room = self
            .width
            .saturating_sub(2 * self.quotes + width_of(&hang))
            .max(MIN_WIDTH / 2);
        let mut lines: Vec<(usize, String)> = vec![(0, String::new())];
        for word in self.words() {
            let w = width_of(&word.text);
            let (used, _) = lines.last().map_or((0, String::new()), Clone::clone);
            if word.text == " " {
                if used > 0
                    && used < room
                    && let Some(last) = lines.last_mut()
                {
                    last.0 += 1;
                    last.1.push(' ');
                }
                continue;
            }
            if used > 0 && used + w > room {
                if let Some(last) = lines.last_mut() {
                    let trimmed = last.1.trim_end().to_string();
                    last.1 = trimmed;
                }
                lines.push((0, String::new()));
            }
            if let Some(last) = lines.last_mut() {
                last.0 += w;
                last.1
                    .push_str(&format!("{}{}{RESET}", word.look.sgr(), word.text));
            }
        }
        for (i, (_, body)) in lines.into_iter().enumerate() {
            let start = if i == 0 { first.clone() } else { hang.clone() };
            self.push_line(&format!("{start}{}", body.trim_end()));
        }
        self.spans.clear();
    }

    fn code_block(&mut self, text: &str) {
        let room = self.width.saturating_sub(2 * self.quotes + CODE_INDENT);
        self.blank();
        for line in text.trim_end_matches('\n').split('\n') {
            let shown = clip(&line.replace('\t', "    "), room);
            self.push_line(&format!(
                "{}\x1b[{CODE_FG}m{shown}",
                " ".repeat(CODE_INDENT)
            ));
        }
        self.blank();
    }

    fn table(&mut self, table: Table) {
        let columns = table.rows.iter().map(Vec::len).max().unwrap_or(0);
        if columns == 0 {
            return;
        }
        let mut widths: Vec<usize> = (0..columns)
            .map(|c| {
                table
                    .rows
                    .iter()
                    .filter_map(|r| r.get(c))
                    .map(|t| width_of(t))
                    .max()
                    .unwrap_or(0)
            })
            .collect();
        let room = self
            .width
            .saturating_sub(2 * self.quotes + columns * (1 + 2 * CELL_PAD) + 1);
        while widths.iter().sum::<usize>() > room {
            let Some(widest) = widths.iter().copied().max().filter(|w| *w > 3) else {
                break;
            };
            if let Some(slot) = widths.iter_mut().find(|w| **w == widest) {
                *slot -= 1;
            }
        }
        let rule = |left: &str, mid: &str, right: &str| {
            let parts: Vec<String> = widths
                .iter()
                .map(|w| "─".repeat(w + 2 * CELL_PAD))
                .collect();
            format!("\x1b[{DIM}m{left}{}{right}", parts.join(mid))
        };
        let arabic = table
            .rows
            .iter()
            .flatten()
            .any(|cell| cell.chars().any(|c| ('\u{0600}'..='\u{06FF}').contains(&c)));
        let total = widths.iter().map(|w| w + 2 * CELL_PAD + 1).sum::<usize>() + 1;
        let shift = if arabic {
            " ".repeat(self.width.saturating_sub(2 * self.quotes + total))
        } else {
            String::new()
        };
        let rule =
            |left: &str, mid: &str, right: &str| format!("{shift}{}", rule(left, mid, right));
        self.blank();
        self.push_line(&rule("┌", "┬", "┐"));
        for (i, row) in table.rows.iter().enumerate() {
            let bold = i < table.head;
            let cells: Vec<String> = (0..columns)
                .map(|c| {
                    let text = clip(row.get(c).map_or("", String::as_str), widths[c]);
                    let look = if bold { "\x1b[1m" } else { "" };
                    format!(" {look}{}{RESET} ", pad_to(&text, widths[c]))
                })
                .collect();
            let bar = format!("\x1b[{DIM}m│{RESET}");
            self.push_line(&format!("{shift}{bar}{}{bar}", cells.join(&bar)));
            if bold && i + 1 == table.head {
                self.push_line(&rule("├", "┼", "┤"));
            }
        }
        self.push_line(&rule("└", "┴", "┘"));
        self.blank();
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Paragraph => {}
            Tag::Heading { level, .. } => {
                self.blank();
                self.look.heading = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    _ => 3,
                };
            }
            Tag::BlockQuote(_) => {
                self.flush();
                self.blank();
                self.quotes += 1;
            }
            Tag::CodeBlock(_) => {
                self.flush();
                self.code = Some(String::new());
            }
            Tag::List(start) => {
                self.flush();
                if self.lists.is_empty() {
                    self.blank();
                }
                self.lists.push(match start {
                    Some(n) => ListKind::Ordered(n),
                    None => ListKind::Bullet,
                });
            }
            Tag::Item => {
                self.flush();
                self.marker = Some(match self.lists.last_mut() {
                    Some(ListKind::Ordered(n)) => {
                        let m = format!("{n}. ");
                        *n += 1;
                        m
                    }
                    _ => "• ".to_string(),
                });
            }
            Tag::Table(_) => {
                self.flush();
                self.table = Some(Table::default());
            }
            Tag::TableHead | Tag::TableRow => {
                if let Some(t) = self.table.as_mut() {
                    t.rows.push(Vec::new());
                }
            }
            Tag::TableCell => {
                if let Some(t) = self.table.as_mut() {
                    t.cell.clear();
                }
            }
            Tag::Emphasis => self.look.italic = true,
            Tag::Strong => self.look.bold = true,
            Tag::Strikethrough => self.look.strike = true,
            Tag::Link { dest_url, .. } => {
                self.look.link = true;
                self.link = Some(dest_url.to_string());
            }
            Tag::Image { dest_url, .. } => self.text(&format!("[{dest_url}]")),
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => {
                self.flush();
                if self.lists.is_empty() {
                    self.blank();
                }
            }
            TagEnd::Heading(_) => {
                self.flush();
                self.look.heading = 0;
                self.blank();
            }
            TagEnd::BlockQuote(_) => {
                self.flush();
                self.quotes = self.quotes.saturating_sub(1);
                self.blank();
            }
            TagEnd::CodeBlock => {
                if let Some(code) = self.code.take() {
                    self.code_block(&code);
                }
            }
            TagEnd::List(_) => {
                self.flush();
                self.lists.pop();
                if self.lists.is_empty() {
                    self.blank();
                }
            }
            TagEnd::Item => self.flush(),
            TagEnd::TableHead => {
                if let Some(t) = self.table.as_mut() {
                    t.head = t.rows.len();
                }
            }
            TagEnd::TableCell => {
                if let Some(t) = self.table.as_mut() {
                    let cell = std::mem::take(&mut t.cell).trim().to_string();
                    if let Some(row) = t.rows.last_mut() {
                        row.push(cell);
                    }
                }
            }
            TagEnd::Table => {
                if let Some(table) = self.table.take() {
                    self.table(table);
                }
            }
            TagEnd::Emphasis => self.look.italic = false,
            TagEnd::Strong => self.look.bold = false,
            TagEnd::Strikethrough => self.look.strike = false,
            TagEnd::Link => {
                self.look.link = false;
                if let Some(url) = self.link.take()
                    && !url.is_empty()
                    && self.table.is_none()
                {
                    self.spans.push(Span {
                        text: format!(" ({url})"),
                        look: Look {
                            italic: true,
                            ..Look::default()
                        },
                    });
                }
            }
            _ => {}
        }
    }

    fn event(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) => self.text(&text),
            Event::Code(text) => {
                if let Some(table) = self.table.as_mut() {
                    table.cell.push_str(&text);
                } else {
                    let look = Look {
                        code: true,
                        ..self.look
                    };
                    self.spans.push(Span {
                        text: text.to_string(),
                        look,
                    });
                }
            }
            Event::SoftBreak => self.text(" "),
            Event::HardBreak => self.flush(),
            Event::Rule => {
                self.flush();
                self.blank();
                let rule = "─".repeat(self.width.saturating_sub(2 * self.quotes));
                self.push_line(&format!("\x1b[{DIM}m{rule}"));
                self.blank();
            }
            Event::TaskListMarker(done) => self.text(if done { "[x] " } else { "[ ] " }),
            Event::Html(text) | Event::InlineHtml(text) => self.text(&text),
            _ => {}
        }
    }
}

fn strip(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for d in chars.by_ref() {
                if d.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub(crate) fn render(source: &str, width: usize) -> Vec<String> {
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let mut renderer = Renderer::new(width);
    for event in Parser::new_ext(source, options) {
        renderer.event(event);
    }
    renderer.flush();
    while renderer.lines.last().is_some_and(String::is_empty) {
        renderer.lines.pop();
    }
    renderer.lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(source: &str, width: usize) -> Vec<String> {
        render(source, width).iter().map(|l| strip(l)).collect()
    }

    #[test]
    fn headings_paragraphs_and_lists() {
        let lines = plain(
            "# عنوان\n\nفقرة **مهمة** فيها `code`.\n\n- أول\n- تاني\n  1. فرعي\n",
            40,
        );
        assert_eq!(lines[0], "عنوان");
        assert_eq!(lines[2], "فقرة مهمة فيها code.");
        assert!(lines.contains(&"• أول".to_string()), "{lines:?}");
        assert!(lines.contains(&"  1. فرعي".to_string()), "{lines:?}");
    }

    #[test]
    fn long_paragraphs_wrap_at_words() {
        let text = "كلمة ".repeat(30);
        let lines = plain(&text, 30);
        assert!(lines.len() > 3);
        assert!(lines.iter().all(|l| width_of(l) <= 30), "{lines:?}");
        assert!(lines.iter().all(|l| !l.starts_with(' ')));
    }

    #[test]
    fn tables_get_box_borders_and_a_header_rule() {
        let lines = plain("| الاسم | العمر |\n|---|---|\n| سارة | 30 |\n", 40);
        let shift = " ".repeat(40 - 17);
        assert_eq!(lines[0], format!("{shift}┌───────┬───────┐"));
        assert_eq!(lines[1], format!("{shift}│ الاسم │ العمر │"));
        assert_eq!(lines[2], format!("{shift}├───────┼───────┤"));
        assert_eq!(lines[3], format!("{shift}│ سارة  │ 30    │"));
        assert_eq!(lines[4], format!("{shift}└───────┴───────┘"));
        let english = plain("| a | b |\n|---|---|\n| 1 | 2 |\n", 40);
        assert_eq!(english[0], "┌───┬───┐");
    }

    #[test]
    fn code_blocks_keep_their_lines_and_quotes_get_a_bar() {
        let lines = plain("```rust\nfn main() {}\n```\n\n> اقتباس\n", 30);
        assert!(
            lines.iter().any(|l| l.contains("fn main() {}")),
            "{lines:?}"
        );
        assert!(lines.contains(&"│ اقتباس".to_string()), "{lines:?}");
    }

    #[test]
    fn styles_become_sgr() {
        let raw = render("**عريض** و *مائل*", 40).join("\n");
        assert!(raw.contains("\x1b[1m"));
        assert!(raw.contains("\x1b[3m"));
    }
}
