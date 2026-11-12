#![allow(
    unsafe_code,
    reason = "LoadLibraryW has no safe wrapper; the module must stay loaded for the whole process"
)]

use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use windows_sys::Win32::System::LibraryLoader::LoadLibraryW;

const MODE_ENV: &str = "MIRAH_CONPTY";
const SYSTEM_MODE: &str = "system";
const DATA_ENV: &str = "LOCALAPPDATA";
const VERSION: &str = "1.25.260930003";
const DLL_NAME: &str = "conpty.dll";
const HOST_NAME: &str = "OpenConsole.exe";
const LICENSE_NAME: &str = "LICENSE.txt";
const LICENSE: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/vendor/conpty/LICENSE"
));

#[cfg(target_arch = "x86_64")]
const ARCH: &str = "x64";
#[cfg(target_arch = "x86_64")]
const DLL: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/vendor/conpty/x64/conpty.dll"
));
#[cfg(target_arch = "x86_64")]
const HOST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/vendor/conpty/x64/OpenConsole.exe"
));

#[cfg(target_arch = "aarch64")]
const ARCH: &str = "arm64";
#[cfg(target_arch = "aarch64")]
const DLL: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/vendor/conpty/arm64/conpty.dll"
));
#[cfg(target_arch = "aarch64")]
const HOST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/vendor/conpty/arm64/OpenConsole.exe"
));

pub(crate) fn install() {
    if std::env::var(MODE_ENV).is_ok_and(|v| v == SYSTEM_MODE) {
        tracing::info!("using the conpty that ships with Windows");
        return;
    }
    match extract().and_then(|dir| load(&dir.join(DLL_NAME))) {
        Ok(()) => tracing::info!(version = VERSION, "using the bundled conpty"),
        Err(error) => {
            tracing::warn!(%error, "bundled conpty unavailable, using the one in Windows")
        }
    }
}

fn extract() -> std::io::Result<PathBuf> {
    let base = std::env::var_os(DATA_ENV)
        .map(PathBuf::from)
        .ok_or_else(|| std::io::Error::other("LOCALAPPDATA is not set"))?;
    let dir = base
        .join("mirah")
        .join("conpty")
        .join(format!("{VERSION}-{ARCH}"));
    std::fs::create_dir_all(&dir)?;
    for (name, bytes) in [(DLL_NAME, DLL), (HOST_NAME, HOST), (LICENSE_NAME, LICENSE)] {
        ensure_file(&dir.join(name), bytes)?;
    }
    remove_older(&dir);
    Ok(dir)
}

fn version_parts(version: &str) -> Option<Vec<u64>> {
    version.split('.').map(|p| p.parse().ok()).collect()
}

fn is_older(version: &str, than: &str) -> bool {
    match (version_parts(version), version_parts(than)) {
        (Some(a), Some(b)) => a < b,
        _ => false,
    }
}

fn remove_older(current: &Path) {
    let Some(parent) = current.parent() else {
        return;
    };
    let Ok(entries) = std::fs::read_dir(parent) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let older = path != current
            && entry.file_type().is_ok_and(|t| t.is_dir())
            && name
                .strip_suffix(&format!("-{ARCH}"))
                .is_some_and(|version| is_older(version, VERSION));
        if older && std::fs::remove_dir_all(&path).is_ok() {
            tracing::debug!(path = %path.display(), "removed an older bundled conpty");
        }
    }
}

fn ensure_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if std::fs::read(path).is_ok_and(|existing| existing == bytes) {
        return Ok(());
    }
    let staging = path.with_extension(format!("{}.tmp", std::process::id()));
    std::fs::write(&staging, bytes)?;
    std::fs::rename(&staging, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&staging);
    })
}

fn load(dll: &Path) -> std::io::Result<()> {
    let wide: Vec<u16> = dll
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let module = unsafe { LoadLibraryW(wide.as_ptr()) };
    if module.is_null() {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::is_older;

    #[test]
    fn only_strictly_older_bundles_count_as_old() {
        assert!(is_older("1.24.9", "1.25.260930003"));
        assert!(is_older("1.25.2", "1.25.10"));
        assert!(!is_older("1.25.260930003", "1.25.260930003"));
        assert!(!is_older("1.26.0", "1.25.260930003"));
        assert!(!is_older("custom", "1.25.260930003"));
    }
}
