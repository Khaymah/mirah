#![cfg(unix)]
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "a test fails loudly when its setup breaks"
)]

use portable_pty::{Child, CommandBuilder, PtySize, native_pty_system};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const CURSOR_QUERY: &[u8] = b"\x1b[6n";
const CURSOR_REPLY: &[u8] = b"\x1b[1;1R";
const READ_BUFFER: usize = 4096;
const STEP: Duration = Duration::from_millis(50);
const SETTLE: Duration = Duration::from_millis(700);
const EXIT_DEADLINE: Duration = Duration::from_secs(5);
const STAY_OPEN: Duration = Duration::from_secs(3);
const TOGGLE: &[u8] = b"\x1d";
const ESC: &[u8] = b"\x1b";
const SIGNAL_EXIT_BASE: u32 = 128;
const SIGTERM: i32 = 15;
const READY_DEADLINE: Duration = Duration::from_secs(20);
const ALT_SCREEN_ENTER: &[u8] = b"\x1b[?1049h";
const ALT_SCREEN_LEAVE: &[u8] = b"\x1b[?1049l";

struct Mirah {
    child: Box<dyn Child + Send + Sync>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    output: Arc<Mutex<Vec<u8>>>,
    home: PathBuf,
    marker: Option<String>,
    _master: Box<dyn portable_pty::MasterPty + Send>,
}

fn home(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mirah-lifecycle-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("bin")).expect("bin dir");
    dir
}

fn program(home: &Path, name: &str, script: &str) {
    use std::os::unix::fs::PermissionsExt;
    let path = home.join("bin").join(name);
    std::fs::write(&path, format!("#!/bin/sh\n{script}\n")).expect("write program");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("chmod");
}

