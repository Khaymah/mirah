use std::path::{Path, PathBuf};

const DEFAULT_PATHEXT: &str = ".COM;.EXE;.BAT;.CMD";
const SCRIPT_EXTENSIONS: [&str; 2] = ["bat", "cmd"];
const RUNNABLE: [&str; 4] = [".com", ".exe", ".bat", ".cmd"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Program {
    Native(PathBuf),
    Script(PathBuf),
}

fn has_directory(program: &str) -> bool {
    program.contains(['/', '\\', ':'])
}

fn kind(path: PathBuf) -> Program {
    let script = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| SCRIPT_EXTENSIONS.iter().any(|s| e.eq_ignore_ascii_case(s)));
    if script {
        Program::Script(path)
    } else {
        Program::Native(path)
    }
}

pub(crate) fn resolve(program: &str, path: Option<&str>, pathext: Option<&str>) -> Option<Program> {
    if has_directory(program) {
        let candidate = Path::new(program);
        return candidate.is_file().then(|| kind(candidate.to_path_buf()));
    }
    let extensions: Vec<&str> = pathext
        .unwrap_or(DEFAULT_PATHEXT)
        .split(';')
        .filter(|e| RUNNABLE.iter().any(|r| e.eq_ignore_ascii_case(r)))
        .collect();
    let explicit = Path::new(program).extension().is_some();
    for dir in path
        .unwrap_or_default()
        .split(';')
        .filter(|d| !d.is_empty())
    {
        let dir = Path::new(dir.trim_matches('"'));
        if explicit && dir.join(program).is_file() {
            return Some(kind(dir.join(program)));
        }
        for ext in &extensions {
            let candidate = dir.join(format!("{program}{ext}"));
            if candidate.is_file() {
                return Some(kind(candidate));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(name: &str, files: &[&str]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("mirah-resolve-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        for file in files {
            std::fs::write(dir.join(file), b"").expect("write");
        }
        dir
    }

    #[test]
    fn prefers_the_cmd_shim_over_the_extensionless_npm_script() {
        let npm = folder("npm", &["opencode", "opencode.CMD", "opencode.ps1"]);
        let path = npm.to_string_lossy().into_owned();
        assert_eq!(
            resolve("opencode", Some(&path), Some(".COM;.EXE;.BAT;.CMD")),
            Some(Program::Script(npm.join("opencode.CMD")))
        );
        let _ = std::fs::remove_dir_all(npm);
    }

    #[test]
    fn finds_an_exe_in_a_later_folder() {
        let empty = folder("empty", &[]);
        let links = folder("links", &["claude.EXE"]);
        let path = format!("{};{}", empty.display(), links.display());
        assert_eq!(
            resolve("claude", Some(&path), None),
            Some(Program::Native(links.join("claude.EXE")))
        );
        let _ = std::fs::remove_dir_all(empty);
        let _ = std::fs::remove_dir_all(links);
    }

    #[test]
    fn a_name_with_an_extension_is_used_as_is() {
        let bin = folder("ext", &["tool.cmd"]);
        let path = bin.to_string_lossy().into_owned();
        assert_eq!(
            resolve("tool.cmd", Some(&path), None),
            Some(Program::Script(bin.join("tool.cmd")))
        );
        let _ = std::fs::remove_dir_all(bin);
    }

    #[test]
    fn script_types_windows_cannot_start_directly_are_skipped() {
        let bin = folder("pathext", &["claude.JS", "claude.CMD"]);
        let path = bin.to_string_lossy().into_owned();
        assert_eq!(
            resolve("claude", Some(&path), Some(".JS;.CMD")),
            Some(Program::Script(bin.join("claude.CMD")))
        );
        let _ = std::fs::remove_dir_all(bin);
    }

    #[test]
    fn unknown_programs_are_left_to_the_system() {
        assert_eq!(resolve("nothing-here", Some(""), None), None);
    }
}
