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
            return through_cmd(&script, &args);
        }
        None => CommandBuilder::new(&program),
    };
    cmd.args(args);
    cmd
}

#[cfg(windows)]
const BATCH_LINE_ENV: &str = "MIRAH_BATCH_LINE";

#[cfg(windows)]
fn through_cmd(script: &std::path::Path, args: &[String]) -> CommandBuilder {
    let shell = std::env::var(SHELL_ENV).unwrap_or_else(|_| FALLBACK_SHELL.into());
    let mut cmd = CommandBuilder::new(shell);
    cmd.args(["/d", "/v:off", "/c"]);
    cmd.arg(format!("%{BATCH_LINE_ENV}%"));
    cmd.env(
        BATCH_LINE_ENV,
        shim::batch_line(&script.to_string_lossy(), args),
    );
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

    #[cfg(windows)]
    #[test]
    fn a_batch_file_in_a_folder_with_spaces_gets_its_arguments_intact() {
        let dir = std::env::temp_dir().join(format!("mirah cmd test {}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dir");
        let script = dir.join("show.cmd");
        std::fs::write(&script, "@echo off\r\necho ARGS=%*\r\n").expect("write");
        let marker = dir.join("ran");
        let args = vec![
            "a b".to_string(),
            format!("x&echo hit>\"{}\"", marker.display()),
            "100%PATH%".to_string(),
        ];
        let built = through_cmd(&script, &args);
        let argv: Vec<String> = built
            .get_argv()
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let line = built
            .get_env(BATCH_LINE_ENV)
            .map(|v| v.to_string_lossy().into_owned())
            .expect("line");
        let out = std::process::Command::new(&argv[0])
            .args(&argv[1..])
            .env(BATCH_LINE_ENV, line)
            .output()
            .expect("run cmd");
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(text.contains("ARGS=\"a b\""), "{text}");
        assert!(text.contains("100%PATH%"), "{text}");
        assert!(!marker.exists(), "cmd ran part of an argument");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[cfg(windows)]
    #[test]
    fn an_npm_shim_runs_the_node_beside_it_with_its_script() {
        let dir = std::env::temp_dir().join(format!("mirah shim test {}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dir");
        let script = dir.join("codex.cmd");
        std::fs::write(
            &script,
            "@ECHO off\r\nSETLOCAL\r\nIF EXIST \"%dp0%\\node.exe\" (\r\n  SET \"_prog=%dp0%\\node.exe\"\r\n) ELSE (\r\n  SET \"_prog=node\"\r\n)\r\nendLocal & goto #_undefined_# 2>NUL || title %COMSPEC% & \"%_prog%\"  \"%dp0%\\node_modules\\codex\\bin\\codex.js\" %*\r\n",
        )
        .expect("write shim");
        std::fs::write(dir.join("node.exe"), b"").expect("write node");
        let built = through_shim(&script, Some(""), None).expect("a shim");
        let argv: Vec<String> = built
            .get_argv()
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            argv,
            [
                dir.join("node.exe").to_string_lossy().into_owned(),
                format!("{}\\node_modules\\codex\\bin\\codex.js", dir.display()),
            ]
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn starts_a_shell_when_no_program_is_given() {
        assert_eq!(argv(&[]).len(), 1);
    }
}
