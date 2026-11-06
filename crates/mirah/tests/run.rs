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

fn log_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("mirah-run-test-{name}-{}", std::process::id()))
}

fn mirah_log(name: &str) -> String {
    std::fs::read_to_string(log_dir(name).join("mirah").join("mirah.log"))
        .unwrap_or_else(|_| "(no mirah log)".to_string())
}

#[allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a test helper fails the test that calls it"
)]
fn run_mirah(name: &str, args: &[&str], path: Option<std::ffi::OsString>) -> (u32, String) {
    let pair = native_pty_system()
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("open pty");
    let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_mirah"));
    cmd.args(args);
    cmd.env("MIRAH_LOG", "debug");
    cmd.env("MIRAH_NO_UPDATE_CHECK", "1");
    cmd.env(STATE_ENV, log_dir(name));
    if let Some(path) = path {
        cmd.env("PATH", path);
    }
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
                mirah_log(name)
            );
        }
        std::thread::sleep(POLL);
    };
    drop(pair.master);
    let output = String::from_utf8_lossy(&collector.join().expect("collector")).into_owned();
    (status.exit_code(), output)
}

#[test]
fn restores_the_terminal_and_returns_the_child_exit_code() {
    let name = "basic";
    let (code, output) = run_mirah(name, &CHILD, None);
    assert_eq!(code, CHILD_EXIT_CODE, "mirah log:\n{}", mirah_log(name));
    assert!(
        output.contains(SHAPED_MARHABA),
        "Arabic is shaped in visual order\n--- mirah log ---\n{}",
        mirah_log(name)
    );
    if cfg!(windows) {
        assert!(
            mirah_log(name).contains(BUNDLED_CONPTY),
            "the bundled conpty is loaded\n--- mirah log ---\n{}",
            mirah_log(name)
        );
    }
    if cfg!(unix) {
        let entered = output.find(BDSM_EXPLICIT).expect("explicit bidi sent");
        let left = output.rfind(BDSM_IMPLICIT).expect("implicit bidi restored");
        assert!(entered < left);
        assert!(output.contains(ALT_SCREEN_LEAVE));
    }
    let _ = std::fs::remove_dir_all(log_dir(name));
}

#[cfg(windows)]
#[test]
fn runs_an_npm_style_cmd_shim_found_on_path() {
    let name = "shim";
    let bin = log_dir(name).join("bin");
    std::fs::create_dir_all(&bin).expect("bin dir");
    std::fs::write(bin.join("mirah-probe"), "#!/bin/sh\nexit 9\n").expect("write script");
    std::fs::write(bin.join("mirah-probe.cmd"), "@echo off\r\nexit /b 4\r\n").expect("write shim");
    let mut path = bin.clone().into_os_string();
    path.push(";");
    path.push(std::env::var_os("PATH").unwrap_or_default());
    let (code, _) = run_mirah(name, &["mirah-probe"], Some(path));
    assert_eq!(code, 4, "mirah log:\n{}", mirah_log(name));
    let _ = std::fs::remove_dir_all(log_dir(name));
}
