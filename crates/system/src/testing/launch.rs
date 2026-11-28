//! A program runner that answers every start the same way, without running
//! anything.

use std::fs::File;
use std::io;
use std::os::unix::process::ExitStatusExt as _;
use std::path::Path;
use std::process::ExitStatus;

use futures_lite::future::Boxed;

use crate::launch::{ProgramRunner, RunError};

#[derive(Clone, Copy, Debug)]
pub enum FixedRunner {
    /// The program ran and exited with this code.
    Exit(i32),
    /// The program is missing, so it never started.
    Missing,
}

impl ProgramRunner for FixedRunner {
    fn run(
        &self,
        _program: &Path,
        _args: &[String],
        _stderr: Option<File>,
    ) -> Boxed<Result<ExitStatus, RunError>> {
        let answer = *self;
        Box::pin(async move {
            match answer {
                Self::Exit(code) => Ok(ExitStatus::from_raw(code << 8)),
                Self::Missing => Err(RunError::NotStarted(io::Error::from(
                    io::ErrorKind::NotFound,
                ))),
            }
        })
    }
}
