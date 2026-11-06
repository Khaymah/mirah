const VERSION: &str = env!("CARGO_PKG_VERSION");
const HELP: &str = "mirah - run any terminal program with correct right-to-left text

Usage:
  mirah                       pick a program (Claude Code, OpenCode, ...) to open
  mirah PROGRAM [ARGS...]     run PROGRAM inside mirah
  mirah --record PROGRAM ...  same, and save the output and a full log in the
                              session folder
  mirah sessions              list recent sessions and their IDs
  mirah resume [ID]           continue a session (the newest one without an ID)
  mirah update                install the newest release
  mirah -- PROGRAM ...        run a program named like a mirah command
  mirah --version             print the version
  mirah --help                print this help

Inside mirah:
  Ctrl+]                      open the bar (arrows move, Enter picks, Esc closes)
  Ctrl+] then 1-9             go to that tab
  Ctrl+] then t               open a new tab
  Ctrl+] Ctrl+]               send Ctrl+] to the program

Environment:
  MIRAH_LOG=debug             write a log file (never to the terminal)
  MIRAH_ALIGN=prose|arabic|off
                              right-align prose (default), Arabic only, or nothing
  MIRAH_SHAPING=full|plain    full: mirah joins letters (default)
                              plain: the terminal joins letters (Windows Terminal)
  MIRAH_INPUT=logical|visual  visual: the program already reorders Arabic itself
                              (picked for Claude Code on Windows)
  MIRAH_CONPTY=system         Windows: use the conpty that ships with Windows
                              instead of the one bundled with mirah
  MIRAH_RECORD=<file>         save the program's raw output for bug reports
  MIRAH_NO_UPDATE_CHECK=1     do not check for a new version at startup
  MIRAH_BAR=0                 hide the mirah bar

Settings are saved in config.toml (%APPDATA%\\mirah on Windows,
~/.config/mirah elsewhere). Sessions are kept in %LOCALAPPDATA%\\mirah\\sessions
on Windows and ~/.local/state/mirah/sessions elsewhere.

https://github.com/Khaymah/mirah";

pub(crate) enum Invocation {
    Run(Run),
    Resume(Option<String>),
    Sessions,
    Update,
    Print(String),
}

pub(crate) struct Run {
    pub(crate) argv: Vec<String>,
    pub(crate) record: bool,
}

fn run(args: impl Iterator<Item = String>, record: bool) -> Invocation {
    Invocation::Run(Run {
        argv: args.collect(),
        record,
    })
}

pub(crate) fn parse(mut args: impl Iterator<Item = String>) -> Invocation {
    let mut args = args.by_ref().peekable();
    match args.peek().map(String::as_str) {
        Some("--version" | "-V") => Invocation::Print(format!("mirah {VERSION}")),
        Some("--help" | "-h") => Invocation::Print(HELP.to_string()),
        Some("update") => Invocation::Update,
        Some("sessions") => Invocation::Sessions,
        Some("resume") => {
            args.next();
            Invocation::Resume(args.next())
        }
        Some("--") => {
            args.next();
            run(args, false)
        }
        Some("--record" | "-r") => {
            args.next();
            run(args, true)
        }
        _ => run(args, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_args(args: &[&str]) -> Invocation {
        parse(args.iter().map(|s| (*s).to_string()))
    }

    fn argv(invocation: Invocation) -> (Vec<String>, bool) {
        match invocation {
            Invocation::Run(run) => (run.argv, run.record),
            _ => panic!("expected a program to run"),
        }
    }

    #[test]
    fn prints_the_version() {
        assert!(
            matches!(parse_args(&["--version"]), Invocation::Print(v) if v == format!("mirah {VERSION}"))
        );
    }

    #[test]
    fn prints_help() {
        assert!(matches!(parse_args(&["-h"]), Invocation::Print(h) if h.contains("Usage:")));
    }

    #[test]
    fn nothing_opens_the_launcher() {
        assert_eq!(argv(parse_args(&[])), (vec![], false));
    }

    #[test]
    fn commands_and_double_dash() {
        assert!(matches!(parse_args(&["update"]), Invocation::Update));
        assert!(matches!(parse_args(&["sessions"]), Invocation::Sessions));
        assert!(matches!(parse_args(&["resume"]), Invocation::Resume(None)));
        assert!(
            matches!(parse_args(&["resume", "1106-0457-k3f2"]), Invocation::Resume(Some(id)) if id == "1106-0457-k3f2")
        );
        assert_eq!(argv(parse_args(&["--", "update", "x"])).0, ["update", "x"]);
    }

    #[test]
    fn record_comes_before_the_program() {
        assert_eq!(
            argv(parse_args(&["--record", "claude", "-r"])),
            (vec!["claude".to_string(), "-r".to_string()], true)
        );
    }

    #[test]
    fn flags_after_the_program_belong_to_the_program() {
        assert_eq!(
            argv(parse_args(&["claude", "--version"])),
            (vec!["claude".to_string(), "--version".to_string()], false)
        );
    }
}
