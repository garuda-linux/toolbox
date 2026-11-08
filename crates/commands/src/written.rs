//! Standard output that remembers whether everything reached its reader.

#[cfg(test)]
mod tests;

use std::io::{self, Write};

use toolbox_cli::Exit;

pub struct Written<W> {
    inner: W,
    failed: Option<io::ErrorKind>,
}

impl<W: Write> Written<W> {
    pub const fn new(inner: W) -> Self {
        Self {
            inner,
            failed: None,
        }
    }

    /// The command's own code, or a failure when output was lost. A reader
    /// that stopped early (`| head`) chose to, so that ends quietly.
    pub fn finish(mut self, exit: Exit) -> Exit {
        if let Err(error) = self.inner.flush() {
            self.failed.get_or_insert_with(|| error.kind());
        }

        match self.failed {
            None | Some(io::ErrorKind::BrokenPipe) => exit,
            Some(kind) => {
                crate::out::lost(kind);
                Exit::Failure
            }
        }
    }

    fn note<T>(&mut self, result: io::Result<T>) -> io::Result<T> {
        if let Err(error) = &result {
            self.failed.get_or_insert_with(|| error.kind());
        }

        result
    }
}

impl<W: Write> Write for Written<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let result = self.inner.write(buf);
        self.note(result)
    }

    fn flush(&mut self) -> io::Result<()> {
        let result = self.inner.flush();
        self.note(result)
    }
}
