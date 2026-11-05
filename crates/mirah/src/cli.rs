use portable_pty::CommandBuilder;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const HELP: &str = "mirah - run any terminal program with correct right-to-left text

Usage:
  mirah [PROGRAM [ARGS...]]   run PROGRAM inside mirah (default: your shell)
  mirah --version             print the version
  mirah --help                print this help

Environment:
  MIRAH_LOG=debug             write a log file (never to the terminal)
  MIRAH_ALIGN=prose|arabic|off
                              right-align prose (default), Arabic only, or nothing
  MIRAH_SHAPING=full|plain    full: mirah joins letters (default)
                              plain: the terminal joins letters (Windows Terminal)
  MIRAH_PASS_WT_SESSION=1     let the program see Windows Terminal (it may then
                              reorder Arabic itself and mirah would reverse it)
  MIRAH_RECORD=<file>         save the program's raw output for bug reports

https://github.com/Khaymah/mirah";

pub(crate) enum Invocation {
    Run(CommandBuilder),
    Print(String),
}

pub(crate) fn parse(mut args: impl Iterator<Item = String>) -> Invocation {
    let mut args = args.by_ref().peekable();
    match args.peek().map(String::as_str) {
        Some("--version" | "-V") => Invocation::Print(format!("mirah {VERSION}")),
        Some("--help" | "-h") => Invocation::Print(HELP.to_string()),
        _ => Invocation::Run(crate::command::from_args(args)),
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

    #[test]
    fn flags_after_the_program_belong_to_the_program() {
        let Invocation::Run(cmd) = parse_args(&["claude", "--version"]) else {
            panic!("expected a program to run");
        };
        let argv: Vec<String> = cmd
            .get_argv()
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(argv, ["claude", "--version"]);
    }
}
