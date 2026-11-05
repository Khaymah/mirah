use mirah_term::sequences::{ALT_SCREEN_LEAVE, BDSM_EXPLICIT, BDSM_IMPLICIT};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

const CHILD_EXIT_CODE: u32 = 3;
const SHAPED_MARHABA: &str = "\u{FEA3}\u{FEAE}\u{FEE3}";
const MIRAH_DEADLINE: Duration = Duration::from_secs(60);
const POLL: Duration = Duration::from_millis(50);
const CURSOR_QUERY: &[u8] = b"\x1b[6n";
const CURSOR_REPLY: &[u8] = b"\x1b[1;1R";
const READ_BUFFER: usize = 4096;
const BUNDLED_CONPTY: &str = "using the bundled conpty";

#[cfg(windows)]
const CHILD: [&str; 3] = ["cmd", "/c", "echo مرحبا& exit 3"];
#[cfg(not(windows))]
const CHILD: [&str; 3] = ["sh", "-c", "printf 'مرحبا\\n'; exit 3"];

#[cfg(windows)]
const STATE_ENV: &str = "LOCALAPPDATA";
#[cfg(not(windows))]
const STATE_ENV: &str = "XDG_STATE_HOME";

fn log_dir() -> PathBuf {
    std::env::temp_dir().join(format!("mirah-run-test-{}", std::process::id()))
}

fn mirah_log() -> String {
    std::fs::read_to_string(log_dir().join("mirah").join("mirah.log"))
        .unwrap_or_else(|_| "(no mirah log)".to_string())
}

#[test]
fn restores_the_terminal_and_returns_the_child_exit_code() {
    let pair = native_pty_system()
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("open pty");
    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_mirah"));
    cmd.args(CHILD);
    cmd.env("MIRAH_LOG", "debug");
    cmd.env(STATE_ENV, log_dir());
    let mut child = pair.slave.spawn_command(cmd).expect("spawn mirah");
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader().expect("reader");
    let mut writer = pair.master.take_writer().expect("writer");
    let collector = std::thread::spawn(move || {
        let mut out = Vec::new();
        let mut buf = [0u8; READ_BUFFER];
        while let Ok(n) = reader.read(&mut buf) {
            if n == 0 {
                break;
            }
            out.extend_from_slice(&buf[..n]);
            if buf[..n]
                .windows(CURSOR_QUERY.len())
                .any(|w| w == CURSOR_QUERY)
            {
                let _ = writer.write_all(CURSOR_REPLY).and_then(|()| writer.flush());
            }
        }
        out
    });

    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().expect("poll mirah") {
            break status;
        }
        if started.elapsed() > MIRAH_DEADLINE {
            let _ = child.kill();
            drop(pair.master);
            let output =
                String::from_utf8_lossy(&collector.join().expect("collector")).into_owned();
            let tail: String = output
                .chars()
                .rev()
                .take(2000)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            panic!(
                "mirah did not exit within {MIRAH_DEADLINE:?}\n--- mirah log ---\n{}\n--- output tail ---\n{tail:?}",
                mirah_log()
            );
        }
        std::thread::sleep(POLL);
    };
    drop(pair.master);
    let output = String::from_utf8_lossy(&collector.join().expect("collector")).into_owned();

    assert_eq!(
        status.exit_code(),
        CHILD_EXIT_CODE,
        "mirah log:\n{}",
        mirah_log()
    );
    assert!(
        output.contains(SHAPED_MARHABA),
        "Arabic is shaped in visual order\n--- mirah log ---\n{}",
        mirah_log()
    );
    if cfg!(windows) {
        assert!(
            mirah_log().contains(BUNDLED_CONPTY),
            "the bundled conpty is loaded\n--- mirah log ---\n{}",
            mirah_log()
        );
    }
    if cfg!(unix) {
        let entered = output.find(BDSM_EXPLICIT).expect("explicit bidi sent");
        let left = output.rfind(BDSM_IMPLICIT).expect("implicit bidi restored");
        assert!(entered < left);
        assert!(output.contains(ALT_SCREEN_LEAVE));
    }
    let _ = std::fs::remove_dir_all(log_dir());
}
