mod agents;
mod chrome;
mod cli;
mod command;
mod config;
mod debug_files;
mod host;
mod logging;
mod paths;
mod session;
mod store;
mod update;
mod viewer;

use session::Spawn;
use std::path::PathBuf;
use store::Store;

const FAILURE: u32 = 1;

#[allow(
    clippy::print_stderr,
    reason = "the terminal is already restored when the error is reported"
)]
fn report(text: &str) {
    eprintln!("mirah: {text}");
}

#[allow(
    clippy::print_stdout,
    reason = "version, help and the session list are printed before the terminal is taken over"
)]
fn print(text: &str) {
    println!("{text}");
}

fn exit(code: u32) -> ! {
    std::process::exit(i32::try_from(code).unwrap_or(1))
}

fn here() -> PathBuf {
    std::env::current_dir().unwrap_or_default()
}

fn first_tab(argv: Vec<String>, extra: &[agents::Agent]) -> Option<Spawn> {
    let program = argv.first()?;
    let agent =
        agents::find(program, extra).map_or_else(|| config::program_name(program), |a| a.name);
    Some(Spawn {
        agent,
        argv,
        cwd: here(),
        meta: None,
        document: false,
    })
}

fn resumed_tab(store: Option<&Store>, id: Option<&str>, extra: &[agents::Agent]) -> Spawn {
    let meta = store.and_then(|store| match id {
        Some(id) => store.load(id),
        None => store.list().into_iter().next(),
    });
    let Some(meta) = meta else {
        report(&match id {
            Some(id) => format!("no session with the ID {id}. Run `mirah sessions` to list them."),
            None => "there are no sessions yet.".to_string(),
        });
        exit(FAILURE);
    };
    let agent = agents::find(&meta.agent, extra).unwrap_or_else(|| agents::Agent {
        name: meta.agent.clone(),
        title: meta.agent.clone(),
        command: meta.argv.first().cloned().unwrap_or_default(),
        resume: Vec::new(),
    });
    Spawn {
        argv: agents::resume_argv(&agent, &meta.argv),
        agent: meta.agent.clone(),
        cwd: meta.cwd.clone(),
        meta: Some(meta),
        document: false,
    }
}

fn recording(what: &str) -> PathBuf {
    let path = PathBuf::from(what);
    if path.exists() {
        return path;
    }
    match Store::open() {
        Some(store) if store::looks_like_id(what) && store.output(what).exists() => {
            store.output(what)
        }
        _ => {
            report(&format!(
                "no recording at {what}, and no session with that ID has one."
            ));
            exit(FAILURE);
        }
    }
}

fn log_file(store: Option<&Store>, first: Option<&Spawn>) -> (Option<PathBuf>, Option<String>) {
    match (store, first.and_then(|s| s.meta.as_ref())) {
        (Some(store), Some(meta)) => (Some(store.log(&meta.id)), None),
        (Some(store), None) => {
            let id = store.reserve();
            (Some(store.log(&id)), Some(id))
        }
        (None, _) => {
            let program = first
                .and_then(|s| s.argv.first().cloned())
                .unwrap_or_default();
            (Some(debug_files::next_in(&here(), &program).log), None)
        }
    }
}

fn main() {
    let (first, record) = match cli::parse(std::env::args().skip(1)) {
        cli::Invocation::Print(text) => {
            print(&text);
            return;
        }
        cli::Invocation::Update => {
            let _log_guard = logging::init(None);
            exit(update::command());
        }
        cli::Invocation::Sessions => {
            match Store::open() {
                Some(store) => print(&store::table(&store, store::now())),
                None => report("there is no folder to keep sessions in."),
            }
            return;
        }
        cli::Invocation::Player(path) => {
            let log_guard = logging::init(None);
            let code = viewer::play::run(&path);
            drop(log_guard);
            exit(code);
        }
        cli::Invocation::Play(what) => (Ok(Err(recording(&what))), false),
        cli::Invocation::Pager(path) => {
            let log_guard = logging::init(None);
            let code = viewer::run(&path);
            drop(log_guard);
            exit(code);
        }
        cli::Invocation::View(path) => (Ok(Err(path)), false),
        cli::Invocation::Resume(id) => (Err(id), false),
        cli::Invocation::Run(run) => (Ok(Ok(run.argv)), run.record),
    };
    let config = config::Config::load();
    let extra = config.agents();
    let store = Store::open();
    let first = match first {
        Ok(Ok(argv)) => first_tab(argv, &extra),
        Ok(Err(path)) => Some(viewer::spawn(&path)),
        Err(id) => Some(resumed_tab(store.as_ref(), id.as_deref(), &extra)),
    };
    let program = first
        .as_ref()
        .and_then(|s| s.argv.first().cloned())
        .unwrap_or_default();
    let record = record || config.settings(&program).record;
    let (log, reserved) = if record {
        log_file(store.as_ref(), first.as_ref())
    } else {
        (None, None)
    };
    let log_guard = logging::init(log.as_deref());
    update::remove_leftover();
    host::install_panic_hook();
    if let Some(store) = &store {
        let removed = store.clean(store::now());
        tracing::debug!(removed, "old sessions cleaned");
    }
    let start = session::Start {
        first,
        record,
        reserved: reserved.clone(),
        store,
        config,
        agents: extra,
    };
    let root = start.store.as_ref().map(|s| s.root().to_path_buf());
    let result = session::run(start);
    drop(log_guard);
    let code = match result {
        Ok(outcome) => {
            for id in &outcome.sessions {
                report(&format!(
                    "session {id} · continue it with `mirah resume {id}`"
                ));
            }
            if let (Some(root), false) = (&root, outcome.sessions.is_empty()) {
                report(&format!("sessions are kept in {}", root.display()));
            }
            outcome.code
        }
        Err(err) => {
            host::disable_raw_mode();
            tracing::error!(error = format!("{err:#}"), "session failed");
            report(&format!("{err:#}"));
            FAILURE
        }
    };
    if let (Some(store), Some(id)) = (Store::open(), reserved) {
        store::forget_if_empty(&store, &id);
    }
    exit(code);
}
