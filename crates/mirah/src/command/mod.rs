#[cfg(any(windows, test))]
mod resolve;
#[cfg(any(windows, test))]
mod shim;

use portable_pty::CommandBuilder;

#[cfg(windows)]
const SHELL_ENV: &str = "COMSPEC";
#[cfg(windows)]
const FALLBACK_SHELL: &str = "cmd.exe";
#[cfg(not(windows))]
const SHELL_ENV: &str = "SHELL";
#[cfg(not(windows))]
const FALLBACK_SHELL: &str = "/bin/sh";

pub(crate) fn shell() -> String {
    std::env::var(SHELL_ENV).unwrap_or_else(|_| FALLBACK_SHELL.into())
}

pub(crate) fn command(argv: &[String], cwd: &std::path::Path) -> CommandBuilder {
    let program = argv.first().cloned().unwrap_or_else(shell);
    let mut cmd = build(program, argv.iter().skip(1).cloned().collect());
    cmd.cwd(cwd);
    cmd
}

#[cfg(windows)]
pub(crate) fn installed(program: &str) -> bool {
    let path = std::env::var("PATH").ok();
    let pathext = std::env::var("PATHEXT").ok();
    resolve::resolve(program, path.as_deref(), pathext.as_deref()).is_some()
}

#[cfg(not(windows))]
pub(crate) fn installed(program: &str) -> bool {
    use std::os::unix::fs::PermissionsExt;
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| {
        std::fs::metadata(dir.join(program))
            .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    })
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
            if let Some(cmd) = through_shim(&script, path.as_deref(), pathext.as_deref()) {
                let mut cmd = cmd;
                cmd.args(args);
                return cmd;
            }
            let shell = std::env::var(SHELL_ENV).unwrap_or_else(|_| FALLBACK_SHELL.into());
            let mut cmd = CommandBuilder::new(shell);
            cmd.args(["/d", "/c"]);
            cmd.arg(script);
            cmd.args(args.iter().map(|a| shim::escape_for_cmd(a)));
            return cmd;
        }
        None => CommandBuilder::new(&program),
    };
    cmd.args(args);
    cmd
}

#[cfg(windows)]
fn through_shim(
    script: &std::path::Path,
    path: Option<&str>,
    pathext: Option<&str>,
) -> Option<CommandBuilder> {
    let dir = script.parent()?;
    let text = std::fs::read_to_string(script).ok()?;
    let shim = shim::parse(&text, dir)?;
    let (program, rest) = match &shim.interpreter {
        Some(name) => {
            let found = shim::local_interpreter(name, dir).or_else(|| {
                match resolve::resolve(name, path, pathext) {
                    Some(resolve::Program::Native(exe)) => Some(exe),
                    _ => None,
                }
            })?;
            (found, shim.args.as_slice())
        }
        None => (std::path::PathBuf::from(&shim.args[0]), &shim.args[1..]),
    };
    if !program.is_file() {
        return None;
    }
    tracing::debug!(script = %script.display(), program = %program.display(), "npm shim run directly");
    let mut cmd = CommandBuilder::new(program);
    cmd.args(rest);
    Some(cmd)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(args: &[&str]) -> Vec<String> {
        let args: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();
        command(&args, std::path::Path::new("."))
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
