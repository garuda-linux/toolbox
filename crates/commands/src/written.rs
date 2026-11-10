//! Standard output that remembers whether everything reached its reader.

#[cfg(test)]
mod tests;

use std::io::{self, Write};

use toolbox_cli::Exit;

use crate::out::{Said, Settings, kept_errors, lost};

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

    /// The command's own code, or what lost the output. A reader that
    /// stopped early (`| head`) chose to, so that ends quietly.
    ///
    /// # Errors
    ///
    /// The kind of the first write or flush that failed.
    pub fn finish(mut self, exit: Exit) -> Result<Exit, io::ErrorKind> {
        if let Err(error) = self.inner.flush() {
            self.failed.get_or_insert_with(|| error.kind());
        }

        match self.failed {
            None | Some(io::ErrorKind::BrokenPipe) => Ok(exit),
            Some(kind) => Err(kind),
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

/// The exit code once the output is flushed, and the kept errors written.
pub fn ended(
    finished: Result<Exit, io::ErrorKind>,
    mut kept: Vec<Said>,
    settings: Settings,
) -> Exit {
    let exit = finished.unwrap_or_else(|kind| {
        kept.extend(lost(settings, kind));
        Exit::Failure
    });

    write!(io::stderr(), "{}", kept_errors(&kept, exit)).ok();
    exit
}
