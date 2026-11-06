use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, read};
use std::io::Write;

const INSTALL: &str = "Install";
const LATER: &str = "Later";
const INVERSE: &str = "\x1b[7m";
const RESET: &str = "\x1b[0m";
const CLEAR_LINE: &str = "\r\x1b[2K";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Choice {
    Install,
    Later,
}

fn button(label: &str, selected: bool) -> String {
    if selected {
        format!("{INVERSE} {label} {RESET}")
    } else {
        format!(" {label} ")
    }
}

pub(crate) fn line(question: &str, choice: Choice) -> String {
    format!(
        "{CLEAR_LINE}{question}  {}  {}",
        button(INSTALL, choice == Choice::Install),
        button(LATER, choice == Choice::Later)
    )
}

pub(crate) fn next(
    choice: Choice,
    key: KeyCode,
    modifiers: KeyModifiers,
) -> Result<Choice, Choice> {
    let toggled = match choice {
        Choice::Install => Choice::Later,
        Choice::Later => Choice::Install,
    };
    match key {
        KeyCode::Left | KeyCode::Right | KeyCode::Tab | KeyCode::BackTab => Ok(toggled),
        KeyCode::Enter => Err(choice),
        KeyCode::Esc => Err(Choice::Later),
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => Err(Choice::Later),
        _ => Ok(choice),
    }
}

#[allow(
    clippy::print_stdout,
    reason = "the prompt is drawn before mirah takes over the terminal"
)]
fn draw(text: &str) {
    print!("{text}");
    let _ = std::io::stdout().flush();
}

pub(crate) fn ask(question: &str) -> Choice {
    if crossterm::terminal::enable_raw_mode().is_err() {
        return Choice::Later;
    }
    let mut choice = Choice::Install;
    draw(&line(question, choice));
    let result = loop {
        match read() {
            Ok(Event::Key(key)) if key.kind != KeyEventKind::Release => {
                match next(choice, key.code, key.modifiers) {
                    Ok(moved) => choice = moved,
                    Err(done) => break done,
                }
                draw(&line(question, choice));
            }
            Ok(_) => {}
            Err(_) => break Choice::Later,
        }
    };
    let _ = crossterm::terminal::disable_raw_mode();
    draw("\r\n");
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_is_selected_first_and_arrows_toggle() {
        assert!(line("q", Choice::Install).contains(&format!("{INVERSE} Install ")));
        assert_eq!(
            next(Choice::Install, KeyCode::Right, KeyModifiers::NONE),
            Ok(Choice::Later)
        );
        assert_eq!(
            next(Choice::Later, KeyCode::Tab, KeyModifiers::NONE),
            Ok(Choice::Install)
        );
        assert_eq!(
            next(Choice::Later, KeyCode::Char('x'), KeyModifiers::NONE),
            Ok(Choice::Later)
        );
    }

    #[test]
    fn enter_confirms_and_escape_or_ctrl_c_mean_later() {
        assert_eq!(
            next(Choice::Install, KeyCode::Enter, KeyModifiers::NONE),
            Err(Choice::Install)
        );
        assert_eq!(
            next(Choice::Install, KeyCode::Esc, KeyModifiers::NONE),
            Err(Choice::Later)
        );
        assert_eq!(
            next(Choice::Install, KeyCode::Char('c'), KeyModifiers::CONTROL),
            Err(Choice::Later)
        );
    }
}
