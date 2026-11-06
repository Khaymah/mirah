use std::fs::File;
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
        self.path.take()
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
