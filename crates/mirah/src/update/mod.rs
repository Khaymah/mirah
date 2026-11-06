mod cache;
mod install;
mod prompt;
mod source;
mod version;

use cache::Cache;
use semver::Version;
use std::io::IsTerminal;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CURRENT: &str = env!("CARGO_PKG_VERSION");
const CACHE_FILE: &str = "update-check";
const NO_CHECK_ENV: &str = "MIRAH_NO_UPDATE_CHECK";
const STARTUP_TIMEOUT: Duration = Duration::from_secs(2);
const COMMAND_TIMEOUT: Duration = Duration::from_secs(20);
const OK: u32 = 0;
const FAILED: u32 = 1;

fn current() -> Version {
    Version::parse(CURRENT).unwrap_or_else(|_| Version::new(0, 0, 0))
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn cache_path() -> Option<PathBuf> {
    crate::paths::state_dir().map(|dir| dir.join(CACHE_FILE))
}

fn latest(timeout: Duration) -> anyhow::Result<Option<Version>> {
    let json = source::fetch_text(&source::releases_url(), timeout)?;
    Ok(version::newest(&json))
}

fn tag(v: &Version) -> String {
    format!("v{v}")
}

#[allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "mirah update is a plain command that reports to the terminal"
)]
pub(crate) fn command() -> u32 {
    let current = current();
    let latest = match latest(COMMAND_TIMEOUT) {
        Ok(latest) => latest,
        Err(err) => {
            eprintln!("mirah: {err:#}");
            eprintln!(
                "mirah: download the new version from https://github.com/Khaymah/mirah/releases"
            );
            return FAILED;
        }
    };
    if let Some(path) = cache_path() {
        let mut cache = Cache::load(&path);
        cache.checked_at = now();
        cache.latest.clone_from(&latest);
        cache.save(&path);
    }
    match latest.filter(|l| *l > current) {
        None => {
            println!("mirah {current} is the newest version");
            OK
        }
        Some(latest) => install_reporting(&current, &latest),
    }
}

#[allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "installing reports before mirah takes over the terminal"
)]
fn install_reporting(current: &Version, latest: &Version) -> u32 {
    println!("mirah: updating {current} to {latest}");
    match install::install(&tag(latest)) {
        Ok(()) => {
            println!("mirah: installed {latest}, it starts next time you run mirah");
            tracing::info!(%latest, "update installed");
            OK
        }
        Err(err) => {
            eprintln!("mirah: {err:#}");
            tracing::warn!(error = format!("{err:#}"), "update failed");
            FAILED
        }
    }
}

fn checking_allowed() -> bool {
    std::env::var_os(NO_CHECK_ENV).is_none()
        && std::io::stdin().is_terminal()
        && std::io::stdout().is_terminal()
}

pub(crate) fn startup() {
    install::remove_leftover();
    if !checking_allowed() {
        return;
    }
    let Some(path) = cache_path() else {
        return;
    };
    let mut cache = Cache::load(&path);
    let now = now();
    if !cache.fresh(now) {
        match latest(STARTUP_TIMEOUT) {
            Ok(latest) => cache.latest = latest,
            Err(err) => tracing::debug!(error = format!("{err:#}"), "update check failed"),
        }
        cache.checked_at = now;
        cache.save(&path);
    }
    let current = current();
    let Some(offer) = cache.offer(&current, now) else {
        return;
    };
    let question = format!("mirah {offer} is available (you have {current}).");
    match prompt::ask(&question) {
        prompt::Choice::Install => {
            install_reporting(&current, &offer);
        }
        prompt::Choice::Later => {
            cache.dismissed = Some((offer, now));
            cache.save(&path);
        }
    }
}
