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

fn test_override(name: &str) -> Option<String> {
    if cfg!(debug_assertions) {
        std::env::var(name).ok()
    } else {
        None
    }
}

pub(crate) fn releases_url() -> String {
    test_override(API_ENV)
        .unwrap_or_else(|| format!("https://api.github.com/repos/{REPO}/releases?per_page=30"))
}

pub(crate) fn download_url(tag: &str, file: &str) -> String {
    match test_override(DOWNLOAD_ENV) {
        Some(base) => format!("{}/{tag}/{file}", base.trim_end_matches('/')),
        None => format!("https://github.com/{REPO}/releases/download/{tag}/{file}"),
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

fn protocols(overridden: bool) -> &'static str {
    if overridden { "=https,file" } else { "=https" }
}

fn curl(timeout: Duration) -> Command {
    let mut cmd = system_tool("curl");
    let overridden = test_override(API_ENV).is_some() || test_override(DOWNLOAD_ENV).is_some();
    cmd.stdin(std::process::Stdio::null())
        .arg("-q")
        .args(["--fail", "--silent", "--show-error", "--location"])
        .args(["--proto", protocols(overridden), "--proto-redir", "=https"])
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
    fn only_https_is_allowed_unless_a_test_points_somewhere_else() {
        assert_eq!(protocols(false), "=https");
        assert_eq!(protocols(true), "=https,file");
    }

    #[test]
    fn curl_ignores_the_user_curlrc() {
        let cmd = curl(Duration::from_secs(1));
        assert_eq!(cmd.get_args().next().and_then(|a| a.to_str()), Some("-q"));
    }

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
