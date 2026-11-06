use std::path::{Path, PathBuf};

const SUFFIX: &str = "-session-";
const FALLBACK_NAME: &str = "mirah";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DebugFiles {
    pub(crate) recording: PathBuf,
    pub(crate) log: PathBuf,
}

fn prefix(program: &str) -> String {
    let name = crate::config::program_name(program);
    let name = if name.is_empty() {
        FALLBACK_NAME
    } else {
        &name
    };
    format!("{name}{SUFFIX}")
}

fn number(name: &str, prefix: &str) -> Option<u32> {
    let rest = name.strip_prefix(prefix)?;
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

pub(crate) fn next_in(dir: &Path, program: &str) -> DebugFiles {
    let prefix = prefix(program);
    let last = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter_map(|e| number(&e.file_name().to_string_lossy(), &prefix))
                .max()
                .unwrap_or(0)
        })
        .unwrap_or(0);
    let stem = format!("{prefix}{}", last + 1);
    DebugFiles {
        recording: dir.join(format!("{stem}.vt")),
        log: dir.join(format!("{stem}.log")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_number_after_the_program_prefix() {
        assert_eq!(
            number("claude-session-3-system.vt", "claude-session-"),
            Some(3)
        );
        assert_eq!(
            number("opencode-session-12.log", "opencode-session-"),
            Some(12)
        );
        assert_eq!(number("claude-session-4.vt", "opencode-session-"), None);
        assert_eq!(number("claude-session-.vt", "claude-session-"), None);
    }

    #[test]
    fn each_program_counts_its_own_sessions() {
        let dir = std::env::temp_dir().join(format!("mirah-debug-files-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        for name in [
            "claude-session-1.vt",
            "claude-session-4.log",
            "opencode-session-2.vt",
            "notes.txt",
        ] {
            std::fs::write(dir.join(name), b"").expect("write");
        }
        let claude = next_in(&dir, "C:\\npm\\claude.exe");
        assert_eq!(claude.recording, dir.join("claude-session-5.vt"));
        assert_eq!(claude.log, dir.join("claude-session-5.log"));
        assert_eq!(
            next_in(&dir, "opencode").recording,
            dir.join("opencode-session-3.vt")
        );
        assert_eq!(
            next_in(&dir, "gemini").recording,
            dir.join("gemini-session-1.vt")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
