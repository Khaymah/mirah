use std::path::Path;

const DIR_VAR: &str = "%dp0%";
const PROGRAM_VAR: &str = "\"%_prog%\"";
const ALL_ARGS: &str = "%*";
const PROGRAM_SET: &str = "SET \"_prog=";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Shim {
    pub(crate) interpreter: Option<String>,
    pub(crate) args: Vec<String>,
}

fn tokens(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = text.trim().chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }
        let mut token = String::new();
        if c == '"' {
            chars.next();
            for c in chars.by_ref() {
                if c == '"' {
                    break;
                }
                token.push(c);
            }
        } else {
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() || c == '"' {
                    break;
                }
                token.push(c);
                chars.next();
            }
        }
        out.push(token);
    }
    out
}

fn expand(token: &str, dir: &Path) -> Option<String> {
    let expanded = match token.strip_prefix(DIR_VAR) {
        Some(rest) => format!(
            "{}\\{}",
            dir.display(),
            rest.trim_start_matches(['\\', '/'])
        ),
        None => token.to_string(),
    };
    (!expanded.contains(['%', '&', '|', '<', '>', '^'])).then_some(expanded)
}

fn interpreter(text: &str) -> Option<String> {
    text.lines()
        .filter_map(|line| line.trim().strip_prefix(PROGRAM_SET))
        .find(|value| !value.starts_with(DIR_VAR))
        .and_then(|value| value.split('"').next())
        .filter(|name| !name.is_empty() && !name.contains(['%', '\\', '/']))
        .map(str::to_string)
}

pub(crate) fn parse(text: &str, dir: &Path) -> Option<Shim> {
    let line = text
        .lines()
        .rev()
        .find(|l| l.trim_end().ends_with(ALL_ARGS))?;
    let line = line.trim_end().strip_suffix(ALL_ARGS)?;
    let (interpreter, rest) = match line.find(PROGRAM_VAR) {
        Some(at) => (Some(interpreter(text)?), &line[at + PROGRAM_VAR.len()..]),
        None => (None, line),
    };
    let args = tokens(rest)
        .iter()
        .map(|t| expand(t, dir))
        .collect::<Option<Vec<_>>>()?;
    let target = args.first()?;
    if interpreter.is_none() && !target.to_ascii_lowercase().ends_with(".exe") {
        return None;
    }
    Some(Shim { interpreter, args })
}

#[cfg(windows)]
pub(crate) fn local_interpreter(name: &str, dir: &Path) -> Option<std::path::PathBuf> {
    let local = dir.join(format!("{name}.exe"));
    local.is_file().then_some(local)
}

fn quoted_for_cmd(part: &str) -> String {
    let flat: String = part
        .chars()
        .map(|c| {
            if matches!(c, '\r' | '\n' | '\0') {
                ' '
            } else {
                c
            }
        })
        .collect();
    format!("\"{}\"", flat.replace('"', "\"\""))
}

pub(crate) fn batch_line(script: &str, args: &[String]) -> String {
    std::iter::once(script)
        .chain(args.iter().map(String::as_str))
        .map(quoted_for_cmd)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    const NODE_SHIM: &str = "@ECHO off\r\nGOTO start\r\n:find_dp0\r\nSET dp0=%~dp0\r\nEXIT /b\r\n:start\r\nSETLOCAL\r\nCALL :find_dp0\r\n\r\nIF EXIST \"%dp0%\\node.exe\" (\r\n  SET \"_prog=%dp0%\\node.exe\"\r\n) ELSE (\r\n  SET \"_prog=node\"\r\n  SET PATHEXT=%PATHEXT:;.JS;=;%\r\n)\r\n\r\nendLocal & goto #_undefined_# 2>NUL || title %COMSPEC% & \"%_prog%\"  \"%dp0%\\node_modules\\@openai\\codex\\bin\\codex.js\" %*\r\n";

    const EXE_SHIM: &str = "@ECHO off\r\nGOTO start\r\n:find_dp0\r\nSET dp0=%~dp0\r\nEXIT /b\r\n:start\r\nSETLOCAL\r\nCALL :find_dp0\r\n\"%dp0%\\node_modules\\@anthropic-ai\\claude-code\\bin\\claude.exe\"   %*\r\n";

    #[test]
    fn an_npm_node_shim_runs_its_script_with_node() {
        let shim = parse(NODE_SHIM, Path::new("C:\\Users\\Sabry Dawood\\npm")).expect("shim");
        assert_eq!(shim.interpreter.as_deref(), Some("node"));
        assert_eq!(
            shim.args,
            ["C:\\Users\\Sabry Dawood\\npm\\node_modules\\@openai\\codex\\bin\\codex.js"]
        );
    }

    #[test]
    fn an_npm_exe_shim_runs_the_exe_itself() {
        let shim = parse(EXE_SHIM, Path::new("C:\\npm")).expect("shim");
        assert_eq!(shim.interpreter, None);
        assert_eq!(
            shim.args,
            ["C:\\npm\\node_modules\\@anthropic-ai\\claude-code\\bin\\claude.exe"]
        );
    }

    #[test]
    fn any_other_script_is_not_a_shim() {
        assert_eq!(
            parse("@echo off\r\necho hi %*\r\n", Path::new("C:\\")),
            None
        );
        assert_eq!(parse("@echo off\r\nset X=1\r\n", Path::new("C:\\")), None);
        assert_eq!(
            parse("\"%_prog%\" \"%dp0%\\a.js\" %*\r\n", Path::new("C:\\")),
            None
        );
        assert_eq!(
            parse(
                "SET \"_prog=node\"\r\n\"%_prog%\" \"%dp0%\\a.js\" %OTHER% %*\r\n",
                Path::new("C:\\")
            ),
            None
        );
    }

    #[test]
    fn a_batch_line_quotes_every_part_so_cmd_runs_nothing_from_it() {
        let args = vec![
            "a b".to_string(),
            "x&calc".to_string(),
            "say \"hi\"".to_string(),
            "two\nlines".to_string(),
        ];
        assert_eq!(
            batch_line("C:\\Users\\Sabry Dawood\\tool.cmd", &args),
            "\"C:\\Users\\Sabry Dawood\\tool.cmd\" \"a b\" \"x&calc\" \"say \"\"hi\"\"\" \"two lines\""
        );
    }
}
