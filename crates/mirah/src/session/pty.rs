use crate::host::TerminalSize;
use anyhow::{Context, Result};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use std::io::{Read, Write};

pub(crate) type ChildProcess = Box<dyn Child + Send + Sync>;

pub(crate) struct Pty {
    master: Box<dyn MasterPty + Send>,
}

fn pty_size(size: TerminalSize) -> PtySize {
    PtySize {
        rows: size.rows,
        cols: size.cols,
        pixel_width: 0,
        pixel_height: 0,
    }
}

impl Pty {
    pub(crate) fn spawn(cmd: CommandBuilder, size: TerminalSize) -> Result<(Self, ChildProcess)> {
        let pair = native_pty_system()
            .openpty(pty_size(size))
            .context("open pty")?;
        let child = pair.slave.spawn_command(cmd).context("spawn command")?;
        Ok((
            Self {
                master: pair.master,
            },
            child,
        ))
    }

    pub(crate) fn reader(&self) -> Result<Box<dyn Read + Send>> {
        self.master.try_clone_reader().context("clone pty reader")
    }

    pub(crate) fn writer(&self) -> Result<Box<dyn Write + Send>> {
        self.master.take_writer().context("take pty writer")
    }

    pub(crate) fn resize(&self, size: TerminalSize) {
        if let Err(err) = self.master.resize(pty_size(size)) {
            tracing::warn!(error = %err, "resize pty");
        }
    }
}
