use mirah_term::Emulator;
use std::fmt::Write as _;
use std::num::NonZeroU16;
use std::path::PathBuf;

const ROWS: u16 = 24;
const COLS: u16 = 80;

fn sessions_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../corpus/sessions")
}

fn host_screen(frame: &str) -> String {
    let size = |n| NonZeroU16::new(n).unwrap_or(NonZeroU16::MIN);
    let mut host = vt100::Parser::new(size(ROWS), size(COLS), 0);
    host.process(frame.as_bytes());
    let screen = host.screen();
    let mut out = String::new();
    for row in 0..ROWS {
        let line: String = (0..COLS)
            .filter_map(|col| screen.cell(row, col))
            .filter(|c| !c.is_wide_continuation())
            .map(|c| {
                if c.has_contents() {
                    c.contents().to_string()
                } else {
                    " ".into()
                }
            })
            .collect();
        let _ = writeln!(out, "{}", line.trim_end());
    }
    let (row, col) = screen.cursor_position();
    let _ = writeln!(out, "@cursor {row},{col} hidden={}", screen.hide_cursor());
    out
}

#[test]
fn recorded_sessions_render_to_their_expected_screens() {
    let mut sessions: Vec<PathBuf> = std::fs::read_dir(sessions_dir())
        .expect("corpus/sessions exists")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "vt"))
        .collect();
    sessions.sort();
    assert!(!sessions.is_empty());
    for vt in sessions {
        let mut emulator = Emulator::new(ROWS, COLS).expect("valid size");
        emulator.feed(&std::fs::read(&vt).expect("session is readable"));
        let got = host_screen(&emulator.frame());
        let expected =
            std::fs::read_to_string(vt.with_extension("expected")).expect("expected screen exists");
        assert_eq!(got, expected, "{} renders differently", vt.display());
    }
}
