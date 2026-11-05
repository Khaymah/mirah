mod cli;
mod command;
mod host;
mod logging;
mod session;

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

fn main() {
    let cmd = match cli::parse(std::env::args().skip(1)) {
        cli::Invocation::Print(text) => {
            print(&text);
            return;
        }
        cli::Invocation::Run(cmd) => cmd,
    };
    let log_guard = logging::init();
    host::install_panic_hook();
    let code = match session::run(cmd) {
        Ok(code) => code,
        Err(err) => {
            host::disable_raw_mode();
            tracing::error!(error = format!("{err:#}"), "session failed");
            report(&err);
            FAILURE
        }
    };
    drop(log_guard);
    std::process::exit(i32::try_from(code).unwrap_or(1));
}
