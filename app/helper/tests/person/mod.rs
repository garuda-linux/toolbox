//! The built helper asked as the app asks for the lock flags.

use std::io;
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::process::{Command, Output, Stdio};

use rustix::process::{Uid, geteuid};
use rustix::thread::set_thread_res_uid;

/// A person's uid, as the app runs under. The Arch image need not have
/// the account: only the socket's peer counts.
const PERSON: u32 = 1000;

/// The helper with stdin a socket whose peer is a person. Run as root,
/// this thread alone takes the person's uid while it makes the socket,
/// which is what the peer reads.
pub fn output(args: &[&str]) -> io::Result<Output> {
    let root = geteuid().is_root();

    if root {
        set_thread_res_uid(None, Some(Uid::from_raw(PERSON)), None)?;
    }

    let pair = UnixStream::pair();

    if root {
        set_thread_res_uid(None, Some(Uid::ROOT), None)?;
    }

    let (app, helper) = pair?;
    let output = Command::new(env!("CARGO_BIN_EXE_garuda-toolbox-helper"))
        .args(args)
        .stdin(Stdio::from(OwnedFd::from(helper)))
        .output();

    drop(app);
    output
}