impl Mirah {
    fn start(home: PathBuf, args: &[&str]) -> Self {
        let pair = native_pty_system()
            .openpty(PtySize {
                rows: 30,
                cols: 100,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("open pty");
        let mut cmd = CommandBuilder::new(env!("CARGO_BIN_EXE_mirah"));
        cmd.args(args);
        cmd.cwd(&home);
        let path = format!("{}:/usr/bin:/bin", home.join("bin").display());
        cmd.env("PATH", path);
        cmd.env("HOME", &home);
        cmd.env("XDG_STATE_HOME", home.join("state"));
        cmd.env("XDG_CONFIG_HOME", home.join("config"));
        cmd.env("MIRAH_NO_UPDATE_CHECK", "1");
        cmd.env("MIRAH_SHAPING", "plain");
        cmd.env("TERM", "xterm-256color");
        let child = pair.slave.spawn_command(cmd).expect("spawn mirah");
        drop(pair.slave);
        let mut reader = pair.master.try_clone_reader().expect("reader");
        let writer = Arc::new(Mutex::new(pair.master.take_writer().expect("writer")));
        let output = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&output);
        let replies = Arc::clone(&writer);
        let master = pair.master;
        std::thread::spawn(move || {
            let mut buf = [0u8; READ_BUFFER];
            while let Ok(n) = reader.read(&mut buf) {
                if n == 0 {
                    break;
                }
                if buf[..n]
                    .windows(CURSOR_QUERY.len())
                    .any(|w| w == CURSOR_QUERY)
                    && let Ok(mut w) = replies.lock()
                {
                    let _ = w.write_all(CURSOR_REPLY);
                    let _ = w.flush();
                }
                if let Ok(mut out) = sink.lock() {
                    out.extend_from_slice(&buf[..n]);
                }
            }
        });
        let mirah = Self {
            child,
            writer,
            output,
            home,
            marker: None,
            _master: master,
        };
        assert!(
            mirah.saw(ALT_SCREEN_ENTER, 0, READY_DEADLINE),
            "mirah never took over the screen"
        );
        mirah.wait(SETTLE);
        mirah
    }

    fn saw(&self, needle: &[u8], from: usize, deadline: Duration) -> bool {
        let started = Instant::now();
        loop {
            let output = self.output();
            let tail = &output[from.min(output.len())..];
            if tail.windows(needle.len()).any(|w| w == needle) {
                return true;
            }
            if started.elapsed() >= deadline {
                return false;
            }
            std::thread::sleep(STEP);
        }
    }

    fn send(&mut self, bytes: &[u8]) {
        if let Ok(mut w) = self.writer.lock() {
            let _ = w.write_all(bytes);
            let _ = w.flush();
        }
        self.wait(SETTLE);
    }

    fn wait(&self, time: Duration) {
        std::thread::sleep(time);
    }

    fn exit(&mut self, deadline: Duration) -> Option<u32> {
        let started = Instant::now();
        while started.elapsed() < deadline {
            if let Ok(Some(status)) = self.child.try_wait() {
                return Some(status.exit_code());
            }
            std::thread::sleep(STEP);
        }
        None
    }

    fn output(&self) -> Vec<u8> {
        self.output.lock().map(|o| o.clone()).unwrap_or_default()
    }

    fn signal(&self, signal: i32) {
        let pid = self.child.process_id().expect("mirah pid");
        let status = std::process::Command::new("kill")
            .arg(format!("-{signal}"))
            .arg(pid.to_string())
            .status()
            .expect("run kill");
        assert!(status.success());
    }
}

impl Drop for Mirah {
    fn drop(&mut self) {
        let _ = self.child.kill();
        if let Some(marker) = &self.marker {
            let _ = std::process::Command::new("pkill")
                .args(["-9", "-f", marker])
                .status();
        }
        let _ = std::fs::remove_dir_all(&self.home);
    }
}

fn still_running(marker: &str) -> bool {
    std::process::Command::new("pgrep")
        .args(["-f", marker])
        .output()
        .is_ok_and(|o| o.status.success())
}

#[test]
fn quitting_ends_a_program_whose_child_ignores_hangups_and_keeps_printing() {
    let home = home("stubborn");
    let marker = format!("mirah-stubborn-{}", std::process::id());
    program(
        &home,
        "stubborn",
        &format!(
            "sh -c 'trap \"\" HUP TERM; while :; do printf \"\\rframe\"; sleep 0.03; done # {marker}' &\nwait"
        ),
    );
    let mut mirah = Mirah::start(home, &["stubborn"]);
    mirah.marker = Some(marker.clone());
    assert!(still_running(&marker));
    mirah.send(TOGGLE);
    mirah.send(b"q");
    assert!(mirah.exit(EXIT_DEADLINE).is_some(), "mirah did not exit");
    std::thread::sleep(SETTLE);
    assert!(!still_running(&marker), "the grandchild outlived mirah");
}

#[test]
fn a_child_left_behind_by_a_program_that_exited_ends_with_its_tab() {
    let home = home("orphan");
    let marker = format!("mirah-orphan-{}", std::process::id());
    program(
        &home,
        "leaver",
        &format!(
            "sh -c 'trap \"\" HUP TERM; while :; do printf \"\\rframe\"; sleep 0.03; done # {marker}' &\nsleep 0.5\nexit 0"
        ),
    );
    let mut mirah = Mirah::start(home, &["leaver"]);
    mirah.marker = Some(marker.clone());
    assert!(mirah.exit(EXIT_DEADLINE).is_some(), "the tab never closed");
    std::thread::sleep(SETTLE);
    assert!(
        !still_running(&marker),
        "the left-behind child outlived its tab"
    );
}

#[test]
fn ctrl_bracket_in_the_picker_opens_the_bar_and_esc_still_leaves() {
    let home = home("picker");
    let mut mirah = Mirah::start(home, &[]);
    mirah.send(TOGGLE);
    assert_eq!(mirah.exit(STAY_OPEN), None, "Ctrl+] quit mirah");
    mirah.send(ESC);
    mirah.send(ESC);
    assert_eq!(mirah.exit(EXIT_DEADLINE), Some(0));
}

#[test]
fn the_picker_stays_open_when_the_last_tab_ends_meanwhile() {
    let home = home("brief");
    program(&home, "brief", "echo ready; sleep 3; exit 0");
    let mut mirah = Mirah::start(home, &["brief"]);
    mirah.send(TOGGLE);
    mirah.send(b"t");
    assert_eq!(mirah.exit(STAY_OPEN), None, "mirah left while choosing");
    mirah.send(ESC);
    assert_eq!(
        mirah.exit(EXIT_DEADLINE),
        Some(0),
        "Esc leaves once nothing runs"
    );
}

#[test]
fn a_terminate_signal_restores_the_terminal() {
    let home = home("signal");
    program(&home, "idle", "while :; do sleep 1; done");
    let mut mirah = Mirah::start(home, &["idle"]);
    let before = mirah.output().len();
    mirah.signal(SIGTERM);
    assert_eq!(mirah.exit(EXIT_DEADLINE), Some(SIGNAL_EXIT_BASE + 15));
    assert!(
        mirah.saw(ALT_SCREEN_LEAVE, before, EXIT_DEADLINE),
        "the alternate screen was not left"
    );
}
