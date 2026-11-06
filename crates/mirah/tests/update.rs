#![allow(
    clippy::expect_used,
    reason = "test helpers fail the test that calls them"
)]

use std::path::{Path, PathBuf};
use std::process::Command;

const NEW_BINARY: &[u8] = b"the new mirah";
const BUSY_RETRIES: u32 = 20;
const BUSY_WAIT: std::time::Duration = std::time::Duration::from_millis(50);

#[cfg(windows)]
const ARTIFACT_SUFFIX: &str = ".zip";
#[cfg(not(windows))]
const ARTIFACT_SUFFIX: &str = ".tar.xz";

#[cfg(windows)]
const STATE_ENV: &str = "LOCALAPPDATA";
#[cfg(not(windows))]
const STATE_ENV: &str = "XDG_STATE_HOME";

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mirah-update-test-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

fn file_url(path: &Path) -> String {
    let text = path.display().to_string().replace('\\', "/");
    if text.starts_with('/') {
        format!("file://{text}")
    } else {
        format!("file:///{text}")
    }
}

fn artifact() -> String {
    let arch = std::env::consts::ARCH;
    let target = match std::env::consts::OS {
        "windows" => format!("{arch}-pc-windows-msvc"),
        "macos" => format!("{arch}-apple-darwin"),
        _ => format!("{arch}-unknown-linux-gnu"),
    };
    format!("mirah-{target}{ARTIFACT_SUFFIX}")
}

#[cfg(windows)]
fn tar() -> Command {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    Command::new(Path::new(&root).join("System32").join("tar.exe"))
}

#[cfg(not(windows))]
fn tar() -> Command {
    Command::new("tar")
}

fn sha256_of(path: &Path) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(std::fs::read(path).expect("read archive"))
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn publish(root: &Path, tag: &str, tamper: bool) {
    let stage = root.join("stage");
    std::fs::create_dir_all(&stage).expect("stage");
    let binary = format!("mirah{}", std::env::consts::EXE_SUFFIX);
    std::fs::write(stage.join(&binary), NEW_BINARY).expect("write binary");
    let release = root.join("download").join(tag);
    std::fs::create_dir_all(&release).expect("release dir");
    let archive = release.join(artifact());
    let flag = if cfg!(windows) { "-acf" } else { "-cJf" };
    let status = tar()
        .arg(flag)
        .arg(&archive)
        .arg("-C")
        .arg(&stage)
        .arg(&binary)
        .status()
        .expect("run tar");
    assert!(status.success(), "tar created the archive");
    let hash = if tamper {
        "0".repeat(64)
    } else {
        sha256_of(&archive)
    };
    std::fs::write(
        release.join(format!("{}.sha256", artifact())),
        format!("{hash}  {}\n", artifact()),
    )
    .expect("write checksum");
    std::fs::write(
        root.join("releases.json"),
        format!(r#"[{{"tag_name": "{tag}", "prerelease": true}}]"#),
    )
    .expect("write release list");
}

fn installed_copy(root: &Path) -> PathBuf {
    let bin = root.join("bin");
    std::fs::create_dir_all(&bin).expect("bin dir");
    let copy = bin.join(format!("mirah{}", std::env::consts::EXE_SUFFIX));
    std::fs::copy(env!("CARGO_BIN_EXE_mirah"), &copy).expect("copy mirah");
    copy
}

fn update(root: &Path, mirah: &Path) -> std::process::Output {
    let mut tries = 0;
    loop {
        match run_update(root, mirah) {
            Err(err)
                if err.kind() == std::io::ErrorKind::ExecutableFileBusy && tries < BUSY_RETRIES =>
            {
                tries += 1;
                std::thread::sleep(BUSY_WAIT);
            }
            result => return result.expect("run mirah update"),
        }
    }
}

fn run_update(root: &Path, mirah: &Path) -> std::io::Result<std::process::Output> {
    Command::new(mirah)
        .arg("update")
        .env(
            "MIRAH_UPDATE_API_URL",
            file_url(&root.join("releases.json")),
        )
        .env(
            "MIRAH_UPDATE_DOWNLOAD_URL",
            file_url(&root.join("download")),
        )
        .env(STATE_ENV, root.join("state"))
        .output()
}

#[test]
fn installs_a_newer_release_after_checking_its_sha256() {
    let root = scratch("install");
    publish(&root, "v99.0.0", false);
    let mirah = installed_copy(&root);
    let out = update(&root, &mirah);
    assert!(
        out.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(std::fs::read(&mirah).expect("read new mirah"), NEW_BINARY);
    assert!(String::from_utf8_lossy(&out.stdout).contains("99.0.0"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn refuses_a_download_with_the_wrong_checksum() {
    let root = scratch("tamper");
    publish(&root, "v99.0.0", true);
    let mirah = installed_copy(&root);
    let before = std::fs::read(&mirah).expect("read mirah");
    let out = update(&root, &mirah);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("SHA-256"));
    assert_eq!(std::fs::read(&mirah).expect("read mirah"), before);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn says_so_when_already_on_the_newest_version() {
    let root = scratch("newest");
    publish(&root, "v0.0.1", false);
    let mirah = installed_copy(&root);
    let out = update(&root, &mirah);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("is the newest version"));
    let _ = std::fs::remove_dir_all(root);
}
