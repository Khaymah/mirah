use std::path::{Path, PathBuf};

const PREFIX: &str = "mirah-session-";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DebugFiles {
    pub(crate) recording: PathBuf,
    pub(crate) log: PathBuf,
}

fn number(name: &str) -> Option<u32> {
    let rest = name.strip_prefix(PREFIX)?;
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

pub(crate) fn next_in(dir: &Path) -> DebugFiles {
    let last = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter_map(|e| number(&e.file_name().to_string_lossy()))
                .max()
                .unwrap_or(0)
        })
        .unwrap_or(0);
    let stem = format!("{PREFIX}{}", last + 1);
    DebugFiles {
        recording: dir.join(format!("{stem}.vt")),
        log: dir.join(format!("{stem}.log")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_number_after_the_prefix() {
        assert_eq!(number("mirah-session-3-system.vt"), Some(3));
        assert_eq!(number("mirah-session-12.log"), Some(12));
        assert_eq!(number("mirah.log"), None);
        assert_eq!(number("mirah-session-.vt"), None);
    }

    #[test]
    fn picks_the_number_after_the_last_session_in_the_folder() {
        let dir = std::env::temp_dir().join(format!("mirah-debug-files-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        for name in [
            "mirah-session-1.vt",
            "mirah-session-4.vt",
            "mirah-session-3-system.vt",
            "notes.txt",
        ] {
            std::fs::write(dir.join(name), b"").expect("write");
        }
        let next = next_in(&dir);
        assert_eq!(next.recording, dir.join("mirah-session-5.vt"));
        assert_eq!(next.log, dir.join("mirah-session-5.log"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn starts_at_one_in_an_empty_folder() {
        let dir = std::env::temp_dir().join(format!("mirah-debug-empty-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        assert_eq!(next_in(&dir).recording, dir.join("mirah-session-1.vt"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
