mod chrome;
mod cli;
mod command;
mod config;
mod debug_files;
mod host;
mod logging;
mod paths;
mod session;
mod update;

const FAILURE: u32 = 1;

#[allow(
    clippy::print_stderr,
    reason = "the terminal is already restored when the error is reported"
)]
fn report(err: &anyhow::Error) {
    eprintln!("mirah: {err:#}");
}

#[allow(
    clippy::print_stdout,
    reason = "version and help are printed before the terminal is taken over"
)]
fn print(text: &str) {
    println!("{text}");
}

#[allow(
    clippy::print_stderr,
    reason = "the terminal is already restored when the saved files are listed"
)]
fn report_saved(files: &debug_files::DebugFiles) {
    eprintln!(
        "mirah: saved {} and {}",
        files.recording.display(),
        files.log.display()
    );
}

fn main() {
    let run = match cli::parse(std::env::args().skip(1)) {
        cli::Invocation::Print(text) => {
            print(&text);
            return;
        }
        cli::Invocation::Update => {
            let _log_guard = logging::init(None);
            let code = update::command();
            std::process::exit(i32::try_from(code).unwrap_or(1));
        }
        cli::Invocation::Run(run) => run,
    };
    let files = run
        .record
        .then(|| debug_files::next_in(&std::env::current_dir().unwrap_or_default()));
    let log_guard = logging::init(files.as_ref().map(|f| f.log.as_path()));
    update::remove_leftover();
    host::install_panic_hook();
    let recording = files.as_ref().map(|f| f.recording.clone());
    let code = match session::run(run.cmd, &run.program, recording) {
        Ok(code) => code,
        Err(err) => {
            host::disable_raw_mode();
            tracing::error!(error = format!("{err:#}"), "session failed");
            report(&err);
            FAILURE
        }
    };
    drop(log_guard);
    if let Some(files) = &files {
        report_saved(files);
    }
    std::process::exit(i32::try_from(code).unwrap_or(1));
}
