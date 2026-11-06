use std::path::PathBuf;

const APP_DIR: &str = "mirah";

#[cfg(windows)]
pub(crate) fn state_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|dir| PathBuf::from(dir).join(APP_DIR))
}

#[cfg(not(windows))]
pub(crate) fn state_dir() -> Option<PathBuf> {
    let state = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state"))
        })?;
    Some(state.join(APP_DIR))
}

const CONFIG_FILE: &str = "config.toml";

#[cfg(windows)]
pub(crate) fn config_file() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|dir| PathBuf::from(dir).join(APP_DIR).join(CONFIG_FILE))
}

#[cfg(not(windows))]
pub(crate) fn config_file() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join(APP_DIR).join(CONFIG_FILE))
}
