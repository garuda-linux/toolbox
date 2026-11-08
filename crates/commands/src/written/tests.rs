use std::io::{self, Write};

use toolbox_cli::Exit;

use super::Written;

struct Failing(io::ErrorKind);

impl Write for Failing {
    fn write(&mut self, _bytes: &[u8]) -> io::Result<usize> {
        Err(io::Error::from(self.0))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn output_that_reached_its_reader_keeps_the_exit_code() {
    let mut out = Written::new(Vec::new());
    write!(out, "rows").ok();

    assert_eq!(out.finish(Exit::Attention), Exit::Attention);
}

#[test]
fn output_that_failed_ends_as_a_failure() {
    let mut out = Written::new(Failing(io::ErrorKind::StorageFull));
    write!(out, "rows").ok();

    assert_eq!(out.finish(Exit::Success), Exit::Failure);
}

#[test]
fn a_reader_that_stopped_reading_ends_the_command_quietly() {
    let mut out = Written::new(Failing(io::ErrorKind::BrokenPipe));
    write!(out, "rows").ok();

    assert_eq!(out.finish(Exit::Success), Exit::Success);
}
