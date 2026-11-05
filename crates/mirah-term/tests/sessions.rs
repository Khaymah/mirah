use mirah_term::{Align, Emulator, InputOrder, RenderOptions, Shaping};
use std::fmt::Write as _;
use std::num::NonZeroU16;
use std::path::Path;
use std::path::PathBuf;

const ROWS: u16 = 24;
const COLS: u16 = 80;
const UPDATE_ENV: &str = "MIRAH_UPDATE_SNAPSHOTS";
const VISUAL_SUFFIX: &str = "-visual";

fn options_for(vt: &Path) -> RenderOptions {
    let visual = vt
        .file_stem()
        .is_some_and(|s| s.to_string_lossy().ends_with(VISUAL_SUFFIX));
    if visual {
        RenderOptions {
            shaping: Shaping::Plain,
            align: Align::Prose,
            input: InputOrder::Visual,
        }
    } else {
        RenderOptions::default()
    }
}

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
                let text = if c.has_contents() { c.contents() } else { " " };
                if c.inverse() {
                    format!("[{text}]")
                } else {
                    text.to_string()
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
        let mut emulator =
            Emulator::with_options(ROWS, COLS, options_for(&vt)).expect("valid size");
        emulator.feed(&std::fs::read(&vt).expect("session is readable"));
        let got = host_screen(&emulator.frame());
        let expected_path = vt.with_extension("expected");
        if std::env::var_os(UPDATE_ENV).is_some() {
            std::fs::write(&expected_path, &got).expect("write expected screen");
            continue;
        }
        let expected = std::fs::read_to_string(&expected_path).expect("expected screen exists");
        assert_eq!(got, expected, "{} renders differently", vt.display());
    }
}
