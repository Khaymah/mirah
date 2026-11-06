#[cfg(any(windows, test))]
mod resolve;

use portable_pty::CommandBuilder;

#[cfg(windows)]
const SHELL_ENV: &str = "COMSPEC";
#[cfg(windows)]
const FALLBACK_SHELL: &str = "cmd.exe";
#[cfg(not(windows))]
const SHELL_ENV: &str = "SHELL";
#[cfg(not(windows))]
const FALLBACK_SHELL: &str = "/bin/sh";

pub(crate) fn from_args(mut args: impl Iterator<Item = String>) -> (CommandBuilder, String) {
    let program = args
        .next()
        .unwrap_or_else(|| std::env::var(SHELL_ENV).unwrap_or_else(|_| FALLBACK_SHELL.into()));
    let mut cmd = build(program.clone(), args.collect());
    if let Ok(dir) = std::env::current_dir() {
        cmd.cwd(dir);
    }
    (cmd, program)
}

#[cfg(not(windows))]
fn build(program: String, args: Vec<String>) -> CommandBuilder {
    let mut cmd = CommandBuilder::new(program);
    cmd.args(args);
    cmd
}

#[cfg(windows)]
fn build(program: String, args: Vec<String>) -> CommandBuilder {
    let path = std::env::var("PATH").ok();
    let pathext = std::env::var("PATHEXT").ok();
    let mut cmd = match resolve::resolve(&program, path.as_deref(), pathext.as_deref()) {
        Some(resolve::Program::Native(exe)) => CommandBuilder::new(exe),
        Some(resolve::Program::Script(script)) => {
            let shell = std::env::var(SHELL_ENV).unwrap_or_else(|_| FALLBACK_SHELL.into());
            let mut cmd = CommandBuilder::new(shell);
            cmd.args(["/d", "/c"]);
            cmd.arg(script);
            cmd
        }
        None => CommandBuilder::new(&program),
    };
    cmd.args(args);
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(args: &[&str]) -> Vec<String> {
        from_args(args.iter().map(|s| (*s).to_string()))
            .0
            .get_argv()
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn passes_the_program_and_its_arguments_through() {
        assert_eq!(argv(&["claude", "--resume"]), ["claude", "--resume"]);
    }

    #[test]
    fn starts_a_shell_when_no_program_is_given() {
        assert_eq!(argv(&[]).len(), 1);
    }
}
