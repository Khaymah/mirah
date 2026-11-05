use mirah_term::sequences::{ALT_SCREEN_LEAVE, BDSM_EXPLICIT, BDSM_IMPLICIT};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::Read;

const CHILD_EXIT_CODE: u32 = 3;

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
    cmd.args(["sh", "-c", "printf 'مرحبا\\n'; exit 3"]);
    let mut child = pair.slave.spawn_command(cmd).expect("spawn mirah");
    drop(pair.slave);

    let mut reader = pair.master.try_clone_reader().expect("reader");
    let collector = std::thread::spawn(move || {
        let mut out = Vec::new();
        let _ = reader.read_to_end(&mut out);
        out
    });
    let status = child.wait().expect("wait for mirah");
    drop(pair.master);
    let output = String::from_utf8_lossy(&collector.join().expect("collector")).into_owned();

    assert_eq!(status.exit_code(), CHILD_EXIT_CODE);
    let entered = output.find(BDSM_EXPLICIT).expect("explicit bidi sent");
    let left = output.rfind(BDSM_IMPLICIT).expect("implicit bidi restored");
    assert!(entered < left);
    assert!(output.contains(ALT_SCREEN_LEAVE));
    assert!(
        output.contains("\u{FEA3}\u{FEAE}\u{FEE3}"),
        "Arabic is shaped in visual order"
    );
}
