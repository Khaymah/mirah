use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

const REPO: &str = "Khaymah/mirah";
const API_ENV: &str = "MIRAH_UPDATE_API_URL";
const DOWNLOAD_ENV: &str = "MIRAH_UPDATE_DOWNLOAD_URL";

#[cfg(all(target_arch = "x86_64", target_os = "windows"))]
const ARTIFACT: Option<&str> = Some("mirah-x86_64-pc-windows-msvc.zip");
#[cfg(all(target_arch = "aarch64", target_os = "windows"))]
const ARTIFACT: Option<&str> = Some("mirah-aarch64-pc-windows-msvc.zip");
#[cfg(all(target_arch = "x86_64", target_os = "macos"))]
const ARTIFACT: Option<&str> = Some("mirah-x86_64-apple-darwin.tar.xz");
#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
const ARTIFACT: Option<&str> = Some("mirah-aarch64-apple-darwin.tar.xz");
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
const ARTIFACT: Option<&str> = Some("mirah-x86_64-unknown-linux-gnu.tar.xz");
#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
const ARTIFACT: Option<&str> = Some("mirah-aarch64-unknown-linux-gnu.tar.xz");
#[cfg(not(any(
    all(
        any(target_arch = "x86_64", target_arch = "aarch64"),
        target_os = "windows"
    ),
    all(
        any(target_arch = "x86_64", target_arch = "aarch64"),
        target_os = "macos"
    ),
    all(
        any(target_arch = "x86_64", target_arch = "aarch64"),
        target_os = "linux"
    ),
)))]
const ARTIFACT: Option<&str> = None;

pub(crate) fn artifact() -> Option<&'static str> {
    ARTIFACT
}

pub(crate) fn releases_url() -> String {
    std::env::var(API_ENV)
        .unwrap_or_else(|_| format!("https://api.github.com/repos/{REPO}/releases?per_page=30"))
}

pub(crate) fn download_url(tag: &str, file: &str) -> String {
    match std::env::var(DOWNLOAD_ENV) {
        Ok(base) => format!("{}/{tag}/{file}", base.trim_end_matches('/')),
        Err(_) => format!("https://github.com/{REPO}/releases/download/{tag}/{file}"),
    }
}

#[cfg(windows)]
pub(crate) fn system_tool(name: &str) -> Command {
    let system = std::env::var_os("SystemRoot").map(std::path::PathBuf::from);
    match system.map(|root| root.join("System32").join(format!("{name}.exe"))) {
        Some(path) if path.is_file() => Command::new(path),
        _ => Command::new(name),
    }
}

#[cfg(not(windows))]
pub(crate) fn system_tool(name: &str) -> Command {
    Command::new(name)
}

fn protocols() -> &'static str {
    let overridden =
        std::env::var_os(API_ENV).is_some() || std::env::var_os(DOWNLOAD_ENV).is_some();
    if overridden { "=https,file" } else { "=https" }
}

fn curl(timeout: Duration) -> Command {
    let mut cmd = system_tool("curl");
    cmd.stdin(std::process::Stdio::null())
        .args(["--fail", "--silent", "--show-error", "--location"])
        .args(["--proto", protocols(), "--proto-redir", "=https"])
        .args(["--max-time", &timeout.as_secs().max(1).to_string()]);
    cmd
}

fn run(mut cmd: Command, what: &str) -> Result<Vec<u8>> {
    let output = cmd
        .output()
        .with_context(|| format!("{what}: could not start curl"))?;
    if !output.status.success() {
        let reason = String::from_utf8_lossy(&output.stderr).trim().to_string();
        bail!("{what}: {reason}");
    }
    Ok(output.stdout)
}

pub(crate) fn fetch_text(url: &str, timeout: Duration) -> Result<String> {
    let mut cmd = curl(timeout);
    cmd.args(["--header", "Accept: application/vnd.github+json", url]);
    let body = run(cmd, "checking for a new version")?;
    Ok(String::from_utf8_lossy(&body).into_owned())
}

pub(crate) fn download(url: &str, dest: &Path, timeout: Duration) -> Result<()> {
    let mut cmd = curl(timeout);
    cmd.arg("--output").arg(dest).arg(url);
    run(cmd, "downloading the new version").map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn this_platform_has_a_release_artifact() {
        assert!(artifact().is_some_and(|a| a.starts_with("mirah-")));
    }

    #[test]
    fn download_urls_point_at_the_release_tag() {
        if std::env::var_os(DOWNLOAD_ENV).is_none() {
            assert_eq!(
                download_url("v0.2.0", "mirah.zip"),
                "https://github.com/Khaymah/mirah/releases/download/v0.2.0/mirah.zip"
            );
        }
    }
}
