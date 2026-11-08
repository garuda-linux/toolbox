//! The built helper with arguments only and nothing on stdin, as the app
//! runs its reads.

use std::io;
use std::process::{Command, Output};

pub fn output(args: &[&str]) -> io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_garuda-toolbox-helper"))
        .args(args)
        .output()
}
