use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

const RECORD_ENV: &str = "MIRAH_RECORD";

pub(crate) struct Recorder {
    file: Option<BufWriter<File>>,
    path: Option<PathBuf>,
}

impl Recorder {
    pub(crate) fn new(path: Option<PathBuf>) -> Self {
        let mut recorder = Self {
            file: None,
            path: None,
        };
        if let Some(path) = path.or_else(|| std::env::var_os(RECORD_ENV).map(PathBuf::from)) {
            recorder.start(path);
        }
        recorder
    }

    pub(crate) fn resume(&mut self) -> Option<bool> {
        let path = self.path.clone()?;
        Some(match OpenOptions::new().append(true).open(&path) {
            Ok(file) => {
                tracing::info!(path = ?path, "recording resumed");
                self.file = Some(BufWriter::new(file));
                true
            }
            Err(err) => {
                tracing::warn!(path = ?path, error = %err, "cannot resume recording");
                false
            }
        })
    }

    pub(crate) fn start(&mut self, path: PathBuf) -> bool {
        match File::create(&path) {
            Ok(file) => {
                tracing::info!(path = ?path, "recording child output");
                self.file = Some(BufWriter::new(file));
                self.path = Some(path);
                true
            }
            Err(err) => {
                tracing::warn!(path = ?path, error = %err, "cannot create recording");
                false
            }
        }
    }

    pub(crate) fn stop(&mut self) -> Option<PathBuf> {
        if let Some(mut file) = self.file.take() {
            let _ = file.flush();
        }
        self.path.clone()
    }

    pub(crate) fn active(&self) -> bool {
        self.file.is_some()
    }

    pub(crate) fn write(&mut self, bytes: &[u8]) {
        let Some(file) = self.file.as_mut() else {
            return;
        };
        if let Err(err) = file.write_all(bytes) {
            tracing::warn!(error = %err, "recording stopped");
            self.file = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stopping_and_starting_again_continues_the_same_file() {
        let dir = std::env::temp_dir().join(format!("mirah-record-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dir");
        let path = dir.join("claude-session-1.vt");
        let mut recorder = Recorder::new(Some(path.clone()));
        recorder.write(b"one ");
        assert_eq!(recorder.stop(), Some(path.clone()));
        recorder.write(b"lost ");
        assert_eq!(recorder.resume(), Some(true));
        recorder.write(b"two");
        recorder.stop();
        assert_eq!(std::fs::read(&path).expect("read"), b"one two");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn nothing_to_resume_before_the_first_recording() {
        assert_eq!(Recorder::new(None).resume(), None);
    }
}
