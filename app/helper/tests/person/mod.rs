//! The built helper asked as the app asks for the lock flags.

use std::io;
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt as _;
use std::process::{Command, Output, Stdio};

use rustix::process::{Uid, geteuid};
use rustix::thread::set_thread_res_uid;

/// A person's uid, as the app runs under. The Arch image need not have
/// the account: only the socket's peer counts.
const PERSON: u32 = 1000;

/// The uid of `nobody`, which owns nothing the tests change.
const NOBODY: u32 = 0xFFFE;

/// The helper with stdin a socket whose peer is a person. Run as root,
/// this thread alone takes the person's uid while it makes the socket,
/// which is what the peer reads. With `unprivileged` the helper runs as
/// `nobody` too, so it is no root whoever runs the suite.
pub fn output(args: &[&str], unprivileged: bool) -> io::Result<Output> {
    run(command(args, unprivileged))
}

/// The helper as [`output`] runs it, but not yet started.
pub fn command(args: &[&str], unprivileged: bool) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_garuda-toolbox-helper"));

    command.args(args);

    if unprivileged && geteuid().is_root() {
        command.uid(NOBODY).gid(NOBODY);
    }

    command
}

/// A socket pair for the helper's stdin: the app's end, and the helper's,
/// whose peer is `uid`. As root, this thread alone takes `uid` while it
/// makes the pair.
pub fn socket_of(uid: u32) -> io::Result<(UnixStream, OwnedFd)> {
    let root = geteuid().is_root();

    if root {
        set_thread_res_uid(None, Some(Uid::from_raw(uid)), None)?;
    }

    let pair = UnixStream::pair();

    if root {
        set_thread_res_uid(None, Some(Uid::ROOT), None)?;
    }

    let (app, helper) = pair?;
    Ok((app, OwnedFd::from(helper)))
}

fn run(mut command: Command) -> io::Result<Output> {
    let (app, helper) = socket_of(PERSON)?;
    let output = command.stdin(Stdio::from(helper)).output();

    drop(app);
    output
}
