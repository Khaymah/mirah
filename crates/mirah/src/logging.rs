use std::path::PathBuf;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

const LOG_ENV: &str = "MIRAH_LOG";
const LOG_FILE: &str = "mirah.log";
const APP_DIR: &str = "mirah";

fn log_dir() -> Option<PathBuf> {
    let state = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state"))
        })?;
    Some(state.join(APP_DIR))
}

pub(crate) fn init() -> Option<WorkerGuard> {
    let filter = std::env::var(LOG_ENV).ok().filter(|v| !v.is_empty())?;
    let dir = log_dir()?;
    std::fs::create_dir_all(&dir).ok()?;
    let (writer, guard) =
        tracing_appender::non_blocking(tracing_appender::rolling::never(dir, LOG_FILE));
    tracing_subscriber::fmt()
        .with_writer(writer)
        .with_ansi(false)
        .with_env_filter(EnvFilter::new(filter))
        .try_init()
        .ok()?;
    Some(guard)
}
