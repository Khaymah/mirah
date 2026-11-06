use portable_pty::CommandBuilder;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const HELP: &str = "mirah - run any terminal program with correct right-to-left text

Usage:
  mirah [PROGRAM [ARGS...]]   run PROGRAM inside mirah (default: your shell)
  mirah --record PROGRAM ...  same, and save PROGRAM-session-N.vt and
                              PROGRAM-session-N.log in the current folder
  mirah update                install the newest release
  mirah -- PROGRAM ...        run a program named like a mirah command
  mirah --version             print the version
  mirah --help                print this help

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
  MIRAH_BAR=0                 hide the mirah bar (Ctrl+] opens the bar when shown)

Settings are saved in config.toml (%APPDATA%\\mirah on Windows,
~/.config/mirah elsewhere).

https://github.com/Khaymah/mirah";

pub(crate) enum Invocation {
    Run(Run),
    Update,
    Print(String),
}

pub(crate) struct Run {
    pub(crate) cmd: CommandBuilder,
    pub(crate) program: String,
    pub(crate) record: bool,
}

fn run(args: impl Iterator<Item = String>, record: bool) -> Invocation {
    let (cmd, program) = crate::command::from_args(args);
    Invocation::Run(Run {
        cmd,
        program,
        record,
    })
}

pub(crate) fn parse(mut args: impl Iterator<Item = String>) -> Invocation {
    let mut args = args.by_ref().peekable();
    match args.peek().map(String::as_str) {
        Some("--version" | "-V") => Invocation::Print(format!("mirah {VERSION}")),
        Some("--help" | "-h") => Invocation::Print(HELP.to_string()),
        Some("update") => Invocation::Update,
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

    fn argv(cmd: &CommandBuilder) -> Vec<String> {
        cmd.get_argv()
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn update_is_a_command_and_double_dash_runs_a_program_with_that_name() {
        assert!(matches!(parse_args(&["update"]), Invocation::Update));
        let Invocation::Run(run) = parse_args(&["--", "update", "x"]) else {
            panic!("expected a program to run");
        };
        assert_eq!(argv(&run.cmd), ["update", "x"]);
    }

    #[test]
    fn record_comes_before_the_program() {
        let Invocation::Run(run) = parse_args(&["--record", "claude", "-r"]) else {
            panic!("expected a program to run");
        };
        assert!(run.record);
        assert_eq!(run.program, "claude");
        assert_eq!(argv(&run.cmd), ["claude", "-r"]);
    }

    #[test]
    fn flags_after_the_program_belong_to_the_program() {
        let Invocation::Run(run) = parse_args(&["claude", "--version"]) else {
            panic!("expected a program to run");
        };
        assert!(!run.record);
        assert_eq!(argv(&run.cmd), ["claude", "--version"]);
    }
}
