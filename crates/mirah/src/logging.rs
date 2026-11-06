use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

const LOG_ENV: &str = "MIRAH_LOG";
const LOG_FILE: &str = "mirah.log";
const DEBUG_FILTER: &str = "debug";

pub(crate) fn init(file: Option<&Path>) -> Option<WorkerGuard> {
    let env_filter = std::env::var(LOG_ENV).ok().filter(|v| !v.is_empty());
    let (filter, dir, name) = match file {
        Some(file) => (
            env_filter.unwrap_or_else(|| DEBUG_FILTER.to_string()),
            file.parent().map(Path::to_path_buf).unwrap_or_default(),
            file.file_name()?.to_os_string(),
        ),
        None => (env_filter?, crate::paths::state_dir()?, LOG_FILE.into()),
    };
    if !dir.as_os_str().is_empty() {
        std::fs::create_dir_all(&dir).ok()?;
    }
    let (writer, guard) =
        tracing_appender::non_blocking(tracing_appender::rolling::never(dir, name));
    tracing_subscriber::fmt()
        .with_writer(writer)
        .with_ansi(false)
        .with_env_filter(EnvFilter::new(filter))
        .try_init()
        .ok()?;
    Some(guard)
}
