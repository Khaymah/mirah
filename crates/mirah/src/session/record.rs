use std::fs::File;
use std::io::{BufWriter, Write};

const RECORD_ENV: &str = "MIRAH_RECORD";

pub(crate) struct Recorder {
    file: Option<BufWriter<File>>,
}

impl Recorder {
    pub(crate) fn from_env() -> Self {
        let file = std::env::var_os(RECORD_ENV).and_then(|path| match File::create(&path) {
            Ok(file) => {
                tracing::info!(path = ?path, "recording child output");
                Some(BufWriter::new(file))
            }
            Err(err) => {
                tracing::warn!(path = ?path, error = %err, "cannot create recording");
                None
            }
        });
        Self { file }
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
